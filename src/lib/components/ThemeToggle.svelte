<script>
  import { themeStore } from '$lib/stores/theme.js';
  import { onMount } from 'svelte';

  onMount(() => {
    themeStore.init();
  });
</script>

<div class="absolute top-4 right-4 z-10">
  <button
    onclick={() => themeStore.toggle()}
    class="p-2 rounded-lg border transition-all {$themeStore 
      ? 'bg-slate-700 border-slate-600 text-slate-200 hover:bg-slate-600' 
      : 'bg-white border-gray-300 text-gray-700 hover:bg-gray-50'}"
    title={$themeStore ? 'Switch to Light Mode' : 'Switch to Dark Mode'}
  >
    {#if $themeStore}
      ☀️
    {:else}
      🌙
    {/if}
  </button>
</div>
