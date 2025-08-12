import { writable } from 'svelte/store';
import { browser } from '$app/environment';

/**
 * @typedef {Object} Credentials
 * @property {string} baseUrl
 * @property {string} email  
 * @property {string} token
 */

function createAuthStore() {
  const { subscribe, set, update } = writable({
    isLoggedIn: false,
    credentials: {
      baseUrl: '',
      email: '',
      token: ''
    }
  });

  return {
    subscribe,
    /**
     * @param {Credentials} credentials
     */
    login: (credentials) => update(state => ({
      ...state,
      isLoggedIn: true,
      credentials
    })),
    logout: () => update(state => ({
      ...state,
      isLoggedIn: false,
      credentials: {
        baseUrl: '',
        email: '',
        token: ''
      }
    })),
    /**
     * @param {Partial<Credentials>} credentials
     */
    updateCredentials: (credentials) => update(state => ({
      ...state,
      credentials: { ...state.credentials, ...credentials }
    })),
    loadSavedCredentials: () => {
      if (browser) {
        const saved = JSON.parse(localStorage.getItem('jiraCredentials') || '{}');
        update(state => ({
          ...state,
          credentials: {
            baseUrl: saved.baseUrl || '',
            email: saved.email || '',
            token: saved.token || ''
          }
        }));
      }
    },
    /**
     * @param {Credentials} credentials
     */
    saveCredentials: (credentials) => {
      if (browser) {
        localStorage.setItem('jiraCredentials', JSON.stringify(credentials));
      }
    }
  };
}

export const authStore = createAuthStore();
