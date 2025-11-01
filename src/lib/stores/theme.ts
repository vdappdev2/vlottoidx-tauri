import { writable } from 'svelte/store';

// Check if we're in a browser environment
const isBrowser = typeof window !== 'undefined';

// Initialize theme from localStorage or system preference
function getInitialTheme(): boolean {
  if (!isBrowser) return false;
  
  const savedTheme = localStorage.getItem('theme');
  if (savedTheme) {
    return savedTheme === 'dark';
  }
  
  // Check system preference
  return window.matchMedia('(prefers-color-scheme: dark)').matches;
}

// Create the theme store
function createThemeStore() {
  const { subscribe, set, update } = writable(getInitialTheme());

  return {
    subscribe,
    toggle: () => update(isDark => {
      const newTheme = !isDark;
      
      if (isBrowser) {
        // Update localStorage
        localStorage.setItem('theme', newTheme ? 'dark' : 'light');
        
        // Update document class
        if (newTheme) {
          document.documentElement.classList.add('dark');
        } else {
          document.documentElement.classList.remove('dark');
        }
      }
      
      return newTheme;
    }),
    set: (isDark: boolean) => {
      if (isBrowser) {
        localStorage.setItem('theme', isDark ? 'dark' : 'light');
        
        if (isDark) {
          document.documentElement.classList.add('dark');
        } else {
          document.documentElement.classList.remove('dark');
        }
      }
      
      set(isDark);
    },
    init: () => {
      if (isBrowser) {
        const isDark = getInitialTheme();
        if (isDark) {
          document.documentElement.classList.add('dark');
        } else {
          document.documentElement.classList.remove('dark');
        }
        set(isDark);
      }
    }
  };
}

export const themeStore = createThemeStore();