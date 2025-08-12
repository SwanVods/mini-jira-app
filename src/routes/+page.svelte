<script lang="ts">
  import { onMount } from 'svelte';
  import '../app.css'

  interface Issue {
    key: string;
    summary: string;
  }

  interface Status {
    message: string;
    type: string;
    visible: boolean;
  }

  interface LoginForm {
    email: string;
    token: string;
    jiraUrl: string;
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
  let userEmail = $state('');
  let accessToken = $state('');
  let jiraBaseUrl = $state('');
  let assignedIssues = $state<Issue[]>([]);
  let status = $state<Status>({ message: '', type: '', visible: false });

  // Form states
  let loginForm = $state<LoginForm>({
    email: '',
    token: '',
    jiraUrl: ''
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
    if (saved.email) {
      loginForm.email = saved.email;
      loginForm.jiraUrl = saved.jiraUrl || '';
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

  async function simulateJiraLogin(): Promise<boolean> {
    // Simulate API delay
    await new Promise(resolve => setTimeout(resolve, 1000));
    return true;
  }

  async function simulateLoadIssues(): Promise<Issue[]> {
    // Simulate API delay
    await new Promise(resolve => setTimeout(resolve, 800));
    
    // Mock data - replace with actual JIRA API integration
    return [
      { key: 'PROJ-123', summary: 'Implement user authentication' },
      { key: 'PROJ-124', summary: 'Fix login page styling' },
      { key: 'PROJ-125', summary: 'Add error handling for API calls' },
      { key: 'PROJ-126', summary: 'Update documentation' },
      { key: 'PROJ-127', summary: 'Performance optimization' }
    ];
  }

  async function simulateSubmitWorkLog(issueKey: string, date: string, timeSpent: string, description: string): Promise<boolean> {
    // Simulate API delay
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    console.log('Work log data:', {
      issueKey,
      date,
      timeSpent,
      description
    });
    
    return true;
  }

  async function handleLogin() {
    const { email, token, jiraUrl } = loginForm;

    if (!email || !token || !jiraUrl) {
      showStatus('Please fill in all fields', 'error');
      return;
    }

    showStatus('Connecting to JIRA...', 'loading');

    try {
      // Save credentials
      userEmail = email;
      accessToken = token;
      jiraBaseUrl = jiraUrl.replace(/\/$/, '');

      // Save to localStorage for convenience (demo only)
      localStorage.setItem('jiraCredentials', JSON.stringify({
        email: email,
        jiraUrl: jiraUrl
      }));

      // Simulate API call
      await simulateJiraLogin();
      
      // Load assigned issues
      await loadAssignedIssues();
      
      isLoggedIn = true;
      showStatus('Successfully connected to JIRA!', 'success');
      
    } catch (error: any) {
      showStatus('Failed to connect: ' + error.message, 'error');
    }
  }

  async function loadAssignedIssues() {
    showStatus('Loading assigned issues...', 'loading');
    
    try {
      const issues = await simulateLoadIssues();
      assignedIssues = issues;
      showStatus('Issues loaded successfully!', 'success');
    } catch (error: any) {
      showStatus('Failed to load issues: ' + error.message, 'error');
    }
  }

  async function handleRefreshIssues() {
    if (!userEmail || !accessToken) {
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
      await simulateSubmitWorkLog(issueKey, workDate, timeSpent, description);
      showStatus('Work log submitted successfully!', 'success');
      
      // Reset form
      workLogForm.timeAmount = '';
      workLogForm.description = '';
      
    } catch (error: any) {
      showStatus('Failed to submit work log: ' + error.message, 'error');
    }
  }

  function handleLogout() {
    isLoggedIn = false;
    userEmail = '';
    accessToken = '';
    jiraBaseUrl = '';
    assignedIssues = [];
    
    // Clear form fields
    loginForm.token = '';
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

    <!-- Status Messages -->
    {#if status.visible}
      <div class="p-2 rounded-md mb-4 text-center font-medium text-sm transition-all max-w-4xl mx-auto {
        status.type === 'success' ? 'bg-green-500/20 text-green-400 border border-green-500/30' :
        status.type === 'error' ? 'bg-red-500/20 text-red-400 border border-red-500/30' :
        'bg-blue-500/20 text-blue-400 border border-blue-500/30'
      }">
        {status.message}
      </div>
    {/if}

    {#if !isLoggedIn}
      <!-- Login Section -->
      <div class="space-y-4 max-w-2xl mx-auto">
        <div>
          <!-- svelte-ignore a11y_label_has_associated_control -->
          <label class="block mb-1.5 text-slate-400 font-medium text-sm uppercase tracking-wide">
            JIRA Base URL
          </label>
          
          <input
            type="url"
            bind:value={loginForm.jiraUrl}
            placeholder="https://yourcompany.atlassian.net"
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
          <span class="text-purple-400 font-medium text-sm">{userEmail}</span>
          <button
            onclick={handleLogout}
            class="bg-transparent border border-red-400 text-red-400 px-3 py-1 rounded-2xl cursor-pointer text-xs hover:bg-red-400 hover:text-slate-800 transition-all"
          >
            Logout
          </button>
        </div>

        <div>
          <!-- svelte-ignore a11y_label_has_associated_control -->
          <label class="block mb-1.5 text-slate-400 font-medium text-sm uppercase tracking-wide">
            JIRA Issue
          </label>
          <select
            bind:value={workLogForm.issueKey}
            class="w-full p-2.5 border border-slate-600 rounded-lg text-sm bg-slate-700/80 text-slate-200 cursor-pointer focus:outline-none focus:border-blue-500 focus:bg-slate-700 focus:shadow-lg focus:shadow-blue-500/10 transition-all"
            required
          >
            <option value="">Select an issue...</option>
            {#each assignedIssues as issue}
              <option value={issue.key}>
                {issue.key} - {issue.summary}
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
          <label class="block mb-1.5 text-slate-400 font-medium text-sm uppercase tracking-wide">
            Date
          </label>
          <input
            type="date"
            bind:value={workLogForm.workDate}
            class="w-full p-2.5 border border-slate-600 rounded-lg text-sm bg-slate-700/80 text-slate-200 focus:outline-none focus:border-blue-500 focus:bg-slate-700 focus:shadow-lg focus:shadow-blue-500/10 transition-all"
            required
          />
        </div>
        
        <div>
          <label class="block mb-1.5 text-slate-400 font-medium text-sm uppercase tracking-wide">
            Time Spent
          </label>
          <div class="flex gap-2">
            <div class="flex-[2]">
              <input
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
          <label class="block mb-1.5 text-slate-400 font-medium text-sm uppercase tracking-wide">
            Work Description
          </label>
          <textarea
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
