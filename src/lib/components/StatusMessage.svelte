<script>
  import { statusStore } from '$lib/stores/status.js';
  import { themeStore } from '$lib/stores/theme.js';
</script>

{#if $statusStore.visible}
  <div class="w-full p-3 rounded-lg text-center font-medium text-sm transition-all {
    $statusStore.type === 'success' ? ($themeStore ? 'bg-green-500/20 text-green-400 border border-green-500/30' : 'bg-green-100 text-green-700 border border-green-300') :
    $statusStore.type === 'error' ? ($themeStore ? 'bg-red-500/20 text-red-400 border border-red-500/30' : 'bg-red-100 text-red-700 border border-red-300') :
    ($themeStore ? 'bg-blue-500/20 text-blue-400 border border-blue-500/30' : 'bg-blue-100 text-blue-700 border border-blue-300')
  }">
    {#if $statusStore.type === 'loading'}
      <div class="flex items-center justify-center gap-2">
        <div class="w-4 h-4 border-2 border-t-transparent rounded-full animate-spin {$themeStore ? 'border-blue-400' : 'border-blue-600'}"></div>
        {$statusStore.message}
      </div>
    {:else}
      {$statusStore.message}
    {/if}
  </div>
{/if}
