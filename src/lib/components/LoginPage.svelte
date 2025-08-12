<script>
  import { invoke } from '@tauri-apps/api/core';
  import { authStore } from '$lib/stores/auth.js';
  import { statusStore } from '$lib/stores/status.js';
  import { themeStore } from '$lib/stores/theme.js';
  import { workLogStore } from '$lib/stores/worklog.js';
  import StatusMessage from './StatusMessage.svelte';

  async function handleLogin() {
    const { baseUrl, email, token } = $authStore.credentials;

    if (!baseUrl || !email || !token) {
      statusStore.show('Please fill in all fields', 'error');
      return;
    }

    statusStore.show('Connecting to JIRA...', 'loading');

    try {
      const isConnected = await invoke('connect_to_jira', {
        baseUrl: baseUrl.replace(/\/$/, ''),
        email: email,
        accessToken: token
      });

      if (isConnected) {
        authStore.saveCredentials({ baseUrl, email, token });
        await loadAssignedIssues();
        authStore.login({ baseUrl, email, token });
        statusStore.show('Successfully connected to JIRA!', 'success');
      } else {
        statusStore.show('Authentication failed. Please check your credentials.', 'error');
      }
    } catch (error) {
      let errorMessage = 'Connection failed: ';
      const errorText = /** @type {any} */ (error)?.message || /** @type {any} */ (error)?.toString() || '';
      
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
      
      statusStore.show(errorMessage, 'error');
    }
  }

  async function loadAssignedIssues() {
    statusStore.show('Loading assigned issues...', 'loading');
    
    try {
      const issues = await invoke('get_assigned_issues');
      workLogStore.setAssignedIssues(issues);
      statusStore.show('Issues loaded successfully!', 'success');
    } catch (error) {
      const errorText = /** @type {any} */ (error)?.message || /** @type {any} */ (error)?.toString() || 'Unknown error';
      statusStore.show('Failed to load issues: ' + errorText, 'error');
    }
  }
</script>

<div class="flex-1 flex flex-col justify-center">
  <div class="space-y-4 max-w-2xl mx-auto w-full">
    <div class="min-h-[60px] flex items-center">
      <StatusMessage />
    </div>
    
    <div>
      <label for="jiraBaseUrl" class="block mb-1.5 font-medium text-sm uppercase tracking-wide {$themeStore ? 'text-slate-400' : 'text-gray-600'}">
        JIRA Base URL
      </label>
      
      <input
        id="jiraBaseUrl"
        type="url"
        bind:value={$authStore.credentials.baseUrl}
        oninput={(e) => authStore.updateCredentials({ baseUrl: /** @type {HTMLInputElement} */ (e.target).value })}
        placeholder="https://yourcompany.domain.net"
        class="w-full p-2.5 border rounded-lg text-sm transition-all focus:outline-none focus:shadow-lg {$themeStore 
          ? 'border-slate-600 bg-slate-700/80 text-slate-200 placeholder-slate-500 focus:border-blue-500 focus:bg-slate-700 focus:shadow-blue-500/10' 
          : 'border-gray-300 bg-white text-gray-900 placeholder-gray-400 focus:border-blue-500 focus:shadow-blue-500/10'}"
        required
      />
      <p class="text-xs mt-1 {$themeStore ? 'text-slate-500' : 'text-gray-500'}">
        Your company domain
      </p>
    </div>

    <div>
      <label for="jiraEmail" class="block mb-1.5 font-medium text-sm uppercase tracking-wide {$themeStore ? 'text-slate-400' : 'text-gray-600'}">
        JIRA Email
      </label>
      
      <input
        id="jiraEmail"
        type="email"
        bind:value={$authStore.credentials.email}
        oninput={(e) => authStore.updateCredentials({ email: /** @type {HTMLInputElement} */ (e.target).value })}
        placeholder="your.email@company.com"
        class="w-full p-2.5 border rounded-lg text-sm transition-all focus:outline-none focus:shadow-lg {$themeStore 
          ? 'border-slate-600 bg-slate-700/80 text-slate-200 placeholder-slate-500 focus:border-blue-500 focus:bg-slate-700 focus:shadow-blue-500/10' 
          : 'border-gray-300 bg-white text-gray-900 placeholder-gray-400 focus:border-blue-500 focus:shadow-blue-500/10'}"
        required
      />
    </div>
    
    <div>
      <label for="jiraAccessToken" class="block mb-1.5 font-medium text-sm uppercase tracking-wide {$themeStore ? 'text-slate-400' : 'text-gray-600'}">
        JIRA Access Token
      </label>
      
      <input
        id="jiraAccessToken"
        type="password"
        bind:value={$authStore.credentials.token}
        oninput={(e) => authStore.updateCredentials({ token: /** @type {HTMLInputElement} */ (e.target).value })}
        placeholder="Your JIRA access token"
        class="w-full p-2.5 border rounded-lg text-sm transition-all focus:outline-none focus:shadow-lg {$themeStore 
          ? 'border-slate-600 bg-slate-700/80 text-slate-200 placeholder-slate-500 focus:border-blue-500 focus:bg-slate-700 focus:shadow-blue-500/10' 
          : 'border-gray-300 bg-white text-gray-900 placeholder-gray-400 focus:border-blue-500 focus:shadow-blue-500/10'}"
        required
      />
      <p class="text-xs mt-1 {$themeStore ? 'text-slate-500' : 'text-gray-500'}">
        Generate an API token in your JIRA account settings → Security → API tokens.
      </p>
    </div>
    
    <button
      onclick={handleLogin}
      disabled={$statusStore.visible && $statusStore.type === 'loading'}
      class="w-full p-2.5 bg-gradient-to-r from-orange-500 to-red-600 text-white border-none rounded-lg text-sm font-semibold cursor-pointer uppercase tracking-wide mt-2 hover:-translate-y-0.5 hover:shadow-lg hover:shadow-orange-500/40 active:translate-y-0 transition-all disabled:opacity-50 disabled:cursor-not-allowed disabled:transform-none"
    >
      {#if $statusStore.visible && $statusStore.type === 'loading'}
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
<div class="text-center py-4 border-t mt-auto {$themeStore ? 'border-slate-700/50' : 'border-gray-200'}">
  <p class="text-xs {$themeStore ? 'text-slate-500' : 'text-gray-500'}">
    Crafted with <span class="text-orange-800 font-semibold">chaos</span> and <span class="text-[#8B4513] font-semibold">coffee</span> by ariefg ☕
  </p>
</div>
