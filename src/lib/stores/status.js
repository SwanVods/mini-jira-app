import { writable } from 'svelte/store';

/**
 * @typedef {Object} Status
 * @property {string} message
 * @property {'success'|'error'|'loading'|''} type
 * @property {boolean} visible
 */

function createStatusStore() {
  const { subscribe, set, update } = writable({
    message: '',
    type: '',
    visible: false
  });

  return {
    subscribe,
    /**
     * @param {string} message
     * @param {'success'|'error'|'loading'} type
     */
    show: (message, type = 'loading') => {
      set({ message, type, visible: true });
      
      if (type === 'success' || type === 'error') {
        setTimeout(() => {
          update(state => ({ ...state, visible: false }));
        }, 5000);
      }
    },
    hide: () => update(state => ({ ...state, visible: false })),
    clear: () => set({ message: '', type: '', visible: false })
  };
}

export const statusStore = createStatusStore();
