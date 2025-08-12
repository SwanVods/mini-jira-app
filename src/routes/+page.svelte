<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import '../app.css'

  interface Issue {
    key: string;
    fields: {
      summary: string;
      status: {
        name: string;
      };
      assignee?: {
        display_name: string;
        email_address: string;
      };
    };
  }

  interface Status {
    message: string;
    type: string;
    visible: boolean;
  }

  interface LoginForm {
    baseUrl: string;
    token: string;
  }

  interface WorkLogForm {
    issueKey: string;
    workDate: string;
    timeAmount: string;
    timeUnit: string;
    description: string;
  }

  // State variables
  let isLoggedIn = $state(false);
  let assignedIssues = $state<Issue[]>([]);
  let status = $state<Status>({ message: '', type: '', visible: false });

  // Form states
  let loginForm = $state<LoginForm>({
    baseUrl: '',
    token: '',
  });

  let workLogForm = $state<WorkLogForm>({
    issueKey: '',
    workDate: '',
    timeAmount: '',
    timeUnit: 'h',
    description: ''
  });

  onMount(() => {
    // Set today's date as default
    const today = new Date().toISOString().split('T')[0];
    workLogForm.workDate = today;
    
    // Load saved credentials (for demo purposes)
    const saved = JSON.parse(localStorage.getItem('jiraCredentials') || '{}');
    if (saved.token) {
      loginForm.token = saved.token;
    }
    if (saved.baseUrl) {
      loginForm.baseUrl = saved.baseUrl;
    }
  });

  function showStatus(message: string, type = 'loading') {
    status = { message, type, visible: true };
    
    if (type === 'success' || type === 'error') {
      setTimeout(() => {
        status = { ...status, visible: false };
      }, 5000);
    }
  }

  async function handleLogin() {
    const { baseUrl, token } = loginForm;

    if (!baseUrl || !token) {
      showStatus('Please fill in all fields', 'error');
      return;
    }

    showStatus('Connecting to JIRA...', 'loading');

    try {
      // Call Rust function to connect to JIRA
      const isConnected = await invoke<boolean>('connect_to_jira', {
        baseUrl: baseUrl.replace(/\/$/, ''), // Remove trailing slash
        accessToken: token
      });

      if (isConnected) {
        // Save credentials
        localStorage.setItem('jiraCredentials', JSON.stringify({
          token: token,
          baseUrl: baseUrl
        }));

        // Load assigned issues
        await loadAssignedIssues();
        
        isLoggedIn = true;
        showStatus('Successfully connected to JIRA!', 'success');
      } else {
        showStatus('Failed to connect to JIRA', 'error');
      }
      
    } catch (error: any) {
      showStatus('Failed to connect: ' + error.message, 'error');
    }
  }

  async function loadAssignedIssues() {
    showStatus('Loading assigned issues...', 'loading');
    
    try {
      const issues = await invoke<Issue[]>('get_assigned_issues');
      assignedIssues = issues;
      showStatus('Issues loaded successfully!', 'success');
    } catch (error: any) {
      showStatus('Failed to load issues: ' + error.message, 'error');
    }
  }

  async function handleRefreshIssues() {
    if (!isLoggedIn) {
      showStatus('Please login first', 'error');
      return;
    }
    
    await loadAssignedIssues();
  }

  async function handleSubmitWorkLog() {
    const { issueKey, workDate, timeAmount, timeUnit, description } = workLogForm;

    if (!issueKey || !workDate || !timeAmount || !description) {
      showStatus('Please fill in all required fields', 'error');
      return;
    }

    const timeSpent = timeAmount + timeUnit;
    showStatus('Submitting work log...', 'loading');

    try {
      // Convert date to ISO format with time
      const startedDateTime = new Date(workDate + 'T09:00:00.000Z').toISOString();
      
      await invoke('create_worklog', {
        issueKey: issueKey,
        description: description,
        started: startedDateTime,
        timeSpent: timeSpent
      });
      
      showStatus('Work log submitted successfully!', 'success');
      
      // Reset form
      workLogForm.timeAmount = '';
      workLogForm.description = '';
      
    } catch (error: any) {
      showStatus('Failed to submit work log: ' + error.message, 'error');
    }
  }

  async function handleLogout() {
    try {
      await invoke('disconnect_from_jira');
    } catch (error) {
      console.warn('Error disconnecting:', error);
    }
    
    isLoggedIn = false;
    assignedIssues = [];
    
    // Clear form fields
    loginForm.token = '';
    loginForm.baseUrl = '';
    workLogForm.issueKey = '';
    workLogForm.timeAmount = '';
    workLogForm.description = '';
    
    status = { message: '', type: '', visible: false };
  }
</script>

<div class="min-h-screen w-screen overflow-hidden p-0 m-0 bg-slate-800">
  <div class="min-h-screen w-full p-6 overflow-y-auto">
    
    <h1 class="text-center text-slate-200 mb-5 font-light text-3xl max-w-4xl mx-auto">
      JIRA Work Log
    </h1>

    {#if !isLoggedIn}
      <!-- Login Section -->
      <div class="space-y-4 max-w-2xl mx-auto">
        <div>
          <label for="jiraBaseUrl" class="block mb-1.5 text-slate-400 font-medium text-sm uppercase tracking-wide">
            JIRA Base URL
          </label>
          
          <input
            id="jiraBaseUrl"
            type="url"
            bind:value={loginForm.baseUrl}
            placeholder="https://yourcompany.atlassian.net"
            class="w-full p-2.5 border border-slate-600 rounded-lg text-sm bg-slate-700/80 text-slate-200 placeholder-slate-500 focus:outline-none focus:border-blue-500 focus:bg-slate-700 focus:shadow-lg focus:shadow-blue-500/10 transition-all"
            required
          />
        </div>
        
        <div>

          <label for="jiraAccessToken" class="block mb-1.5 text-slate-400 font-medium text-sm uppercase tracking-wide">
            JIRA Access Token
          </label>
          
          <input
            id="jiraAccessToken"
            type="password"
            bind:value={loginForm.token}
            placeholder="Your JIRA access token"
            class="w-full p-2.5 border border-slate-600 rounded-lg text-sm bg-slate-700/80 text-slate-200 placeholder-slate-500 focus:outline-none focus:border-blue-500 focus:bg-slate-700 focus:shadow-lg focus:shadow-blue-500/10 transition-all"
            required
          />
        </div>
        
        <button
          onclick={handleLogin}
          class="w-full p-2.5 bg-gradient-to-r from-blue-600 to-purple-600 text-white border-none rounded-lg text-sm font-semibold cursor-pointer uppercase tracking-wide mt-2 hover:-translate-y-0.5 hover:shadow-lg hover:shadow-blue-500/40 active:translate-y-0 transition-all"
        >
          Connect to JIRA
        </button>
      </div>
    {:else}
      <!-- Work Log Section -->
      <div class="space-y-4 max-w-2xl mx-auto">
        <div class="bg-blue-500/15 p-2.5 rounded-lg flex justify-between items-center">
          <div class="flex flex-col">
            <span class="text-purple-400 font-medium text-sm">Using Access Token</span>
            {#if status.visible}
              <span class="text-xs font-normal {
                status.type === 'success' ? 'text-green-400' :
                status.type === 'error' ? 'text-red-400' :
                'text-blue-400'
              }">
                {status.message}
              </span>
            {/if}
          </div>
          <button
            onclick={handleLogout}
            class="bg-transparent border border-red-400 text-red-400 px-3 py-1 rounded-2xl cursor-pointer text-xs hover:bg-red-400 hover:text-slate-800 transition-all"
          >
            Logout
          </button>
        </div>

        <div>

          <label for="issueKey" class="block mb-1.5 text-slate-400 font-medium text-sm uppercase tracking-wide">
            JIRA Issue
          </label>
          <select
            id="issueKey"
            bind:value={workLogForm.issueKey}
            class="w-full p-2.5 border border-slate-600 rounded-lg text-sm bg-slate-700/80 text-slate-200 cursor-pointer focus:outline-none focus:border-blue-500 focus:bg-slate-700 focus:shadow-lg focus:shadow-blue-500/10 transition-all"
            required
          >
            <option value="">Select an issue...</option>
            {#each assignedIssues as issue}
              <option value={issue.key}>
                {issue.key} - {issue.fields.summary}
              </option>
            {/each}
          </select>
          <button
            onclick={handleRefreshIssues}
            class="w-full p-2 bg-gradient-to-r from-slate-600 to-slate-700 text-white border-none rounded-lg text-xs font-semibold cursor-pointer mt-2 hover:-translate-y-0.5 hover:shadow-md transition-all"
          >
            Refresh Issues
          </button>
        </div>
        
        <div>
          <label for="workDate" class="block mb-1.5 text-slate-400 font-medium text-sm uppercase tracking-wide">
            Date
          </label>
          <input
            id="workDate"
            type="date"
            bind:value={workLogForm.workDate}
            class="w-full p-2.5 border border-slate-600 rounded-lg text-sm bg-slate-700/80 text-slate-200 focus:outline-none focus:border-blue-500 focus:bg-slate-700 focus:shadow-lg focus:shadow-blue-500/10 transition-all"
            required
          />
        </div>
        
        <div>
          <label for="timeAmount" class="block mb-1.5 text-slate-400 font-medium text-sm uppercase tracking-wide">
            Time Spent
          </label>
          <div class="flex gap-2">
            <div class="flex-[2]">
              <input
                id="timeAmount"
                type="number"
                bind:value={workLogForm.timeAmount}
                placeholder="1"
                min="0.25"
                step="0.25"
                class="w-full p-2.5 border border-slate-600 rounded-lg text-sm bg-slate-700/80 text-slate-200 placeholder-slate-500 focus:outline-none focus:border-blue-500 focus:bg-slate-700 focus:shadow-lg focus:shadow-blue-500/10 transition-all"
                required
              />
            </div>
            <div class="flex-1">
              <select
                bind:value={workLogForm.timeUnit}
                class="w-full p-2.5 border border-slate-600 rounded-lg text-sm bg-slate-700/80 text-slate-200 cursor-pointer focus:outline-none focus:border-blue-500 focus:bg-slate-700 focus:shadow-lg focus:shadow-blue-500/10 transition-all"
              >
                <option value="h">Hours</option>
                <option value="m">Minutes</option>
                <option value="d">Days</option>
              </select>
            </div>
          </div>
        </div>
        
        <div>
          <label for="workDescription" class="block mb-1.5 text-slate-400 font-medium text-sm uppercase tracking-wide">
            Work Description
          </label>
          <textarea
            id="workDescription"
            bind:value={workLogForm.description}
            placeholder="Describe the work you performed..."
            class="w-full p-2.5 border border-slate-600 rounded-lg text-sm bg-slate-700/80 text-slate-200 placeholder-slate-500 resize-vertical min-h-[70px] font-inherit focus:outline-none focus:border-blue-500 focus:bg-slate-700 focus:shadow-lg focus:shadow-blue-500/10 transition-all"
            required
          ></textarea>
        </div>
        
        <button
          onclick={handleSubmitWorkLog}
          class="w-full p-2.5 bg-gradient-to-r from-blue-600 to-purple-600 text-white border-none rounded-lg text-sm font-semibold cursor-pointer uppercase tracking-wide mt-2 hover:-translate-y-0.5 hover:shadow-lg hover:shadow-blue-500/40 active:translate-y-0 transition-all"
        >
          Log Work
        </button>
      </div>
    {/if}
  </div>
</div>
