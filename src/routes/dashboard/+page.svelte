<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { connectionStore, getChainParam, type ConnectionState } from "$lib/stores/connection";
  import { goto } from "$app/navigation";
  import { BlockHeightHeader, ExpandableCard } from "$lib/components";
  import { onMount, onDestroy } from "svelte";

  const DEBUG = false; // Set to true for development debugging

  // Helper function to format hash rate with appropriate units
  function formatHashRate(hashps: number): string {
    if (hashps >= 1e12) {
      return (hashps / 1e12).toFixed(2) + ' TH/s';
    } else if (hashps >= 1e9) {
      return (hashps / 1e9).toFixed(2) + ' GH/s';
    } else if (hashps >= 1e6) {
      return (hashps / 1e6).toFixed(2) + ' MH/s';
    } else if (hashps >= 1e3) {
      return (hashps / 1e3).toFixed(2) + ' KH/s';
    } else {
      return hashps.toFixed(2) + ' H/s';
    }
  }

  let connectionState = $state<ConnectionState>({ current: null, isConnecting: false, lastError: null, availableChains: [], selectedChain: null });
  let systemInfo = $state<any>(null);
  let walletInfo = $state<any>(null);
  let miningInfo = $state<any>(null);
  let totalBalances = $state<any>(null);
  let identities = $state<any[]>([]);
  let isLoading = $state(true);

  connectionStore.subscribe(state => {
    connectionState = state;
    if (!state.current?.isConnected) {
      goto("/");
    }
  });

  async function loadDashboardData() {
    if (!connectionState.current?.isConnected) return;

    isLoading = true;
    try {
      // Use the chain parameter helper to get correct chain value
      const chainParam = getChainParam(connectionState.selectedChain);
      if (DEBUG) console.log("Dashboard loading data for chain:", connectionState.selectedChain, "param:", chainParam);
      
      const [systemInfoResult, walletInfoResult, miningInfoResult, totalBalancesResult, identitiesResult] = await Promise.allSettled([
        invoke("get_info", { chain: chainParam }),
        invoke("get_wallet_info", { chain: chainParam }),
        invoke("get_mining_info", { chain: chainParam }),
        invoke("z_get_total_balance", { minconf: 1, include_watchonly: false, chain: chainParam }),
        invoke("list_identities", { chain: chainParam })
      ]);

      if (systemInfoResult.status === "fulfilled") {
        systemInfo = systemInfoResult.value;
        if (DEBUG) {
          console.log("=== SYSTEM INFO RESPONSE ===");
          console.log("Raw response:", JSON.stringify(systemInfo, null, 2));
          console.log("Field checks:");
          console.log("- vrsc_version (struct field):", systemInfo?.vrsc_version);
          console.log("- VRSCversion (actual field):", systemInfo?.VRSCversion);
          console.log("- version:", systemInfo?.version);
          console.log("- name:", systemInfo?.name);
          console.log("- chainid:", systemInfo?.chainid);
          console.log("- blocks:", systemInfo?.blocks);
          console.log("- difficulty:", systemInfo?.difficulty);
          console.log("=== END SYSTEM INFO ===");
        }
      } else {
        if (DEBUG) console.error("systemInfo failed:", systemInfoResult.reason);
      }
      if (walletInfoResult.status === "fulfilled") {
        walletInfo = walletInfoResult.value;
        if (DEBUG) {
          console.log("=== WALLET INFO RESPONSE ===");
          console.log("Raw response:", JSON.stringify(walletInfo, null, 2));
          console.log("Field checks:");
          console.log("- balance:", walletInfo?.balance);
          console.log("- unlocked_balance:", walletInfo?.unlocked_balance);
          console.log("- reserve_balance:", walletInfo?.reserve_balance);
          console.log("=== END WALLET INFO ===");
        }
      } else {
        if (DEBUG) console.error("walletInfo failed:", walletInfoResult.reason);
      }
      if (miningInfoResult.status === "fulfilled") {
        miningInfo = miningInfoResult.value;
        if (DEBUG) {
          console.log("=== MINING INFO RESPONSE ===");
          console.log("Raw response:", JSON.stringify(miningInfo, null, 2));
          console.log("Field checks:");
          console.log("- networkhashps:", miningInfo?.networkhashps);
          console.log("- stakingsupply:", miningInfo?.stakingsupply);
          console.log("- difficulty:", miningInfo?.difficulty);
          console.log("=== END MINING INFO ===");
        }
      } else {
        if (DEBUG) console.error("miningInfo failed:", miningInfoResult.reason);
      }
      if (totalBalancesResult.status === "fulfilled") {
        totalBalances = totalBalancesResult.value;
        if (DEBUG) {
          console.log("=== TOTAL BALANCE RESPONSE ===");
          console.log("Raw response:", JSON.stringify(totalBalances, null, 2));
          console.log("Field checks:");
          console.log("- transparent:", totalBalances?.transparent);
          console.log("- private:", totalBalances?.private);
          console.log("- total:", totalBalances?.total);
          console.log("=== END TOTAL BALANCE ===");
        }
      } else {
        if (DEBUG) console.error("totalBalances failed:", totalBalancesResult.reason);
      }
      if (identitiesResult.status === "fulfilled") {
        identities = identitiesResult.value as any[];
        if (DEBUG) {
          console.log("=== IDENTITIES RESPONSE ===");
          console.log("Response count:", identities.length);
          if (identities.length > 0) {
            console.log("First identity sample:", JSON.stringify(identities[0], null, 2));
          }
          console.log("=== END IDENTITIES ===");
        }
      } else {
        if (DEBUG) console.error("identities failed:", identitiesResult.reason);
      }
    } catch (error) {
      if (DEBUG) console.error("Failed to load dashboard data:", error);
    } finally {
      isLoading = false;
    }
  }


  function disconnect() {
    connectionStore.update(state => ({ ...state, current: null }));
    goto("/");
  }

  // Navigation functions for defi operations
  function navigateToSendTransparent() {
    goto("/defi?operation=send-transparent");
  }

  function navigateToSendPrivate() {
    goto("/defi?operation=send-private");
  }

  function navigateToConvertReserveToBasket() {
    goto("/defi?operation=convert-reserve-to-basket");
  }

  function navigateToConvertBasketToReserve() {
    goto("/defi?operation=convert-basket-to-reserve");
  }

  function navigateToConvertReserveToReserve() {
    goto("/defi?operation=convert-reserve");
  }

  // State for convert dropdown
  let convertDropdownOpen = $state<string | null>(null);
  let dropdownPositions = $state<Record<string, 'above' | 'below'>>({});

  function toggleConvertDropdown(currency: string, event: MouseEvent) {
    if (convertDropdownOpen === currency) {
      convertDropdownOpen = null;
      return;
    }
    
    // Calculate optimal dropdown position relative to modal container
    const button = event.currentTarget as HTMLElement;
    const rect = button.getBoundingClientRect();
    const dropdownHeight = 120; // Approximate height of 3 options
    
    // Find the modal container to calculate space within it
    const modal = button.closest('[role="dialog"]') as HTMLElement;
    let spaceBelow: number;
    
    if (modal) {
      // Calculate space within modal container
      const modalRect = modal.getBoundingClientRect();
      spaceBelow = modalRect.bottom - rect.bottom;
    } else {
      // Fallback to viewport if modal not found
      spaceBelow = window.innerHeight - rect.bottom;
    }
    
    // Check if there's enough space below the button within the modal
    const shouldShowAbove = spaceBelow < dropdownHeight + 20; // 20px buffer
    
    dropdownPositions = {
      ...dropdownPositions,
      [currency]: shouldShowAbove ? 'above' : 'below'
    };
    
    convertDropdownOpen = currency;
  }

  function closeConvertDropdown() {
    convertDropdownOpen = null;
  }

  // Chain change event handler for global chain switching
  function handleChainChanged(event: CustomEvent) {
    if (DEBUG) console.log("Dashboard: Chain changed to", event.detail.chainName);
    loadDashboardData();
  }

  // Load dashboard data when component mounts and connection is available
  $effect(() => {
    if (connectionState.current?.isConnected && connectionState.selectedChain) {
      loadDashboardData();
    }
  });

  // Set up chain change event listener
  onMount(() => {
    window.addEventListener('chainChanged', handleChainChanged);
  });

  onDestroy(() => {
    window.removeEventListener('chainChanged', handleChainChanged);
  });

  // Close dropdown when clicking outside
  $effect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (convertDropdownOpen && !(event.target as Element)?.closest('.convert-dropdown-container')) {
        closeConvertDropdown();
      }
    }

    if (convertDropdownOpen) {
      document.addEventListener('click', handleClickOutside, true);
      return () => document.removeEventListener('click', handleClickOutside, true);
    }
  });
</script>

<div class="bg-verusidx-sky-soft dark:bg-verusidx-lake-deep min-h-screen max-h-screen overflow-y-auto">
  <!-- Block Height Header -->
  <BlockHeightHeader />

  <div class="max-w-7xl mx-auto p-8 pb-16 space-y-8">
    <!-- Header -->
    <div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl p-6">
      <div class="flex justify-between items-center">
        <div class="flex items-center space-x-6">
          <div>
            <h1 class="text-3xl font-bold text-verusidx-stone-dark dark:text-white">VerusIDX Dashboard</h1>
            <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
              Connected to {connectionState.current?.chainName} 
              ({connectionState.current?.host}:{connectionState.current?.port})
            </p>
          </div>
          
        </div>
        
        <button
          onclick={disconnect}
          class="px-4 py-2 bg-verusidx-lake-deep dark:bg-verusidx-turquoise-deep text-white rounded-lg hover:bg-verusidx-mountain-blue dark:hover:bg-verusidx-turquoise-bright transition-colors"
        >
          Home
        </button>
      </div>
    </div>


    {#if isLoading}
      <div class="text-center py-12">
        <div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-verusidx-mountain-blue"></div>
        <p class="mt-4 text-verusidx-mountain-grey">Loading dashboard...</p>
      </div>
    {:else}
      <!-- Enhanced Stats Grid -->
      <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
        <!-- Network Status Card -->
        {#if systemInfo || miningInfo}
          <ExpandableCard title="Network Status" cardClass="bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white" modalSize="lg">
            <div slot="preview">
              <div class="space-y-3">
                <div class="flex justify-between">
                  <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Chain:</span>
                  <span class="font-medium text-verusidx-stone-dark dark:text-white">{systemInfo?.name || connectionState.selectedChain?.toUpperCase()}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Blocks:</span>
                  <span class="font-medium text-verusidx-stone-dark dark:text-white">{systemInfo?.blocks?.toLocaleString() || 'N/A'}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Difficulty:</span>
                  <span class="font-medium text-verusidx-stone-dark dark:text-white">{systemInfo?.difficulty ? Number(systemInfo.difficulty).toLocaleString() : 'N/A'}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Network Hash:</span>
                  <span class="font-medium text-verusidx-stone-dark dark:text-white">{miningInfo?.networkhashps ? formatHashRate(miningInfo.networkhashps) : 'N/A'}</span>
                </div>
              </div>
            </div>
            
            <div slot="expanded" class="space-y-6">
              <!-- Basic Info -->
              <div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-4 rounded-lg">
                <h4 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-3">Chain Information</h4>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div class="flex justify-between">
                    <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Chain:</span>
                    <span class="font-medium text-verusidx-stone-dark dark:text-white">{systemInfo?.name || connectionState.selectedChain?.toUpperCase()}</span>
                  </div>
                  <div class="flex justify-between">
                    <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Version:</span>
                    <span class="font-medium text-verusidx-stone-dark dark:text-white">{systemInfo?.VRSCversion || systemInfo?.vrsc_version || systemInfo?.version || 'N/A'}</span>
                  </div>
                  <div class="col-span-1 md:col-span-2">
                    <div class="flex items-start space-x-2">
                      <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist whitespace-nowrap">Chain ID:</span>
                      <span class="font-medium text-verusidx-stone-dark dark:text-white text-xs break-all">{systemInfo?.chainid || 'N/A'}</span>
                    </div>
                  </div>
                </div>
              </div>
              
              <!-- Block Info -->
              <div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-4 rounded-lg">
                <h4 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-3">Block Information</h4>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div class="flex justify-between">
                    <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Current Blocks:</span>
                    <span class="font-medium text-verusidx-stone-dark dark:text-white">{systemInfo?.blocks?.toLocaleString() || 'N/A'}</span>
                  </div>
                  <div class="flex justify-between">
                    <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Longest Chain:</span>
                    <span class="font-medium text-verusidx-stone-dark dark:text-white">{systemInfo?.longestchain?.toLocaleString() || 'N/A'}</span>
                  </div>
                  <div class="flex justify-between">
                    <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Difficulty:</span>
                    <span class="font-medium text-verusidx-stone-dark dark:text-white">{systemInfo?.difficulty ? Number(systemInfo.difficulty).toLocaleString() : 'N/A'}</span>
                  </div>
                  <div class="flex justify-between">
                    <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Connections:</span>
                    <span class="font-medium text-verusidx-stone-dark dark:text-white">{systemInfo?.connections || 'N/A'}</span>
                  </div>
                </div>
              </div>
              
              <!-- Mining Info -->
              {#if miningInfo}
                <div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-4 rounded-lg">
                  <h4 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-3">Network Mining</h4>
                  <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div class="flex justify-between">
                      <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Network Hash:</span>
                      <span class="font-medium text-verusidx-stone-dark dark:text-white">{miningInfo?.networkhashps ? formatHashRate(miningInfo.networkhashps) : 'N/A'}</span>
                    </div>
                    <div class="flex justify-between">
                      <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Staking Supply:</span>
                      <span class="font-medium text-verusidx-stone-dark dark:text-white">{miningInfo?.stakingsupply !== undefined ? Number(miningInfo.stakingsupply).toLocaleString() : 'N/A'}</span>
                    </div>
                    <div class="flex justify-between">
                      <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Staking:</span>
                      <span class="font-medium text-verusidx-stone-dark dark:text-white">{miningInfo?.staking ? 'Yes' : 'No'}</span>
                    </div>
                    <div class="flex justify-between">
                      <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Generate:</span>
                      <span class="font-medium text-verusidx-stone-dark dark:text-white">{miningInfo?.generate ? 'Yes' : 'No'}</span>
                    </div>
                  </div>
                </div>
              {/if}
            </div>
          </ExpandableCard>
        {/if}

        <!-- Wallet Balances Card -->
        {#if walletInfo}
          <ExpandableCard title="{systemInfo?.name || connectionState.selectedChain?.toUpperCase() || 'Wallet'} Balances" cardClass="bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white" modalSize="md">
            <div slot="preview">
              <div class="space-y-3">
                <div class="flex justify-between">
                  <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Transparent:</span>
                  <span class="font-medium text-verusidx-stone-dark dark:text-white">{walletInfo?.balance !== undefined ? Number(walletInfo.balance).toFixed(8) : '0.00000000'}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Private:</span>
                  <span class="font-medium text-verusidx-stone-dark dark:text-white">{totalBalances?.private !== undefined ? totalBalances.private : '0.00000000'}</span>
                </div>
                <div class="flex justify-between border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium pt-2">
                  <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Total:</span>
                  <span class="font-bold text-verusidx-stone-dark dark:text-white text-lg">{totalBalances?.total !== undefined ? totalBalances.total : '0.00000000'}</span>
                </div>
              </div>
            </div>
            
            <div slot="expanded" class="space-y-4">
              <div class="grid gap-4">
                <div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-4 rounded-lg">
                  <div class="flex justify-between items-center mb-2">
                    <div>
                      <span class="font-medium text-verusidx-stone-dark dark:text-white">Transparent Balance</span>
                      <div class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Available for spending</div>
                    </div>
                    <div class="text-right">
                      <div class="font-medium text-verusidx-stone-dark dark:text-white">{walletInfo?.balance !== undefined ? Number(walletInfo.balance).toFixed(8) : '0.00000000'}</div>
                      <div class="flex gap-2 mt-2">
                        <button 
                          onclick={navigateToSendTransparent}
                          class="text-xs px-2 py-1 bg-verusidx-mountain-blue text-white rounded hover:bg-verusidx-lake-blue transition-colors"
                        >
                          Send
                        </button>
                        <button class="text-xs px-2 py-1 bg-verusidx-turquoise-deep text-white rounded hover:bg-verusidx-turquoise-bright transition-colors">
                          Receive
                        </button>
                      </div>
                    </div>
                  </div>
                </div>
                
                {#if walletInfo?.unlocked_balance !== undefined && walletInfo?.unlocked_balance !== null}
                  <div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-4 rounded-lg">
                    <div class="flex justify-between items-center mb-2">
                      <div>
                        <span class="font-medium text-verusidx-stone-dark dark:text-white">Unlocked Balance</span>
                        <div class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Ready for immediate use</div>
                      </div>
                      <div class="text-right">
                        <div class="font-medium text-verusidx-stone-dark dark:text-white">{Number(walletInfo.unlocked_balance).toFixed(8)}</div>
                      </div>
                    </div>
                  </div>
                {/if}
                
                <div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-4 rounded-lg">
                  <div class="flex justify-between items-center mb-2">
                    <div>
                      <span class="font-medium text-verusidx-stone-dark dark:text-white">Private Balance</span>
                      <div class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Shielded funds</div>
                    </div>
                    <div class="text-right">
                      <div class="font-medium text-verusidx-stone-dark dark:text-white">{totalBalances?.private !== undefined ? totalBalances.private : '0.00000000'}</div>
                      <div class="flex gap-2 mt-2">
                        <button 
                          onclick={navigateToSendPrivate}
                          class="text-xs px-2 py-1 bg-verusidx-mountain-blue text-white rounded hover:bg-verusidx-lake-blue transition-colors"
                        >
                          Send
                        </button>
                        <button class="text-xs px-2 py-1 bg-verusidx-turquoise-deep text-white rounded hover:bg-verusidx-turquoise-bright transition-colors">
                          Receive
                        </button>
                      </div>
                    </div>
                  </div>
                </div>
                
                <div class="bg-verusidx-turquoise-light/10 dark:bg-verusidx-turquoise-light/20 p-4 rounded-lg border border-verusidx-turquoise-light">
                  <div class="flex justify-between items-center">
                    <div>
                      <span class="font-bold text-verusidx-stone-dark dark:text-white text-lg">Total Balance</span>
                      <div class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">All funds combined</div>
                    </div>
                    <div class="text-right">
                      <div class="font-bold text-verusidx-stone-dark dark:text-white text-xl">{totalBalances?.total !== undefined ? totalBalances.total : '0.00000000'}</div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </ExpandableCard>
        {/if}

        <!-- Currencies Card -->
        <ExpandableCard title="Currencies" cardClass="bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white" modalSize="md">
          <div slot="preview">
            {#if walletInfo?.reserve_balance && Object.keys(walletInfo.reserve_balance).length > 0}
              {#each Object.entries(walletInfo.reserve_balance).slice(0, 4) as [currency, amount]}
                <div class="flex justify-between">
                  <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">{currency}:</span>
                  <span class="font-medium text-verusidx-stone-dark dark:text-white">{Number(amount).toFixed(8)}</span>
                </div>
              {/each}
              {#if Object.keys(walletInfo.reserve_balance).length > 4}
                <div class="text-center text-verusidx-mountain-grey dark:text-verusidx-mountain-mist text-sm mt-2">
                  And {Object.keys(walletInfo.reserve_balance).length - 4} more...
                </div>
              {/if}
            {:else}
              <div class="text-center text-verusidx-mountain-grey dark:text-verusidx-mountain-mist py-4">
                <p class="text-sm">No currencies</p>
              </div>
            {/if}
          </div>
          
          <div slot="expanded" class="space-y-4">
            {#if walletInfo?.reserve_balance && Object.keys(walletInfo.reserve_balance).length > 0}
              <div class="grid gap-4">
                {#each Object.entries(walletInfo.reserve_balance) as [currency, amount]}
                  <div class="flex justify-between items-center p-4 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium rounded-lg">
                    <div>
                      <span class="font-medium text-verusidx-stone-dark dark:text-white">{currency}</span>
                      <div class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Balance</div>
                    </div>
                    <div class="text-right">
                      <div class="font-medium text-verusidx-stone-dark dark:text-white">{Number(amount).toFixed(8)}</div>
                      <div class="flex gap-2 mt-2">
                        <button 
                          onclick={navigateToSendTransparent}
                          class="text-xs px-2 py-1 bg-verusidx-mountain-blue text-white rounded hover:bg-verusidx-lake-blue transition-colors"
                        >
                          Send
                        </button>
                        <div class="relative convert-dropdown-container">
                          <button 
                            onclick={(e) => toggleConvertDropdown(currency, e)}
                            class="text-xs px-2 py-1 bg-verusidx-turquoise-deep text-white rounded hover:bg-verusidx-turquoise-bright transition-colors flex items-center gap-1"
                          >
                            Convert
                            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                            </svg>
                          </button>
                          
                          {#if convertDropdownOpen === currency}
                            <div class="absolute right-0 w-48 bg-white dark:bg-verusidx-stone-dark border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg shadow-lg z-50 {dropdownPositions[currency] === 'above' ? 'bottom-full mb-1' : 'top-full mt-1'}">
                              <button 
                                onclick={() => { closeConvertDropdown(); navigateToConvertReserveToBasket(); }}
                                class="w-full text-left px-3 py-2 text-xs text-verusidx-stone-dark dark:text-white hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors first:rounded-t-lg"
                              >
                                Reserve to Basket
                              </button>
                              <button 
                                onclick={() => { closeConvertDropdown(); navigateToConvertBasketToReserve(); }}
                                class="w-full text-left px-3 py-2 text-xs text-verusidx-stone-dark dark:text-white hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors"
                              >
                                Basket to Reserve
                              </button>
                              <button 
                                onclick={() => { closeConvertDropdown(); navigateToConvertReserveToReserve(); }}
                                class="w-full text-left px-3 py-2 text-xs text-verusidx-stone-dark dark:text-white hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors last:rounded-b-lg"
                              >
                                Reserve to Reserve
                              </button>
                            </div>
                          {/if}
                        </div>
                      </div>
                    </div>
                  </div>
                {/each}
              </div>
            {:else}
              <div class="text-center text-verusidx-mountain-grey dark:text-verusidx-mountain-mist py-8">
                <p>No currencies available</p>
              </div>
            {/if}
          </div>
        </ExpandableCard>
      </div>


      <!-- Navigation Links -->
      <div class="mt-8 space-y-4">
        <!-- Top row -->
        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
          <a
            href="/vlotto"
            class="p-6 bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl hover:shadow-2xl transition-all text-center"
          >
            <h4 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-2">vLotto</h4>
            <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">View stats and status</p>
          </a>
          
          <a 
            href="/wallet" 
            class="p-6 bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl hover:shadow-2xl transition-all text-center"
          >
            <h4 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-2">Wallet</h4>
            <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Manage your addresses and transactions</p>
          </a>
          
          <a 
            href="/defi" 
            class="p-6 bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl hover:shadow-2xl transition-all text-center"
          >
            <h4 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-2">DeFi Operations</h4>
            <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Send and convert currencies</p>
          </a>
        </div>
        
        <!-- Bottom row -->
        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
          <a 
            href="/identities" 
            class="p-6 bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl hover:shadow-2xl transition-all text-center"
          >
            <h4 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-2">Identities</h4>
            <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Create, manage, and search IDs</p>
          </a>
          
          <a 
            href="/currencies" 
            class="p-6 bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl hover:shadow-2xl transition-all text-center"
          >
            <h4 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-2">Currencies</h4>
            <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Create, search, and discover network currencies</p>
          </a>
          
          <a 
            href="/marketplace" 
            class="p-6 bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl hover:shadow-2xl transition-all text-center"
          >
            <h4 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-2">Marketplace</h4>
            <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Browse, make, take, and manage offers for IDs & currencies</p>
          </a>
        </div>
      </div>
    {/if}
  </div>
</div>