<script lang="ts">
  import { themeStore } from '$lib/stores/theme';

  let isDarkMode = $state(false);

  // Props for different contexts
  let { variant = 'menu' }: { variant?: 'menu' | 'floating' } = $props();

  // Subscribe to theme store
  themeStore.subscribe(value => {
    isDarkMode = value;
  });

  function toggleTheme() {
    themeStore.toggle();
  }

  // Dynamic classes based on variant and theme
  let buttonClasses = $derived(variant === 'floating'
    ? "p-3 rounded-lg bg-verusidx-mountain-blue dark:bg-verusidx-turquoise-deep hover:bg-verusidx-lake-blue dark:hover:bg-verusidx-turquoise-bright text-white shadow-lg transition-all duration-200"
    : "p-2 rounded-lg bg-verusidx-snow-ice dark:bg-verusidx-stone-medium hover:bg-verusidx-mountain-mist dark:hover:bg-verusidx-stone-dark transition-colors");

  let iconClasses = $derived(variant === 'floating'
    ? "w-6 h-6 text-white"
    : isDarkMode 
      ? "w-5 h-5 text-verusidx-turquoise-bright" 
      : "w-5 h-5 text-verusidx-stone-medium");
</script>

<button
  onclick={toggleTheme}
  class={buttonClasses}
  title="{isDarkMode ? 'Switch to Light Mode' : 'Switch to Dark Mode'}"
>
  {#if isDarkMode}
    <svg class={iconClasses} fill="currentColor" viewBox="0 0 20 20">
      <path fill-rule="evenodd" d="M10 2a1 1 0 011 1v1a1 1 0 11-2 0V3a1 1 0 011-1zm4 8a4 4 0 11-8 0 4 4 0 018 0zm-.464 4.95l.707.707a1 1 0 001.414-1.414l-.707-.707a1 1 0 00-1.414 1.414zm2.12-10.607a1 1 0 010 1.414l-.706.707a1 1 0 11-1.414-1.414l.707-.707a1 1 0 011.414 0zM17 11a1 1 0 100-2h-1a1 1 0 100 2h1zm-7 4a1 1 0 011 1v1a1 1 0 11-2 0v-1a1 1 0 011-1zM5.05 6.464A1 1 0 106.465 5.05l-.708-.707a1 1 0 00-1.414 1.414l.707.707zm1.414 8.486l-.707.707a1 1 0 01-1.414-1.414l.707-.707a1 1 0 011.414 1.414zM4 11a1 1 0 100-2H3a1 1 0 000 2h1z" clip-rule="evenodd"/>
    </svg>
  {:else}
    <svg class={iconClasses} fill="currentColor" viewBox="0 0 20 20">
      <path d="M17.293 13.293A8 8 0 016.707 2.707a8.001 8.001 0 1010.586 10.586z"/>
    </svg>
  {/if}
</button>