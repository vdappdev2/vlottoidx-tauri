<script lang="ts">
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { connectionStore } from '$lib/stores/connection';
  import ThemeToggle from './ThemeToggle.svelte';
  import { onMount } from 'svelte';

  // Get current route
  let currentPath = $state('');
  let isMenuOpen = $state(false);
  
  page.subscribe(value => {
    currentPath = value.url.pathname;
  });

  // Navigation items
  const navItems = [
    { path: '/', label: 'Home' },
    { path: '/dashboard', label: 'Dashboard' },
    { path: '/vlotto', label: 'vLotto' },
    { path: '/identities', label: 'Identities' },
    { path: '/currencies', label: 'Currencies' },
    { path: '/charts', label: 'Charts' },
    { path: '/defi', label: 'DeFi Operations' },
    { path: '/marketplace', label: 'Marketplace' },
    { path: '/wallet', label: 'Wallet' }
  ];

  function toggleMenu() {
    isMenuOpen = !isMenuOpen;
  }

  function closeMenu() {
    isMenuOpen = false;
  }

  function navigate(path: string) {
    // Handle Home navigation specially
    if (path === '/') {
      handleHomeNavigation();
    } else {
      goto(path);
      closeMenu(); // Close menu after navigation
    }
  }

  function handleHomeNavigation() {
    // Clear connection state and navigate to home (chain discovery page)
    connectionStore.update(state => ({ ...state, current: null }));
    goto("/");
    closeMenu();
  }

  // Close menu when clicking outside
  function handleBackdropClick() {
    closeMenu();
  }

  // Handle escape key
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && isMenuOpen) {
      closeMenu();
    }
  }

  onMount(() => {
    // Add global keydown listener
    document.addEventListener('keydown', handleKeydown);
    
    return () => {
      document.removeEventListener('keydown', handleKeydown);
    };
  });
</script>

<!-- Hamburger Menu Button -->
<button
  onclick={toggleMenu}
  class="fixed top-4 left-4 z-50 p-3 rounded-lg bg-verusidx-mountain-blue hover:bg-verusidx-lake-blue text-white shadow-lg transition-all duration-200"
  aria-label="Toggle navigation menu"
>
  <svg 
    class="w-6 h-6 transition-transform duration-200 {isMenuOpen ? 'rotate-90' : ''}" 
    fill="none" 
    stroke="currentColor" 
    viewBox="0 0 24 24"
  >
    {#if isMenuOpen}
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
    {:else}
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"/>
    {/if}
  </svg>
</button>

<!-- Backdrop Overlay -->
{#if isMenuOpen}
  <div 
    class="fixed inset-0 bg-black/50 z-40 transition-opacity duration-200"
    onclick={handleBackdropClick}
  ></div>
{/if}

<!-- Slide-out Menu Panel -->
<div 
  class="fixed top-0 left-0 h-full w-80 bg-white dark:bg-verusidx-stone-dark shadow-xl z-50 transform transition-transform duration-300 ease-in-out {isMenuOpen ? 'translate-x-0' : '-translate-x-full'}"
>
  <div class="flex flex-col h-full">
    <!-- Menu Header -->
    <div class="p-6 border-b border-gray-200 dark:border-verusidx-stone-medium">
      <h2 class="text-xl font-bold text-verusidx-stone-dark dark:text-white">VerusIDX</h2>
      <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">Navigation Menu</p>
    </div>

    <!-- Navigation Items -->
    <nav class="flex-1 overflow-y-auto">
      <div class="py-4">
        {#each navItems as item}
          <button
            onclick={() => navigate(item.path)}
            class="w-full px-6 py-3 text-left hover:bg-verusidx-snow-ice dark:hover:bg-verusidx-stone-medium transition-colors {currentPath === item.path ? 'bg-verusidx-turquoise-light/10 dark:bg-verusidx-turquoise-deep/20 border-r-4 border-verusidx-turquoise-deep dark:border-verusidx-turquoise-light' : ''}"
          >
            <span class="text-verusidx-stone-dark dark:text-white font-medium {currentPath === item.path ? 'text-verusidx-turquoise-deep dark:text-verusidx-turquoise-light' : ''}">{item.label}</span>
          </button>
        {/each}
      </div>
    </nav>

    <!-- Menu Footer with Theme Toggle -->
    <div class="p-6 border-t border-gray-200 dark:border-verusidx-stone-medium">
      <div class="flex items-center justify-between">
        <span class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Theme</span>
        <ThemeToggle />
      </div>
    </div>
  </div>
</div>