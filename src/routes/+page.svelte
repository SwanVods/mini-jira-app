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
    token: string;
  }

  interface WorkLogForm {
    issueKey: string;
    workDate: string;
    timeAmount: string;
    timeUnit: string;
    description: string;
  }

  let isLoggedIn = $state(false);
  let assignedIssues = $state<Issue[]>([]);
  let status = $state<Status>({ message: '', type: '', visible: false });
  let isDarkMode = $state(true);

  const isDebugMode = import.meta.env.DEV || import.meta.env.MODE === 'development';

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
    const today = new Date().toISOString().split('T')[0];
    workLogForm.workDate = today;
    
    const saved = JSON.parse(localStorage.getItem('jiraCredentials') || '{}');
    if (saved.token) {
      loginForm.token = saved.token;
    }
    if (saved.baseUrl) {
      loginForm.baseUrl = saved.baseUrl;
    }

    const savedTheme = localStorage.getItem('theme');
    if (savedTheme) {
      isDarkMode = savedTheme === 'dark';
    }

    setupNotifications();

    listen('daily-reminder', () => {
      sendDailyReminder();
    });

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
    const { baseUrl, token } = loginForm;

    if (!baseUrl || !token) {
      showStatus('Please fill in all fields', 'error');
      return;
    }

    showStatus('Connecting to JIRA...', 'loading');

    try {
      const isConnected = await invoke<boolean>('connect_to_jira', {
        baseUrl: baseUrl.replace(/\/$/, ''),
        accessToken: token
      });

      if (isConnected) {
        localStorage.setItem('jiraCredentials', JSON.stringify({
          token: token,
          baseUrl: baseUrl
        }));

        await loadAssignedIssues();
        
        isLoggedIn = true;
        showStatus('Successfully connected to JIRA!', 'success');
      } else {
        showStatus('Authentication failed. Please check your credentials.', 'error');
      }
      
    } catch (error: any) {
      let errorMessage = 'Connection failed: ';
      
      const errorText = error?.message || error?.toString() || '';
      
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
      const dateObj = new Date(workDate + 'T09:00:00.000');
      const startedDateTime = dateObj.toISOString().replace('Z', '+0000');
      
      await invoke('create_worklog', {
        issueKey: issueKey,
        description: description,
        started: startedDateTime,
        timeSpent: timeSpent
      });
      
      showStatus('Work log submitted successfully!', 'success');
      
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

  function toggleTheme() {
    isDarkMode = !isDarkMode;
    localStorage.setItem('theme', isDarkMode ? 'dark' : 'light');
  }
</script>

<div class="min-h-screen w-screen overflow-hidden p-0 m-0 {isDarkMode ? 'bg-slate-800' : 'bg-gray-50'}">
  <div class="min-h-screen w-full p-6 overflow-y-scroll scrollbar-hide {!isLoggedIn ? 'flex flex-col' : ''}">
    
    <!-- Theme Toggle Button -->
    <div class="absolute top-4 right-4 z-10">
      <button
        onclick={toggleTheme}
        class="p-2 rounded-lg border transition-all {isDarkMode 
          ? 'bg-slate-700 border-slate-600 text-slate-200 hover:bg-slate-600' 
          : 'bg-white border-gray-300 text-gray-700 hover:bg-gray-50'}"
        title={isDarkMode ? 'Switch to Light Mode' : 'Switch to Dark Mode'}
      >
        {#if isDarkMode}
          ☀️
        {:else}
          🌙
        {/if}
      </button>
    </div>

    <h1 class="text-center mb-5 font-light text-3xl max-w-4xl mx-auto {isDarkMode ? 'text-slate-200' : 'text-gray-800'}">
      Mini Jira Logger
    </h1>

    {#if !isLoggedIn}
      <!-- Login Section -->
      <div class="flex-1 flex flex-col justify-center">
        <div class="space-y-4 max-w-2xl mx-auto w-full">
        <div class="min-h-[60px] flex items-center">
          {#if status.visible}
            <div class="w-full p-3 rounded-lg text-center font-medium text-sm transition-all {
              status.type === 'success' ? (isDarkMode ? 'bg-green-500/20 text-green-400 border border-green-500/30' : 'bg-green-100 text-green-700 border border-green-300') :
              status.type === 'error' ? (isDarkMode ? 'bg-red-500/20 text-red-400 border border-red-500/30' : 'bg-red-100 text-red-700 border border-red-300') :
              (isDarkMode ? 'bg-blue-500/20 text-blue-400 border border-blue-500/30' : 'bg-blue-100 text-blue-700 border border-blue-300')
            }">
              {#if status.type === 'loading'}
                <div class="flex items-center justify-center gap-2">
                  <div class="w-4 h-4 border-2 border-t-transparent rounded-full animate-spin {isDarkMode ? 'border-blue-400' : 'border-blue-600'}"></div>
                  {status.message}
                </div>
              {:else}
                {status.message}
              {/if}
            </div>
          {/if}
        </div>
        
        <div>
          <label for="jiraBaseUrl" class="block mb-1.5 font-medium text-sm uppercase tracking-wide {isDarkMode ? 'text-slate-400' : 'text-gray-600'}">
            JIRA Base URL
          </label>
          
          <input
            id="jiraBaseUrl"
            type="url"
            bind:value={loginForm.baseUrl}
            placeholder="https://yourcompany.domain.net"
            class="w-full p-2.5 border rounded-lg text-sm transition-all focus:outline-none focus:shadow-lg {isDarkMode 
              ? 'border-slate-600 bg-slate-700/80 text-slate-200 placeholder-slate-500 focus:border-blue-500 focus:bg-slate-700 focus:shadow-blue-500/10' 
              : 'border-gray-300 bg-white text-gray-900 placeholder-gray-400 focus:border-blue-500 focus:shadow-blue-500/10'}"
            required
          />
          <p class="text-xs mt-1 {isDarkMode ? 'text-slate-500' : 'text-gray-500'}">
            Your company domain
          </p>
        </div>

        <div>
          <label for="jiraAccessToken" class="block mb-1.5 font-medium text-sm uppercase tracking-wide {isDarkMode ? 'text-slate-400' : 'text-gray-600'}">
            JIRA Access Token
          </label>
          
          <input
            id="jiraAccessToken"
            type="password"
            bind:value={loginForm.token}
            placeholder="Your JIRA access token"
            class="w-full p-2.5 border rounded-lg text-sm transition-all focus:outline-none focus:shadow-lg {isDarkMode 
              ? 'border-slate-600 bg-slate-700/80 text-slate-200 placeholder-slate-500 focus:border-blue-500 focus:bg-slate-700 focus:shadow-blue-500/10' 
              : 'border-gray-300 bg-white text-gray-900 placeholder-gray-400 focus:border-blue-500 focus:shadow-blue-500/10'}"
            required
          />
          <p class="text-xs mt-1 {isDarkMode ? 'text-slate-500' : 'text-gray-500'}">
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
      </div>
      
      <!-- Footer - Sticky to bottom -->
      <div class="text-center py-4 border-t mt-auto {isDarkMode ? 'border-slate-700/50' : 'border-gray-200'}">
        <p class="text-xs {isDarkMode ? 'text-slate-500' : 'text-gray-500'}">
          Crafted with <span class="text-orange-800 font-semibold">chaos</span> and <span class="text-[#8B4513] font-semibold">coffee</span> by ariefg ☕
        </p>
      </div>
    {:else}
      <!-- Work Log Section -->
      <div class="space-y-4 max-w-2xl mx-auto">
        <div class="p-2.5 rounded-lg flex justify-between items-center {isDarkMode ? 'bg-blue-500/15' : 'bg-blue-50 border border-blue-200'}">
          <div class="flex flex-col min-h-[40px] justify-center">
            <span class="font-medium text-sm {isDarkMode ? 'text-purple-400' : 'text-purple-600'}">Logged in!</span>
            <div class="min-h-[16px] flex items-start">
              {#if status.visible}
                <span class="text-xs font-normal {
                  status.type === 'success' ? (isDarkMode ? 'text-green-400' : 'text-green-600') :
                  status.type === 'error' ? (isDarkMode ? 'text-red-400' : 'text-red-600') :
                  (isDarkMode ? 'text-blue-400' : 'text-blue-600')
                }">
                  {status.message}
                </span>
              {/if}
            </div>
          </div>
          <button
            onclick={handleLogout}
            class="border px-3 py-1 rounded-2xl cursor-pointer text-xs transition-all {isDarkMode 
              ? 'bg-transparent border-red-400 text-red-400 hover:bg-red-400 hover:text-slate-800' 
              : 'bg-transparent border-red-500 text-red-500 hover:bg-red-500 hover:text-white'}"
          >
            Logout
          </button>
        </div>

        <!-- Background Controls Section (Debug Mode Only) -->
        {#if isDebugMode}
          <div class="p-3 rounded-lg {isDarkMode ? 'bg-slate-700/50' : 'bg-gray-100 border border-gray-200'}">
            <h3 class="font-medium text-sm mb-3 uppercase tracking-wide {isDarkMode ? 'text-slate-300' : 'text-gray-700'}">Background Controls</h3>
            <div class="flex gap-2 flex-wrap">
              <button
                onclick={handleHideToTray}
                class="flex-1 p-2 border-none rounded-lg text-xs font-semibold cursor-pointer hover:-translate-y-0.5 hover:shadow-md transition-all {isDarkMode 
                  ? 'bg-gradient-to-r from-slate-600 to-slate-700 text-white' 
                  : 'bg-gradient-to-r from-gray-500 to-gray-600 text-white'}"
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
            <p class="text-xs mt-2 {isDarkMode ? 'text-slate-400' : 'text-gray-600'}">
              • App runs in background when closed<br>
              • Daily reminder at 5 PM<br>
              • Access via system tray icon
            </p>
          </div>
        {/if}

        <div>

          <label for="issueKey" class="block mb-1.5 font-medium text-sm uppercase tracking-wide {isDarkMode ? 'text-slate-400' : 'text-gray-600'}">
            JIRA Issue
          </label>
          <select
            id="issueKey"
            bind:value={workLogForm.issueKey}
            class="w-full p-2.5 border rounded-lg text-sm cursor-pointer transition-all focus:outline-none focus:shadow-lg {isDarkMode 
              ? 'border-slate-600 bg-slate-700/80 text-slate-200 focus:border-blue-500 focus:bg-slate-700 focus:shadow-blue-500/10' 
              : 'border-gray-300 bg-white text-gray-900 focus:border-blue-500 focus:shadow-blue-500/10'}"
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
          <label for="workDate" class="block mb-1.5 font-medium text-sm uppercase tracking-wide {isDarkMode ? 'text-slate-400' : 'text-gray-600'}">
            Date
          </label>
          <input
            id="workDate"
            type="date"
            bind:value={workLogForm.workDate}
            class="w-full p-2.5 border rounded-lg text-sm transition-all focus:outline-none focus:shadow-lg {isDarkMode 
              ? 'border-slate-600 bg-slate-700/80 text-slate-200 focus:border-blue-500 focus:bg-slate-700 focus:shadow-blue-500/10' 
              : 'border-gray-300 bg-white text-gray-900 focus:border-blue-500 focus:shadow-blue-500/10'}"
            required
          />
        </div>
        
        <div>
          <label for="timeAmount" class="block mb-1.5 font-medium text-sm uppercase tracking-wide {isDarkMode ? 'text-slate-400' : 'text-gray-600'}">
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
                class="w-full p-2.5 border rounded-lg text-sm transition-all focus:outline-none focus:shadow-lg {isDarkMode 
                  ? 'border-slate-600 bg-slate-700/80 text-slate-200 placeholder-slate-500 focus:border-blue-500 focus:bg-slate-700 focus:shadow-blue-500/10' 
                  : 'border-gray-300 bg-white text-gray-900 placeholder-gray-400 focus:border-blue-500 focus:shadow-blue-500/10'}"
                required
              />
            </div>
            <div class="flex-1">
              <select
                bind:value={workLogForm.timeUnit}
                class="w-full p-2.5 border rounded-lg text-sm cursor-pointer transition-all focus:outline-none focus:shadow-lg {isDarkMode 
                  ? 'border-slate-600 bg-slate-700/80 text-slate-200 focus:border-blue-500 focus:bg-slate-700 focus:shadow-blue-500/10' 
                  : 'border-gray-300 bg-white text-gray-900 focus:border-blue-500 focus:shadow-blue-500/10'}"
              >
                <option value="h">Hours</option>
                <option value="m">Minutes</option>
                <option value="d">Days</option>
              </select>
            </div>
          </div>
        </div>
        
        <div>
          <label for="workDescription" class="block mb-1.5 font-medium text-sm uppercase tracking-wide {isDarkMode ? 'text-slate-400' : 'text-gray-600'}">
            Work Description
          </label>
          <textarea
            id="workDescription"
            bind:value={workLogForm.description}
            placeholder="Describe the work you performed..."
            class="w-full p-2.5 border rounded-lg text-sm resize-vertical min-h-[70px] font-inherit transition-all focus:outline-none focus:shadow-lg {isDarkMode 
              ? 'border-slate-600 bg-slate-700/80 text-slate-200 placeholder-slate-500 focus:border-blue-500 focus:bg-slate-700 focus:shadow-blue-500/10' 
              : 'border-gray-300 bg-white text-gray-900 placeholder-gray-400 focus:border-blue-500 focus:shadow-blue-500/10'}"
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
