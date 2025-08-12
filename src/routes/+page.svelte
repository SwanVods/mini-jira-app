<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { 
    isPermissionGranted, 
    requestPermission, 
    sendNotification 
  } from '@tauri-apps/plugin-notification';
  import '../app.css'
  
  // Import components and stores
  import ThemeToggle from '$lib/components/ThemeToggle.svelte';
  import StatusMessage from '$lib/components/StatusMessage.svelte';
  import LoginPage from '$lib/components/LoginPage.svelte';
  import WorkLogPage from '$lib/components/WorkLogPage.svelte';
  import { authStore } from '$lib/stores/auth.js';
  import { themeStore } from '$lib/stores/theme.js';
  import { statusStore } from '$lib/stores/status.js';

  // For development/debugging
  import { invoke } from '@tauri-apps/api/core';

  const isDebugMode = import.meta.env.DEV || import.meta.env.MODE === 'development';

  onMount(() => {
    // Load saved credentials and theme
    authStore.loadSavedCredentials();
    themeStore.init();

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

  async function handleTestNotification() {
    try {
      await sendNotification({
        title: 'JIRA Work Log',
        body: 'Test notification - Background functionality is working!'
      });
      statusStore.show('Test notification sent!', 'success');
    } catch (error) {
      console.error('Error sending notification:', error);
      statusStore.show('Failed to send notification', 'error');
    }
  }

  async function handleLogout() {
    try {
      await invoke('disconnect_from_jira');
    } catch (error) {
      console.warn('Error disconnecting:', error);
    }
    
    authStore.logout();
    statusStore.clear();
  }

  async function handleHideToTray() {
    try {
      await invoke('hide_to_tray');
      statusStore.show('App minimized to system tray', 'success');
    } catch (error) {
      console.error('Error hiding to tray:', error);
    }
  }
</script>

<div class="min-h-screen w-screen overflow-hidden p-0 m-0 {$themeStore ? 'bg-slate-800' : 'bg-gray-50'}">
  <div class="min-h-screen w-full p-6 overflow-y-scroll scrollbar-hide {!$authStore.isLoggedIn ? 'flex flex-col' : ''}">
    
    <!-- Theme Toggle Button -->
    <ThemeToggle />

    <!-- Status Message -->
    <StatusMessage />

    <h1 class="text-center mb-5 font-light text-3xl max-w-4xl mx-auto {$themeStore ? 'text-slate-200' : 'text-gray-800'}">
      Mini Jira Logger
    </h1>

    {#if !$authStore.isLoggedIn}
      <LoginPage />
    {:else}
      <!-- User Info Panel -->
      <div class="max-w-4xl mx-auto mb-6">
        <div class="p-2.5 rounded-lg flex justify-between items-center {$themeStore ? 'bg-blue-500/15' : 'bg-blue-50 border border-blue-200'}">
          <div class="flex flex-col min-h-[40px] justify-center">
            <span class="font-medium text-sm {$themeStore ? 'text-purple-400' : 'text-purple-600'}">{$authStore.credentials.email}</span>
          </div>
          <button
            onclick={handleLogout}
            class="border px-3 py-1 rounded-2xl cursor-pointer text-xs transition-all {$themeStore 
              ? 'bg-transparent border-red-400 text-red-400 hover:bg-red-400 hover:text-slate-800' 
              : 'bg-transparent border-red-500 text-red-500 hover:bg-red-500 hover:text-white'}"
          >
            Logout
          </button>
        </div>

        <!-- Background Controls Section (Debug Mode Only) -->
        {#if isDebugMode}
          <div class="mt-4 p-3 rounded-lg {$themeStore ? 'bg-slate-700/50' : 'bg-gray-100 border border-gray-200'}">
            <h3 class="font-medium text-sm mb-3 uppercase tracking-wide {$themeStore ? 'text-slate-300' : 'text-gray-700'}">Background Controls</h3>
            <div class="flex gap-2 flex-wrap">
              <button
                onclick={handleHideToTray}
                class="flex-1 p-2 border-none rounded-lg text-xs font-semibold cursor-pointer hover:-translate-y-0.5 hover:shadow-md transition-all {$themeStore 
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
            <p class="text-xs mt-2 {$themeStore ? 'text-slate-400' : 'text-gray-600'}">
              • App runs in background when closed<br>
              • Daily reminder at 5 PM<br>
              • Access via system tray icon
            </p>
          </div>
        {/if}
      </div>

      <!-- Work Log Page Component -->
      <WorkLogPage />
    {/if}
  </div>
</div>
