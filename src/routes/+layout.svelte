<script lang="ts">
  import '../app.css';
  import { page } from '$app/stores';
  import { connectionStore } from '$lib/stores/connection';
  import { themeStore } from '$lib/stores/theme';
  import Navigation from '$lib/components/Navigation.svelte';
  import { onMount } from 'svelte';
  import { Toaster } from 'svelte-5-french-toast';

  let isConnected = $state(false);
  let currentPath = $state('');

  // Subscribe to connection state
  connectionStore.subscribe(state => {
    isConnected = state.current?.isConnected || false;
  });

  // Subscribe to current path
  page.subscribe(value => {
    currentPath = value.url.pathname;
  });

  // Initialize theme on mount
  onMount(() => {
    themeStore.init();
  });

  // Determine if we should show navigation
  // Hide on landing, access, and onboard pages
  let showNavigation = $derived(
    isConnected &&
    currentPath !== '/' &&
    currentPath !== '/access' &&
    currentPath !== '/onboard-subid' &&
    currentPath !== '/onboard-verusidx'
  );
</script>

{#if showNavigation}
  <Navigation />
{/if}

<main>
  <slot />
</main>

<Toaster 
  toastOptions={{
    className: 'verusidx-toast'
  }}
/>

<style>
  main {
    height: 100vh;
    overflow: hidden;
  }
</style>