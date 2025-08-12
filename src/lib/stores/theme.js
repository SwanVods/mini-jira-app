import { writable } from 'svelte/store';
import { browser } from '$app/environment';

function createThemeStore() {
  const { subscribe, set, update } = writable(true);

  return {
    subscribe,
    toggle: () => update(isDark => {
      const newTheme = !isDark;
      if (browser) {
        localStorage.setItem('theme', newTheme ? 'dark' : 'light');
      }
      return newTheme;
    }),
    init: () => {
      if (browser) {
        const savedTheme = localStorage.getItem('theme');
        if (savedTheme) {
          set(savedTheme === 'dark');
        }
      }
    }
  };
}

export const themeStore = createThemeStore();
