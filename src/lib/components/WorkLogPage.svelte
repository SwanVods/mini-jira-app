<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { authStore } from '$lib/stores/auth.js';
  import { themeStore } from '$lib/stores/theme.js';
  import { statusStore } from '$lib/stores/status.js';
  import { workLogStore } from '$lib/stores/worklog.js';

  /**
   * @typedef {Object} Issue
   * @property {string} key
   * @property {Object} fields
   * @property {string} fields.summary
   * @property {Object} fields.status
   * @property {string} fields.status.name
   * @property {Object} [fields.assignee]
   * @property {string} [fields.assignee.display_name]
   * @property {string} [fields.assignee.email_address]
   */

  let assignedIssues = $state([] as any[]);
  let isSubmittingWorkLog = $state(false);

  onMount(() => {
    const today = new Date().toISOString().split('T')[0];
    workLogStore.updateForm({ workDate: today });
    
    // Load issues if logged in
    if ($authStore.isLoggedIn) {
      loadAssignedIssues();
    }
  });

  async function loadAssignedIssues() {
    statusStore.show('Loading assigned issues...', 'loading');
    
    try {
      const issues = await invoke('get_assigned_issues') as any[];
      assignedIssues = issues;
      statusStore.show('Issues loaded successfully!', 'success');
    } catch (error) {
      const errorText = (error && typeof error === 'object' && 'message' in error) 
        ? String(error.message) 
        : String(error);
      statusStore.show('Failed to load issues: ' + errorText, 'error');
    }
  }

  async function handleRefreshIssues() {
    if (!$authStore.isLoggedIn) {
      statusStore.show('Please login first', 'error');
      return;
    }
    await loadAssignedIssues();
  }

  async function handleSubmitWorkLog(event: Event) {
    event.preventDefault();
    
    const form = $workLogStore.form;
    const { issueKey, workDate, timeAmount, timeUnit, description } = form;

    if (!issueKey || !workDate || !timeAmount || !description) {
      statusStore.show('Please fill in all required fields', 'error');
      return;
    }

    const timeSpent = timeAmount + timeUnit;
    isSubmittingWorkLog = true;
    statusStore.show('Submitting work log...', 'loading');

    try {
      const dateObj = new Date(workDate + 'T09:00:00.000');
      const startedDateTime = dateObj.toISOString().replace('Z', '+0000');
      
      await invoke('create_worklog', {
        issueKey: issueKey,
        description: description,
        started: startedDateTime,
        timeSpent: timeSpent
      });
      
      statusStore.show('Work log submitted successfully!', 'success');
      
      // Reset form
      workLogStore.updateForm({
        timeAmount: '',
        description: ''
      });
      
    } catch (error) {
      const errorText = (error && typeof error === 'object' && 'message' in error) 
        ? String(error.message) 
        : String(error);
      statusStore.show('Failed to submit work log: ' + errorText, 'error');
    } finally {
      isSubmittingWorkLog = false;
    }
  }
</script>

<div class="space-y-4 max-w-2xl mx-auto">
  <div>
    <label for="issueKey" class="block mb-1.5 font-medium text-sm uppercase tracking-wide {$themeStore ? 'text-slate-400' : 'text-gray-600'}">
      JIRA Issue
    </label>
    <select
      id="issueKey"
      bind:value={$workLogStore.form.issueKey}
      class="w-full p-2.5 border rounded-lg text-sm cursor-pointer transition-all focus:outline-none focus:shadow-lg {$themeStore 
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
      type="button"
      onclick={handleRefreshIssues}
      class="w-full p-2 bg-gradient-to-r from-orange-500 to-red-600 text-white border-none rounded-lg text-xs font-semibold cursor-pointer mt-2 hover:-translate-y-0.5 hover:shadow-md transition-all"
    >
      Refresh Issues
    </button>
  </div>
  
  <div>
    <label for="workDate" class="block mb-1.5 font-medium text-sm uppercase tracking-wide {$themeStore ? 'text-slate-400' : 'text-gray-600'}">
      Date
    </label>
    <input
      id="workDate"
      type="date"
      bind:value={$workLogStore.form.workDate}
      class="w-full p-2.5 border rounded-lg text-sm transition-all focus:outline-none focus:shadow-lg {$themeStore 
        ? 'border-slate-600 bg-slate-700/80 text-slate-200 focus:border-blue-500 focus:bg-slate-700 focus:shadow-blue-500/10' 
        : 'border-gray-300 bg-white text-gray-900 focus:border-blue-500 focus:shadow-blue-500/10'}"
      required
    />
  </div>
  
  <div>
    <label for="timeAmount" class="block mb-1.5 font-medium text-sm uppercase tracking-wide {$themeStore ? 'text-slate-400' : 'text-gray-600'}">
      Time Spent
    </label>
    <div class="flex gap-2">
      <div class="flex-[2]">
        <input
          id="timeAmount"
          type="number"
          bind:value={$workLogStore.form.timeAmount}
          placeholder="1"
          min="0.25"
          step="0.25"
          class="w-full p-2.5 border rounded-lg text-sm transition-all focus:outline-none focus:shadow-lg {$themeStore 
            ? 'border-slate-600 bg-slate-700/80 text-slate-200 placeholder-slate-500 focus:border-blue-500 focus:bg-slate-700 focus:shadow-blue-500/10' 
            : 'border-gray-300 bg-white text-gray-900 placeholder-gray-400 focus:border-blue-500 focus:shadow-blue-500/10'}"
          required
        />
      </div>
      <div class="flex-1">
        <select
          bind:value={$workLogStore.form.timeUnit}
          class="w-full p-2.5 border rounded-lg text-sm cursor-pointer transition-all focus:outline-none focus:shadow-lg {$themeStore 
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
    <label for="workDescription" class="block mb-1.5 font-medium text-sm uppercase tracking-wide {$themeStore ? 'text-slate-400' : 'text-gray-600'}">
      Work Description
    </label>
    <textarea
      id="workDescription"
      bind:value={$workLogStore.form.description}
      placeholder="Describe the work you performed..."
      class="w-full p-2.5 border rounded-lg text-sm resize-vertical min-h-[70px] font-inherit transition-all focus:outline-none focus:shadow-lg {$themeStore 
        ? 'border-slate-600 bg-slate-700/80 text-slate-200 placeholder-slate-500 focus:border-blue-500 focus:bg-slate-700 focus:shadow-blue-500/10' 
        : 'border-gray-300 bg-white text-gray-900 placeholder-gray-400 focus:border-blue-500 focus:shadow-blue-500/10'}"
      required
    ></textarea>
  </div>
  
  <button
    onclick={handleSubmitWorkLog}
    disabled={isSubmittingWorkLog}
    class="w-full p-2.5 bg-gradient-to-r from-orange-500 to-red-600 text-white border-none rounded-lg text-sm font-semibold cursor-pointer uppercase tracking-wide mt-2 hover:-translate-y-0.5 hover:shadow-lg hover:shadow-orange-500/40 active:translate-y-0 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
  >
    {isSubmittingWorkLog ? 'Logging Work...' : 'Log Work'}
  </button>
</div>
