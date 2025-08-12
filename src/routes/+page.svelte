<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { 
    isPermissionGranted, 
    requestPermission, 
    sendNotification 
  } from '@tauri-apps/plugin-notification';
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
    email: string;
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

  // Debug mode - only show background controls in development
  const isDebugMode = import.meta.env.DEV || import.meta.env.MODE === 'development';

  // Form states
  let loginForm = $state<LoginForm>({
    baseUrl: 'https://ariefgraifhan.atlassian.net',
    email: '',
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
    if (saved.email) {
      loginForm.email = saved.email;
    }
    if (saved.baseUrl) {
      loginForm.baseUrl = saved.baseUrl;
    }

    // Setup notification permissions
    setupNotifications();

    // Listen for daily reminder events from Rust
    listen('daily-reminder', () => {
      sendDailyReminder();
    });

    // Listen for test notification events from system tray
    listen('test-notification', () => {
      handleTestNotification();
    });
  });

  async function setupNotifications() {
    try {
      let permissionGranted = await isPermissionGranted();
      
      if (!permissionGranted) {
        const permission = await requestPermission();
        permissionGranted = permission === 'granted';
      }

      if (!permissionGranted) {
        console.warn('Notification permission not granted');
      }
    } catch (error) {
      console.error('Error setting up notifications:', error);
    }
  }

  async function sendDailyReminder() {
    try {
      await sendNotification({
        title: 'JIRA Work Log Reminder',
        body: 'Ayo catat kerjaanmu! Jangan lupa masukin worklog jira harian, ya.'
      });
    } catch (error) {
      console.error('Error sending daily reminder:', error);
    }
  }

  function showStatus(message: string, type = 'loading') {
    status = { message, type, visible: true };
    
    if (type === 'success' || type === 'error') {
      setTimeout(() => {
        status = { ...status, visible: false };
      }, 5000);
    }
  }

  async function handleLogin() {
    const { baseUrl, email, token } = loginForm;

    if (!baseUrl || !email || !token) {
      showStatus('Please fill in all fields', 'error');
      return;
    }

    showStatus('Connecting to JIRA...', 'loading');

    try {
      // Call Rust function to connect to JIRA
      const isConnected = await invoke<boolean>('connect_to_jira', {
        baseUrl: baseUrl.replace(/\/$/, ''), // Remove trailing slash
        email: email,
        accessToken: token
      });

      if (isConnected) {
        // Save credentials
        localStorage.setItem('jiraCredentials', JSON.stringify({
          email: email,
          token: token,
          baseUrl: baseUrl
        }));

        // Load assigned issues
        await loadAssignedIssues();
        
        isLoggedIn = true;
        showStatus('Successfully connected to JIRA!', 'success');
      } else {
        showStatus('Authentication failed. Please check your credentials.', 'error');
      }
      
    } catch (error: any) {
      let errorMessage = 'Connection failed: ';
      
      // Safely get the error message
      const errorText = error?.message || error?.toString() || '';
      
      // Parse different types of errors
      if (errorText.includes('network')) {
        errorMessage += 'Network error. Please check your internet connection.';
      } else if (errorText.includes('timeout')) {
        errorMessage += 'Connection timeout. The server may be slow or unreachable.';
      } else if (errorText.includes('ssl') || errorText.includes('certificate')) {
        errorMessage += 'SSL certificate error. Please check your JIRA URL.';
      } else if (errorText.includes('401') || errorText.includes('Unauthorized')) {
        errorMessage += 'Invalid credentials. Please check your access token.';
      } else if (errorText.includes('404') || errorText.includes('Not Found')) {
        errorMessage += 'JIRA server not found. Please check your base URL.';
      } else if (errorText.includes('403') || errorText.includes('Forbidden')) {
        errorMessage += 'Access denied. Your token may not have sufficient permissions.';
      } else if (errorText.includes('500')) {
        errorMessage += 'Server error. The JIRA server is experiencing issues.';
      } else if (errorText) {
        errorMessage += errorText;
      } else {
        errorMessage += 'Unknown error occurred.';
      }
      
      showStatus(errorMessage, 'error');
    }
  }

  async function loadAssignedIssues() {
    showStatus('Loading assigned issues...', 'loading');
    
    try {
      const issues = await invoke<Issue[]>('get_assigned_issues');
      assignedIssues = issues;
      showStatus('Issues loaded successfully!', 'success');
    } catch (error: any) {
      const errorText = error?.message || error?.toString() || 'Unknown error';
      showStatus('Failed to load issues: ' + errorText, 'error');
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
      // Convert date to JIRA-compatible format: yyyy-MM-ddTHH:mm:ss.SSS+0000
      const dateObj = new Date(workDate + 'T09:00:00.000');
      const startedDateTime = dateObj.toISOString().replace('Z', '+0000');
      
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
      const errorText = error?.message || error?.toString() || 'Unknown error';
      showStatus('Failed to submit work log: ' + errorText, 'error');
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
    loginForm.email = '';
    loginForm.token = '';
    loginForm.baseUrl = '';
    workLogForm.issueKey = '';
    workLogForm.timeAmount = '';
    workLogForm.description = '';
    
    status = { message: '', type: '', visible: false };
  }

  async function handleHideToTray() {
    try {
      await invoke('hide_to_tray');
      showStatus('App minimized to system tray', 'success');
    } catch (error) {
      console.error('Error hiding to tray:', error);
    }
  }

  async function handleTestNotification() {
    try {
      await sendNotification({
        title: 'JIRA Work Log',
        body: 'Test notification - Background functionality is working!'
      });
      showStatus('Test notification sent!', 'success');
    } catch (error) {
      console.error('Error sending notification:', error);
      showStatus('Failed to send notification', 'error');
    }
  }
</script>

<div class="min-h-screen w-screen overflow-hidden p-0 m-0 bg-slate-800">
  <div class="min-h-screen w-full p-6 overflow-y-scroll scrollbar-hide">
    
    <h1 class="text-center text-slate-200 mb-5 font-light text-3xl max-w-4xl mx-auto">
      Mini Jira Logger
    </h1>

    {#if !isLoggedIn}
      <!-- Login Section -->
      <div class="space-y-4 max-w-2xl mx-auto">
        
        <!-- Status Messages for Login - Fixed height container -->
        <div class="min-h-[60px] flex items-center">
          {#if status.visible}
            <div class="w-full p-3 rounded-lg text-center font-medium text-sm transition-all {
              status.type === 'success' ? 'bg-green-500/20 text-green-400 border border-green-500/30' :
              status.type === 'error' ? 'bg-red-500/20 text-red-400 border border-red-500/30' :
              'bg-blue-500/20 text-blue-400 border border-blue-500/30'
            }">
              {#if status.type === 'loading'}
                <div class="flex items-center justify-center gap-2">
                  <div class="w-4 h-4 border-2 border-blue-400 border-t-transparent rounded-full animate-spin"></div>
                  {status.message}
                </div>
              {:else}
                {status.message}
              {/if}
            </div>
          {/if}
        </div>
        
        <div>
          <label for="jiraBaseUrl" class="block mb-1.5 text-slate-400 font-medium text-sm uppercase tracking-wide">
            JIRA Base URL
          </label>
          
          <input
            id="jiraBaseUrl"
            type="url"
            bind:value={loginForm.baseUrl}
            placeholder="https://yourcompany.atlassian.net"
            class="w-full p-2.5 border border-slate-600 rounded-lg text-sm bg-slate-700/80 text-slate-200 placeholder-slate-500 focus:outline-none focus:border-blue-500 focus:bg-slate-700 focus:shadow-lg focus:shadow-blue-500/10 transition-all disabled:opacity-50 disabled:cursor-not-allowed disabled:transform-none"
            disabled
            required
          />
        </div>

        <div>
          <label for="jiraEmail" class="block mb-1.5 text-slate-400 font-medium text-sm uppercase tracking-wide">
            JIRA Email
          </label>
          
          <input
            id="jiraEmail"
            type="email"
            bind:value={loginForm.email}
            placeholder="your.email@company.com"
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
          <p class="text-xs text-slate-500 mt-1">
            Generate an API token in your JIRA account settings → Security → API tokens.
          </p>
        </div>
        
        <button
          onclick={handleLogin}
          disabled={status.visible && status.type === 'loading'}
          class="w-full p-2.5 bg-gradient-to-r from-orange-500 to-red-600 text-white border-none rounded-lg text-sm font-semibold cursor-pointer uppercase tracking-wide mt-2 hover:-translate-y-0.5 hover:shadow-lg hover:shadow-orange-500/40 active:translate-y-0 transition-all disabled:opacity-50 disabled:cursor-not-allowed disabled:transform-none"
        >
          {#if status.visible && status.type === 'loading'}
            <div class="flex items-center justify-center gap-2">
              <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
              Connecting...
            </div>
          {:else}
            Connect to JIRA
          {/if}
        </button>
      </div>
    {:else}
      <!-- Work Log Section -->
      <div class="space-y-4 max-w-2xl mx-auto">
        <div class="bg-blue-500/15 p-2.5 rounded-lg flex justify-between items-center">
          <div class="flex flex-col min-h-[40px] justify-center">
            <span class="text-purple-400 font-medium text-sm">{loginForm.email}</span>
            <div class="min-h-[16px] flex items-start">
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
          </div>
          <button
            onclick={handleLogout}
            class="bg-transparent border border-red-400 text-red-400 px-3 py-1 rounded-2xl cursor-pointer text-xs hover:bg-red-400 hover:text-slate-800 transition-all"
          >
            Logout
          </button>
        </div>

        <!-- Background Controls Section (Debug Mode Only) -->
        {#if isDebugMode}
          <div class="bg-slate-700/50 p-3 rounded-lg">
            <h3 class="text-slate-300 font-medium text-sm mb-3 uppercase tracking-wide">Background Controls</h3>
            <div class="flex gap-2 flex-wrap">
              <button
                onclick={handleHideToTray}
                class="flex-1 p-2 bg-gradient-to-r from-slate-600 to-slate-700 text-white border-none rounded-lg text-xs font-semibold cursor-pointer hover:-translate-y-0.5 hover:shadow-md transition-all"
              >
                Hide to Tray
              </button>
              <button
                onclick={handleTestNotification}
                class="flex-1 p-2 bg-gradient-to-r from-green-600 to-green-700 text-white border-none rounded-lg text-xs font-semibold cursor-pointer hover:-translate-y-0.5 hover:shadow-md transition-all"
              >
                Test Notification
              </button>
            </div>
            <p class="text-xs text-slate-400 mt-2">
              • App runs in background when closed<br>
              • Daily reminder at 5 PM<br>
              • Access via system tray icon
            </p>
          </div>
        {/if}

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
            class="w-full p-2 bg-gradient-to-r from-orange-500 to-red-600 text-white border-none rounded-lg text-xs font-semibold cursor-pointer mt-2 hover:-translate-y-0.5 hover:shadow-md transition-all"
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
          class="w-full p-2.5 bg-gradient-to-r from-orange-500 to-red-600 text-white border-none rounded-lg text-sm font-semibold cursor-pointer uppercase tracking-wide mt-2 hover:-translate-y-0.5 hover:shadow-lg hover:shadow-orange-500/40 active:translate-y-0 transition-all"
        >
          Log Work
        </button>
      </div>
    {/if}
  </div>
</div>
