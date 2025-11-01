<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { connectionStore, getChainParam, type ConnectionState } from "$lib/stores/connection";
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";

  let isLoading = $state(true);
  let identities = $state<any[]>([]);
  let hasValidSubIdentity = $state(false);
  let validSubIdentities = $state<any[]>([]);
  let connectionState = $state<ConnectionState>({ current: null, isConnecting: false, lastError: null, availableChains: [], selectedChain: null });

  // Testing configuration
  // const REQUIRED_PARENT = "iH9HFQeKRNVWguokGLLaiVYqy9u8VuFWMe";
  // Production configuration
  const REQUIRED_PARENT = "i8Rar9Z64hDXtpBRJBhK9b6S6dMV1EttiS";

  connectionStore.subscribe(state => {
    connectionState = state;
    if (!state.current?.isConnected) {
      goto("/");
    }
  });

  async function checkAccess() {
    if (!connectionState.current?.isConnected || !connectionState.selectedChain) {
      console.log("Access page: Not ready to check access - connection:", !!connectionState.current?.isConnected, "selectedChain:", connectionState.selectedChain);
      return;
    }
    
    isLoading = true;
    try {
      // Get identities for the current chain
      const chainParam = getChainParam(connectionState.selectedChain);
      console.log("Access page loading identities for chain:", connectionState.selectedChain, "param:", chainParam);
      
      const identityList = await invoke("list_identities", { chain: chainParam });
      identities = Array.isArray(identityList) ? identityList : [];
      
      // Filter for valid agents sub-identities
      validSubIdentities = identities.filter(identity => {
        const parent = identity.identity?.parent || "";
        const primaryAddresses = identity.identity?.primaryaddresses || [];
        
        // Must have correct parent AND exactly 1 primary address (prevents shared access)
        return parent === REQUIRED_PARENT && primaryAddresses.length === 1;
      });
      
      hasValidSubIdentity = validSubIdentities.length > 0;
      
      // If user has valid sub-identity, redirect to dashboard
      if (hasValidSubIdentity) {
        goto("/dashboard");
      }
    } catch (error) {
      console.error("Failed to check access:", error);
    } finally {
      isLoading = false;
    }
  }

  // Check access when connection and chain are ready
  $effect(() => {
    if (connectionState.current?.isConnected && connectionState.selectedChain) {
      checkAccess();
    }
  });

  onMount(() => {
    checkAccess();
  });
</script>

<div class="bg-verusidx-sky-soft dark:bg-verusidx-lake-deep min-h-screen max-h-screen overflow-y-auto">
  <div class="flex items-center justify-center min-h-screen p-8">
    <div class="max-w-2xl w-full">
    {#if isLoading}
      <div class="text-center">
        <div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-verusidx-mountain-blue"></div>
        <p class="mt-4 text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Checking marketplace access...</p>
      </div>
    {:else if !hasValidSubIdentity}
      <div class="bg-white dark:bg-verusidx-stone-dark rounded-2xl shadow-xl p-8">
        <div class="text-center mb-8">
          <div class="w-16 h-16 bg-verusidx-turquoise-bright/20 dark:bg-verusidx-lake-deep/50 rounded-full flex items-center justify-center mx-auto mb-4">
            <svg class="w-8 h-8 text-verusidx-turquoise-deep dark:text-verusidx-turquoise-light" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd"/>
            </svg>
          </div>
          <h1 class="text-3xl font-bold text-verusidx-stone-dark dark:text-white mb-2">Access Required</h1>
          <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
            You need a VerusIDX sub-identity to access the app on {connectionState?.chainName || 'this chain'}.
          </p>
        </div>

        <div class="bg-verusidx-sky-soft dark:bg-verusidx-lake-deep/20 rounded-lg p-6 mb-8">
          <h3 class="font-semibold text-verusidx-stone-dark dark:text-white mb-3">What is a VerusIDX Sub-Identity?</h3>
          <ul class="space-y-2 text-sm text-verusidx-stone-medium dark:text-verusidx-mountain-mist">
            <li class="flex items-start">
              <span class="text-verusidx-turquoise-deep dark:text-verusidx-turquoise-light mr-2">•</span>
              <span>A namespace under the VerusIDX parent identity</span>
            </li>
            <li class="flex items-start">
              <span class="text-verusidx-turquoise-deep dark:text-verusidx-turquoise-light mr-2">•</span>
              <span>All the same functionality as a root VerusID (except full currency & chain definitions)</span>
            </li>
            <li class="flex items-start">
              <span class="text-verusidx-turquoise-deep dark:text-verusidx-turquoise-light mr-2">•</span>
              <span>Provides identity-based access control to this app</span>
            </li>
          </ul>
        </div>

        <div class="space-y-3">
          <a 
            href="/onboard-subid" 
            class="block w-full px-6 py-3 bg-verusidx-mountain-blue dark:bg-verusidx-turquoise-deep text-white text-center rounded-lg hover:bg-verusidx-lake-blue dark:hover:bg-verusidx-turquoise-bright transition-colors"
          >
            Get VerusIDX SubID
          </a>
          
          <button 
            onclick={() => checkAccess()}
            class="block w-full px-6 py-3 bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-stone-medium dark:text-verusidx-mountain-mist text-center rounded-lg hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-dark transition-colors"
          >
            Check Again
          </button>
        </div>

        {#if identities.length > 0}
          <div class="mt-8 pt-8 border-t border-gray-200 dark:border-verusidx-stone-medium">
            <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-2">
              You have {identities.length} identit{identities.length === 1 ? 'y' : 'ies'}, but none are valid VerusIDX sub-identities.
            </p>
            <details class="text-sm">
              <summary class="cursor-pointer text-verusidx-turquoise-deep dark:text-verusidx-turquoise-light hover:underline">
                View your identities
              </summary>
              <ul class="mt-2 space-y-1 text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
                {#each identities.slice(0, 5) as identity}
                  <li>{identity.name || identity.identity?.name || 'Unknown'}</li>
                {/each}
                {#if identities.length > 5}
                  <li>...and {identities.length - 5} more</li>
                {/if}
              </ul>
            </details>
          </div>
        {/if}
      </div>
    {/if}
    </div>
  </div>
</div>