<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { connectionStore, getChainParam, type ChainConfig, type ConnectionState } from "$lib/stores/connection";
  import { onDestroy } from "svelte";

  let blockHeight = $state<number | null>(null);
  let isLoading = $state(true);
  let connectionState = $state<ConnectionState>({ current: null, isConnecting: false, lastError: null, availableChains: [], selectedChain: null });
  let refreshInterval: ReturnType<typeof setInterval> | undefined;
  let isLoadingChains = $state(false);
  let hasLoadedChains = $state(false);
  let lastConnectedChain = $state<string | null>(null);

  connectionStore.subscribe(state => {
    connectionState = state;
  });

  async function fetchBlockHeight() {
    if (!connectionState?.current?.isConnected) return;

    try {
      const chainParam = getChainParam(connectionState.selectedChain);
      const height = await invoke("get_block_count", { chain: chainParam });
      blockHeight = height as number;
    } catch (error) {
      console.error("Failed to fetch block height:", error);
    } finally {
      isLoading = false;
    }
  }

  async function loadAvailableChains() {
    if (isLoadingChains || hasLoadedChains) return;
    
    console.log('🚀 BlockHeightHeader: Loading available chains...');
    isLoadingChains = true;
    try {
      // Get discovered chains
      const discoveredChains = await invoke("get_discovered_chains") as ChainConfig[];
      
      // Filter for only active chains
      const activeChains = discoveredChains.filter(chain => chain.is_active);
      
      console.log('📝 BlockHeightHeader: Found active chains:', activeChains.map(c => c.name));
      
      // Update store with available chains
      connectionStore.update(state => ({
        ...state,
        availableChains: activeChains
      }));
      
      hasLoadedChains = true;
      
    } catch (error) {
      console.error("Failed to load available chains:", error);
    } finally {
      isLoadingChains = false;
    }
  }

  async function handleChainSwitch(chainName: string) {
    if (chainName === connectionState.selectedChain) return;
    
    console.log('🔀 BlockHeightHeader: Switching chain from', connectionState.selectedChain, 'to', chainName);
    
    // Store previous state for rollback on failure
    const previousChain = connectionState.selectedChain;
    const previousConnection = connectionState.current;
    
    try {
      // Update selected chain in store first
      connectionStore.update(state => ({
        ...state,
        selectedChain: chainName,
        isConnecting: true
      }));
      
      // Reset block height to trigger loading state
      isLoading = true;
      blockHeight = null;
      
      // Actually connect to the new chain's daemon
      console.log('🔌 BlockHeightHeader: Connecting to new daemon for chain:', chainName);
      const connectionResult = await invoke("connect_to_chain", {
        chainName: chainName
      });
      
      if (connectionResult) {
        // Find the chain config to get the new connection details
        const targetChain = connectionState.availableChains.find(c => c.name === chainName);
        
        if (targetChain) {
          // Update connection state with new daemon credentials
          const newConnection = {
            host: targetChain.credentials.host,
            port: targetChain.credentials.port,
            username: targetChain.credentials.username,
            password: targetChain.credentials.password,
            isConnected: true,
            chainName: chainName
          };
          
          connectionStore.update(state => ({
            ...state,
            current: newConnection,
            selectedChain: chainName,
            isConnecting: false,
            lastError: null
          }));
          
          console.log('✅ BlockHeightHeader: Successfully connected to', chainName, 'daemon');
          
          // Fetch new block height for the newly connected chain
          await fetchBlockHeight();
          
          // Emit a custom event that pages can listen to for data refresh
          console.log('📡 BlockHeightHeader: Emitting chainChanged event for:', chainName);
          window.dispatchEvent(new CustomEvent('chainChanged', { 
            detail: { chainName, chainParam: getChainParam(chainName) }
          }));
        } else {
          throw new Error(`Chain config not found for ${chainName}`);
        }
      } else {
        throw new Error(`Failed to connect to ${chainName} daemon`);
      }
      
    } catch (error) {
      console.error("Failed to switch chain:", error);
      
      // Rollback to previous state on failure
      connectionStore.update(state => ({
        ...state,
        selectedChain: previousChain,
        current: previousConnection,
        isConnecting: false,
        lastError: `Failed to switch to ${chainName}: ${error}`
      }));
      
      // Reset block height loading state
      isLoading = false;
      
      // Re-fetch block height for the reverted chain
      await fetchBlockHeight();
    }
  }

  // Set up refresh interval
  function setupRefresh() {
    // Initial fetch
    fetchBlockHeight();
    
    // Set up 30-second interval
    refreshInterval = setInterval(() => {
      fetchBlockHeight();
    }, 30000); // 30 seconds
  }

  // Start refresh when connected and load available chains
  $effect(() => {
    if (connectionState?.current?.isConnected) {
      const currentChain = connectionState.current.chainName;
      
      // Only run setup logic when we first connect or chain changes
      if (lastConnectedChain !== currentChain) {
        console.log('🔄 BlockHeightHeader: Connected to chain:', currentChain);
        lastConnectedChain = currentChain;
        
        // Remove competing initialization - let landing page be source of truth
        // selectedChain should already be set by the landing page connection flow
        
        // Load available chains only once per connection
        loadAvailableChains();
        
        // Setup refresh
        setupRefresh();
      }
    } else {
      // Clear interval if disconnected
      if (refreshInterval) {
        clearInterval(refreshInterval);
        refreshInterval = undefined;
      }
      // Reset state on disconnect
      lastConnectedChain = null;
      hasLoadedChains = false;
    }
  });

  // Refresh block height when selected chain changes
  $effect(() => {
    if (connectionState.selectedChain && connectionState.current?.isConnected) {
      console.log('🔄 BlockHeightHeader: Selected chain changed to:', connectionState.selectedChain, '- fetching block height');
      fetchBlockHeight();
    }
  });

  // Clean up on component destroy
  onDestroy(() => {
    if (refreshInterval) {
      clearInterval(refreshInterval);
    }
  });
</script>

<div class="bg-verusidx-snow-ice dark:bg-verusidx-stone-dark border-b border-verusidx-mountain-mist dark:border-verusidx-stone-medium px-8 py-3">
  <div class="max-w-7xl mx-auto flex items-center justify-between">
    <div class="flex items-center space-x-6">
      <!-- Chain Selector -->
      <div class="flex items-center space-x-2">
        <span class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Chain:</span>
        {#if connectionState.current?.chainName?.toLowerCase() !== 'vrsctest' && connectionState.availableChains.length > 1}
          <!-- Dropdown when multiple chains available -->
          <select 
            value={connectionState.selectedChain}
            onchange={(e) => handleChainSwitch((e.target as HTMLSelectElement).value)}
            disabled={isLoadingChains || connectionState.isConnecting}
            class="text-sm font-medium bg-transparent border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded px-2 py-1 text-verusidx-stone-dark dark:text-white focus:outline-none focus:ring-2 focus:ring-verusidx-turquoise-deep disabled:opacity-50"
          >
            {#each connectionState.availableChains as chain}
              <option value={chain.name} class="bg-white dark:bg-verusidx-stone-dark">
                {chain.name.toUpperCase()} {chain.is_active ? '🟢' : '🔴'}
              </option>
            {/each}
          </select>
          {#if connectionState.isConnecting}
            <span class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist ml-1">
              (Switching...)
            </span>
          {/if}
        {:else}
          <!-- Static display when only one chain or loading -->
          <span class="font-medium text-verusidx-stone-dark dark:text-white">
            {connectionState.selectedChain?.toUpperCase() || connectionState?.current?.chainName?.toUpperCase() || 'Unknown'}
            {#if isLoadingChains}
              <span class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist ml-1">(Loading chains...)</span>
            {/if}
          </span>
        {/if}
      </div>
      
      <!-- Block Height Display -->
      <div class="flex items-center space-x-2">
        <span class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Block Height:</span>
        {#if isLoading}
          <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Loading...</span>
        {:else if blockHeight !== null}
          <span class="font-medium text-verusidx-stone-dark dark:text-white">
            {blockHeight.toLocaleString()}
          </span>
        {:else}
          <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">N/A</span>
        {/if}
      </div>
    </div>
    
    <!-- Status Indicator -->
    <div class="flex items-center space-x-2">
      <div class="w-2 h-2 bg-verusidx-forest-deep rounded-full animate-pulse"></div>
      <span class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
        Auto-refresh (30s)
      </span>
    </div>
  </div>
</div>