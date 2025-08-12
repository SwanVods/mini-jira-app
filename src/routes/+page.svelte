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

    <h1 class="text-center mb-5 font-light text-3xl max-w-4xl mx-auto {$themeStore ? 'text-slate-200' : 'text-gray-800'}">
      Mini Jira Logger
    </h1>

    {#if !$authStore.isLoggedIn}
      <LoginPage />
    {:else}
      <!-- User Info Panel -->
      <div class="max-w-4xl mx-auto mb-6">
        <div class="p-3 rounded-lg flex justify-between items-center {$themeStore ? 'bg-blue-500/15' : 'bg-blue-50 border border-blue-200'}">
          <div class="flex items-center gap-3">
            <span class="font-medium {$themeStore ? 'text-purple-400' : 'text-purple-600'}">{$authStore.credentials.email}</span>
          </div>
          <div class="flex items-center gap-2">
            <!-- Debug Controls (Development Only) -->
            {#if isDebugMode}
              <button
                onclick={handleHideToTray}
                class="px-3 py-1.5 text-xs font-semibold rounded-lg transition-all hover:-translate-y-0.5 {$themeStore 
                  ? 'bg-slate-600 text-white hover:bg-slate-500' 
                  : 'bg-gray-500 text-white hover:bg-gray-400'}"
              >
                Hide to Tray
              </button>
              <button
                onclick={handleTestNotification}
                class="px-3 py-1.5 bg-green-600 text-white text-xs font-semibold rounded-lg transition-all hover:-translate-y-0.5 hover:bg-green-700"
              >
                Test Notification
              </button>
            {/if}
            <button
              onclick={handleLogout}
              class="px-3 py-1.5 border rounded-lg text-xs font-semibold transition-all hover:-translate-y-0.5 {$themeStore 
                ? 'border-red-400 text-red-400 hover:bg-red-400 hover:text-slate-800' 
                : 'border-red-500 text-red-500 hover:bg-red-500 hover:text-white'}"
            >
              Logout
            </button>
          </div>
        </div>

        {#if isDebugMode}
          <div class="mt-2 p-2 rounded text-xs {$themeStore ? 'bg-slate-700/30 text-slate-400' : 'bg-gray-100 text-gray-600'}">
            <strong>Debug Mode:</strong> App runs in background • Daily reminder at 5 PM • Access via system tray icon
          </div>
        {/if}
      </div>

      <!-- Work Log Page Component -->
      <WorkLogPage />
    {/if}
  </div>
</div>
