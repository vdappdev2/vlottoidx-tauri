<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { connectionStore, getChainParam, type ConnectionState } from "$lib/stores/connection";
  import { goto } from "$app/navigation";
  import ThemeToggle from "$lib/components/ThemeToggle.svelte";

  let connectionState = $state<ConnectionState>({ current: null, isConnecting: false, lastError: null, availableChains: [], selectedChain: null });
  let formData = $state({
    host: "127.0.0.1",
    port: 18843, // Default to testnet port
    username: "",
    password: "",
    rememberCredentials: false
  });
  let availableChains = $state<any[]>([]);
  let isDiscoveringChains = $state(false);
  let showManualEntry = $state(false);
  let hasSavedCredentials = $state(false);
  let showRemoteWarning = $state(false);
  
  // RPC data for mock elements
  let blockHeight = $state<number | null>(null);
  let systemInfo = $state<any>(null);
  let walletInfo = $state<any>(null);
  let identities = $state<any[]>([]);
  let isLoadingRPCData = $state(false);
  let hasValidSubIdentity = $state<boolean>(false);

  connectionStore.subscribe(state => {
    connectionState = state;
  });

  async function discoverChains() {
    isDiscoveringChains = true;
    try {
      console.log("🔍 Frontend: Starting chain discovery...");
      const chains = await invoke("discover_chains");
      availableChains = chains as any[];
      console.log("🔍 Frontend discovered chains:", availableChains);
      
      // Debug each chain's status
      availableChains.forEach((chain, index) => {
        console.log(`🔍 Chain ${index}:`, {
          name: chain.name,
          host: chain.credentials?.host,
          port: chain.credentials?.port,
          is_active: chain.is_active,
          full_chain: chain
        });
      });
      
      if (availableChains.length === 0) {
        console.log("🔍 No chains found, showing manual entry");
        showManualEntry = true;
      }
    } catch (error) {
      console.error("🔍 Chain discovery failed:", error);
      showManualEntry = true;
    } finally {
      isDiscoveringChains = false;
    }
  }

  async function connectToChain(chainConfig?: any) {
    connectionStore.update(state => ({ ...state, isConnecting: true, lastError: null }));
    
    try {
      if (chainConfig) {
        // Connect to discovered chain
        console.log("Connecting to chain:", chainConfig.name, "Full config:", chainConfig);
        const result = await invoke("connect_to_chain", {
          chainName: chainConfig.name
        });
        
        if (result) {
          const connection = {
            host: chainConfig.credentials.host,
            port: chainConfig.credentials.port,
            username: chainConfig.credentials.username,
            password: chainConfig.credentials.password,
            isConnected: true,
            chainName: chainConfig.name
          };
          
          connectionStore.update(state => ({ 
            ...state, 
            current: connection, 
            selectedChain: chainConfig.name,  // Fix race condition: set selectedChain immediately
            isConnecting: false 
          }));
          
          // Check if this is testnet and redirect immediately
          const isTestnet = chainConfig.name.toLowerCase().includes('test');
          if (isTestnet) {
            goto("/dashboard");
          } else {
            // Mainnet chains go to access page first
            goto("/access");
          }
        }
      } else {
        // Manual connection - keep existing logic for manual form
        // This path will need manual testing since test_rpc_connection was removed
        const credentials = formData;
        
        // For manual connections, determine chain name based on port
        // Common ports: 27486 (VRSC), 18843 (VRSCTEST), etc.
        let chainName = "VRSC"; // Default to mainnet
        if (credentials.port === 18843) {
          chainName = "VRSCTEST";
        }
        
        // For manual connections, we'll assume success and let dashboard handle validation
        const connection = {
          host: credentials.host,
          port: credentials.port,
          username: credentials.username,
          password: credentials.password,
          isConnected: true,
          chainName: chainName
        };
        
        connectionStore.update(state => ({ 
          ...state, 
          current: connection, 
          selectedChain: chainName,  // Fix: use actual chain name, not "Manual"
          isConnecting: false 
        }));
        
        if (formData.rememberCredentials) {
          await invoke("store_credentials", { 
            chain_name: "manual",
            username: credentials.username,
            password: credentials.password,
            host: credentials.host,
            port: credentials.port
          });
        }
      }
    } catch (error) {
      connectionStore.update(state => ({ 
        ...state, 
        isConnecting: false, 
        lastError: typeof error === 'string' ? error : 'Connection failed' 
      }));
    }
  }

  async function handleManualConnect(event: Event) {
    event.preventDefault();

    // Check if connecting to remote host and show warning
    const isRemote = formData.host !== "127.0.0.1" && formData.host !== "localhost";
    if (isRemote && !showRemoteWarning) {
      showRemoteWarning = true;
      return;
    }

    connectionStore.update(state => ({ ...state, isConnecting: true, lastError: null }));

    try {
      console.log("Testing manual connection to:", formData.host, formData.port);

      // Call backend to test connection and create RPC client
      const result = await invoke("test_and_connect_manual", {
        host: formData.host,
        port: formData.port,
        username: formData.username,
        password: formData.password
      }) as any;

      console.log("Manual connection successful:", result);

      // Update connection store with successful connection
      const connection = {
        host: formData.host,
        port: formData.port,
        username: formData.username,
        password: formData.password,
        isConnected: true,
        chainName: result.chainName
      };

      connectionStore.update(state => ({
        ...state,
        current: connection,
        selectedChain: result.chainName,
        isConnecting: false
      }));

      // Store credentials if user requested
      if (formData.rememberCredentials) {
        await invoke("store_credentials", {
          chain_name: "manual",
          username: formData.username,
          password: formData.password,
          host: formData.host,
          port: formData.port
        });
      }

      // Redirect based on chain type
      const isTestnet = result.chainName.toLowerCase().includes('test');
      if (isTestnet) {
        goto("/dashboard");
      } else {
        goto("/access");
      }

    } catch (error) {
      console.error("Manual connection failed:", error);
      connectionStore.update(state => ({
        ...state,
        isConnecting: false,
        lastError: typeof error === 'string' ? error : 'Connection failed. Please check your credentials and daemon status.'
      }));
    }
  }

  async function checkSavedCredentials() {
    try {
      await invoke("load_credentials", { chain_name: "manual" });
      hasSavedCredentials = true;
    } catch {
      hasSavedCredentials = false;
    }
  }

  async function forgetCredentials() {
    try {
      await invoke("clear_credentials", { chain_name: "manual" });
      hasSavedCredentials = false;
      // Clear form
      formData.username = "";
      formData.password = "";
      formData.rememberCredentials = false;
    } catch (error) {
      console.error("Failed to clear credentials:", error);
    }
  }

  // Disconnect function
  function disconnect() {
    connectionStore.update(() => ({ 
      current: null, 
      isConnecting: false, 
      lastError: null 
    }));
    // Reset all RPC data
    blockHeight = null;
    systemInfo = null;
    walletInfo = null;
    identities = [];
    hasValidSubIdentity = false;
  }

  // Validate sub-identity for marketplace access
  function validateSubIdentity(identityList: any[]) {
    // Required parent identity for agents marketplace access
    // Testing configuration
    // const REQUIRED_PARENT = "iH9HFQeKRNVWguokGLLaiVYqy9u8VuFWMe";
    // Production configuration
    const REQUIRED_PARENT = "i8Rar9Z64hDXtpBRJBhK9b6S6dMV1EttiS";
    
    const validSubIdentities = identityList.filter(identity => {
      const parent = identity.identity?.parent || "";
      const primaryAddresses = identity.identity?.primaryaddresses || [];
      
      // Must have correct parent AND exactly 1 primary address (prevents shared access)
      return parent === REQUIRED_PARENT && primaryAddresses.length === 1;
    });
    
    hasValidSubIdentity = validSubIdentities.length > 0;
    console.log(`Found ${validSubIdentities.length} valid sub-identities:`, validSubIdentities);
    return hasValidSubIdentity;
  }

  // Load RPC data for landing page gateway
  async function loadRPCData() {
    if (!connectionState.current?.isConnected) return;
    
    isLoadingRPCData = true;
    try {
      // Use chain parameter helper to get correct chain value
      const chainParam = getChainParam(connectionState.selectedChain);
      console.log("Landing page loading data for chain:", connectionState.selectedChain, "param:", chainParam);
      
      const blockCount = await invoke("get_block_count", { chain: chainParam });
      blockHeight = blockCount as number;
      
      // Get system info
      const info = await invoke("get_info", { chain: chainParam });
      systemInfo = info;
      
      // Get wallet info
      const wallet = await invoke("get_wallet_info", { chain: chainParam });
      walletInfo = wallet;
      
      // Check if this is testnet (vrsctest)
      const currentChain = connectionState.current?.chainName?.toLowerCase() || '';
      const isTestnet = currentChain.includes('test');
      
      if (isTestnet) {
        // For testnet, go directly to dashboard
        goto("/dashboard");
        return;
      }
      
      // For mainnet chains, get identities for sub-identity validation
      const identityList = await invoke("list_identities", { chain: chainParam });
      identities = Array.isArray(identityList) ? identityList : [];
      
      // Validate sub-identity for marketplace access
      validateSubIdentity(identities);
    } catch (error) {
      console.error("Failed to load RPC data:", error);
    } finally {
      isLoadingRPCData = false;
    }
  }
  
  // Load RPC data when connected
  $effect(() => {
    if (connectionState.current?.isConnected) {
      loadRPCData();
    }
  });

  // Auto-discover chains on component mount
  discoverChains();

  // Check for saved credentials when manual entry is shown
  $effect(() => {
    if (showManualEntry) {
      checkSavedCredentials();
    }
  });
</script>

<div class="bg-verusidx-sky-soft dark:bg-verusidx-lake-deep min-h-screen max-h-screen overflow-y-auto transition-colors duration-300">
  
  
  <!-- Theme Toggle for Landing Page (top right when not connected) -->
  {#if !connectionState.current?.isConnected}
    <div class="fixed top-4 right-4 z-50">
      <ThemeToggle variant="floating" />
    </div>
  {/if}

  <!-- Main Content Area -->
  <div class="flex flex-col">
    
    <!-- Header with System Status (when connected) -->
    {#if connectionState.current?.isConnected && blockHeight !== null}
      <div class="h-20 bg-verusidx-snow-ice dark:bg-verusidx-stone-dark border-b border-verusidx-mountain-mist dark:border-verusidx-stone-medium px-8 flex items-center justify-between">
        <div class="flex items-center space-x-8">
          <div>
            <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Chain</p>
            <p class="font-semibold text-verusidx-stone-dark dark:text-white">VRSCTEST</p>
          </div>
          <div>
            <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Block Height</p>
            <p class="font-semibold text-verusidx-stone-dark dark:text-white">{blockHeight.toLocaleString()}</p>
          </div>
          {#if systemInfo}
            <div>
              <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Version</p>
              <p class="font-semibold text-verusidx-stone-dark dark:text-white">{systemInfo.VRSCversion || systemInfo.vrsc_version || 'Unknown'}</p>
            </div>
            <div>
              <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Connections</p>
              <p class="font-semibold text-verusidx-stone-dark dark:text-white">{systemInfo.connections || 0}</p>
            </div>
          {/if}
        </div>
        <div class="flex items-center space-x-2">
          <div class="w-3 h-3 bg-verusidx-forest-deep rounded-full animate-pulse"></div>
          <span class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Connected</span>
        </div>
      </div>
    {/if}

    <!-- Connected State: RPC Data & Validation -->
    {#if connectionState.current?.isConnected}
      <div class="flex-1 p-8">
        <!-- Connection Status with Disconnect -->
        <div class="flex justify-between items-center mb-8">
          <div>
            <h2 class="text-2xl font-bold text-verusidx-stone-dark dark:text-white">Connected to Verus PBaaS</h2>
            <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
              {connectionState.current.chainName} - {connectionState.current.host}:{connectionState.current.port}
            </p>
          </div>
          <button 
            onclick={disconnect}
            class="px-4 py-2 bg-verusidx-lake-deep text-white rounded-lg hover:bg-verusidx-mountain-blue transition-colors"
          >
            Disconnect
          </button>
        </div>

        {#if isLoadingRPCData}
          <div class="flex justify-center items-center py-12">
            <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
            <span class="ml-3 text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Loading RPC data...</span>
          </div>
        {:else if systemInfo || blockHeight !== null}
          <!-- RPC Data Cards -->
          <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
            <!-- System Info Card -->
            <div class="bg-verusidx-mountain-blue p-6 rounded-2xl shadow-xl text-white">
              <h3 class="text-lg font-medium opacity-90">System Info</h3>
              <div class="mt-4 space-y-3">
                <div>
                  <span class="text-sm opacity-70">Version</span>
                  <p class="font-semibold">{systemInfo?.VRSCversion || 'Loading...'}</p>
                </div>
                <div>
                  <span class="text-sm opacity-70">Block Height</span>
                  <p class="font-semibold">{blockHeight !== null ? blockHeight.toLocaleString() : 'Loading...'}</p>
                </div>
                <div>
                  <span class="text-sm opacity-70">Connections</span>
                  <p class="font-semibold">{systemInfo?.connections || 'Loading...'}</p>
                </div>
              </div>
            </div>

            <!-- Wallet Balance Card -->
            {#if walletInfo}
              <div class="bg-verusidx-turquoise-deep p-6 rounded-2xl shadow-xl text-white">
                <h3 class="text-lg font-medium opacity-90">Wallet Balance</h3>
                <p class="text-2xl font-bold mt-2">{walletInfo.balance?.toFixed(8) || '0.00000000'} VRSC</p>
                <div class="mt-4 space-y-2">
                  {#if walletInfo.unlocked_balance}
                    <div class="flex justify-between text-sm">
                      <span class="opacity-70">Unlocked</span>
                      <span>{walletInfo.unlocked_balance.toFixed(8)}</span>
                    </div>
                  {/if}
                  <div class="flex justify-between text-sm">
                    <span class="opacity-70">Unconfirmed</span>
                    <span>{walletInfo.unconfirmed_balance?.toFixed(8) || '0.00000000'}</span>
                  </div>
                </div>
              </div>
            {:else}
              <div class="bg-verusidx-snow-ice dark:bg-verusidx-stone-dark/30 p-6 rounded-2xl shadow-xl border-2 border-dashed border-gray-300 dark:border-verusidx-stone-medium">
                <h3 class="text-lg font-medium text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Wallet Info</h3>
                <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-grey mt-2">Loading wallet data...</p>
              </div>
            {/if}

            <!-- Reserve Currencies Card -->
            {#if walletInfo?.reserve_balance && Object.keys(walletInfo.reserve_balance).length > 0}
              <div class="bg-verusidx-stone-dark p-6 rounded-2xl shadow-xl text-white">
                <h3 class="text-lg font-medium opacity-90">Reserve Currencies</h3>
                <div class="mt-4 space-y-2">
                  {#each Object.entries(walletInfo.reserve_balance) as [currency, amount]}
                    <div class="flex justify-between text-sm">
                      <span class="opacity-80">{currency}</span>
                      <span class="font-medium">{amount}</span>
                    </div>
                  {/each}
                </div>
              </div>
            {:else}
              <div class="bg-verusidx-snow-ice dark:bg-verusidx-stone-dark/30 p-6 rounded-2xl shadow-xl border-2 border-dashed border-gray-300 dark:border-verusidx-stone-medium">
                <h3 class="text-lg font-medium text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Reserve Currencies</h3>
                <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-grey mt-2">No reserve currencies found</p>
              </div>
            {/if}
          </div>
        {:else}
          <!-- RPC Data Loading Failed -->
          <div class="text-center py-12">
            <div class="mb-4">
              <svg class="w-16 h-16 text-verusidx-mountain-mist dark:text-verusidx-mountain-grey mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
              </svg>
            </div>
            <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-2">Loading RPC Data</h3>
            <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-4">
              Fetching blockchain and wallet information...
            </p>
            <button 
              onclick={loadRPCData}
              class="px-6 py-2 bg-verusidx-mountain-blue dark:bg-verusidx-turquoise-deep text-white rounded-lg hover:bg-verusidx-lake-blue dark:hover:bg-verusidx-turquoise-bright transition-all duration-200"
            >
              Retry Loading Data
            </button>
          </div>
        {/if}

          <!-- Continue to Dashboard Section -->
          <div class="bg-verusidx-snow-ice dark:bg-verusidx-stone-dark rounded-2xl p-6 text-center">
            <h3 class="text-xl font-semibold text-verusidx-stone-dark dark:text-white mb-2">Access Dashboard</h3>
            
            {#if hasValidSubIdentity}
              <div class="mb-4 p-4 bg-verusidx-forest-deep/10 dark:bg-verusidx-stone-dark/20 border border-verusidx-turquoise-light dark:border-verusidx-turquoise-light rounded-lg">
                <div class="flex items-center justify-center mb-2">
                  <svg class="w-6 h-6 text-verusidx-forest-deep dark:text-verusidx-turquoise-light mr-2" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"></path>
                  </svg>
                  <span class="text-verusidx-forest-deep dark:text-verusidx-turquoise-light font-medium">Marketplace Access Approved</span>
                </div>
                <p class="text-verusidx-forest-deep dark:text-verusidx-turquoise-light text-sm">
                  You have valid VerusIDX sub-identities for marketplace operations.
                </p>
              </div>
              <button 
                onclick={() => goto("/dashboard")}
                class="px-8 py-3 bg-verusidx-mountain-blue dark:bg-verusidx-turquoise-deep text-white rounded-lg hover:bg-verusidx-lake-blue dark:hover:bg-verusidx-turquoise-bright transition-all duration-200"
              >
                Continue to Dashboard
              </button>
            {:else}
              <div class="mb-4 p-4 bg-verusidx-turquoise-bright/10 dark:bg-verusidx-lake-deep/50 border border-verusidx-turquoise-light dark:border-verusidx-turquoise-light rounded-lg">
                <div class="flex items-center justify-center mb-2">
                  <svg class="w-6 h-6 text-verusidx-turquoise-deep dark:text-verusidx-turquoise-light mr-2" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd"></path>
                  </svg>
                  <span class="text-verusidx-turquoise-deep dark:text-verusidx-turquoise-light font-medium">Sub-Identity Required</span>
                </div>
                <p class="text-verusidx-turquoise-deep dark:text-verusidx-turquoise-light text-sm mb-3">
                  You need a VerusIDX sub-identity to access marketplace features.
                </p>
                <a 
                  href="/onboard-subid" 
                  class="inline-block px-6 py-2 bg-verusidx-mountain-blue dark:bg-verusidx-turquoise-deep text-white text-sm rounded-lg hover:bg-verusidx-lake-blue dark:hover:bg-verusidx-turquoise-bright transition-colors"
                >
                  Get VerusIDX Sub-Identity
                </a>
              </div>
              <button 
                class="px-8 py-3 bg-verusidx-mountain-mist text-white rounded-lg cursor-not-allowed opacity-50"
                disabled
                title="Requires valid VerusIDX sub-identity"
              >
                Dashboard Access Restricted
              </button>
            {/if}
          </div>
      </div>
    {:else}
      <!-- Connection Form -->
      <div class="flex-1">
        <div class="min-h-full flex items-center justify-center p-8">
        <div class="bg-verusidx-snow-ice dark:bg-verusidx-stone-dark rounded-2xl shadow-xl p-8 max-w-2xl w-full border border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
          <h1 class="text-4xl font-bold text-center text-verusidx-stone-dark dark:text-white mb-8">
            Connect to Verus PBaaS
          </h1>

          <p class="text-center text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-8">
            Select a local daemon to use VerusIDX
          </p>

    {#if isDiscoveringChains}
      <div class="text-center mb-8">
            <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-verusidx-mountain-blue"></div>
            <p class="mt-2 text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Discovering available chains...</p>
      </div>
    {:else if availableChains.length > 0}
      <div class="mb-8">
            <h3 class="text-lg font-semibold mb-4 text-verusidx-stone-dark dark:text-white">Available Chains</h3>
        <div class="space-y-3">
          {#each availableChains as chain}
            <button
              onclick={() => connectToChain(chain)}
              disabled={connectionState.isConnecting}
              class="w-full p-4 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg hover:bg-verusidx-mountain-mist/20 dark:hover:bg-verusidx-mountain-blue/20 disabled:opacity-50 text-left flex justify-between items-center transition-all duration-200"
            >
              <div>
                  <div class="font-medium text-verusidx-stone-dark dark:text-white">{chain.name}</div>
                  <div class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">{chain.credentials.host}:{chain.credentials.port}</div>
              </div>
              <div class="text-sm {chain.is_active ? 'text-verusidx-turquoise-bright' : 'text-verusidx-mountain-mist'}">
                {chain.is_active ? '● Active' : '○ Inactive'}
              </div>
            </button>
          {/each}
        </div>
        
        <button
          onclick={() => showManualEntry = !showManualEntry}
          class="mt-4 text-verusidx-turquoise-light hover:text-verusidx-turquoise-bright text-sm"
        >
          {showManualEntry ? 'Hide' : 'Show'} manual connection
        </button>
      </div>
    {/if}

    {#if showManualEntry}
      <form onsubmit={handleManualConnect} class="space-y-4">
            <div class="flex justify-between items-center mb-4">
              <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">Manual Connection</h3>
              {#if hasSavedCredentials}
                <button
                  type="button"
                  onclick={forgetCredentials}
                  class="text-sm text-verusidx-lake-deep dark:text-verusidx-turquoise-light hover:underline flex items-center gap-1"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"></path>
                  </svg>
                  Forget Saved Credentials
                </button>
              {/if}
            </div>

        {#if showRemoteWarning}
          <div class="mb-4 p-4 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-300 dark:border-yellow-700 rounded-lg">
            <div class="flex items-start gap-3">
              <svg class="w-6 h-6 text-yellow-600 dark:text-yellow-500 flex-shrink-0 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd"></path>
              </svg>
              <div class="flex-1">
                <h4 class="font-semibold text-yellow-800 dark:text-yellow-200 mb-2">Remote Connection Warning</h4>
                <p class="text-sm text-yellow-700 dark:text-yellow-300 mb-2">
                  You are connecting to a remote daemon. Be aware:
                </p>
                <ul class="text-sm text-yellow-700 dark:text-yellow-300 list-disc list-inside space-y-1 mb-3">
                  <li>Credentials are sent over unencrypted HTTP</li>
                  <li>The remote node operator can see all your queries</li>
                  <li>Your addresses, balances, and transactions are visible to the node</li>
                </ul>
                <p class="text-sm text-yellow-700 dark:text-yellow-300 mb-3">
                  <strong>Only connect to:</strong> Your own server, SSH tunnels, or trusted networks.
                </p>
                <div class="flex gap-2">
                  <button
                    type="submit"
                    class="px-4 py-2 bg-yellow-600 hover:bg-yellow-700 text-white rounded-lg text-sm font-medium transition-colors"
                  >
                    I Understand, Connect Anyway
                  </button>
                  <button
                    type="button"
                    onclick={() => showRemoteWarning = false}
                    class="px-4 py-2 bg-verusidx-snow-ice dark:bg-verusidx-stone-medium text-verusidx-stone-dark dark:text-white rounded-lg text-sm font-medium hover:bg-verusidx-mountain-mist dark:hover:bg-verusidx-stone-dark transition-colors"
                  >
                    Cancel
                  </button>
                </div>
              </div>
            </div>
          </div>
        {/if}

        <div class="grid grid-cols-2 gap-4">
          <div>
              <label for="host" class="block text-sm font-medium text-verusidx-stone-medium dark:text-verusidx-mountain-mist mb-1">Host</label>
            <input
              type="text"
              id="host"
              bind:value={formData.host}
              class="w-full px-3 py-2 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white rounded-lg focus:ring-2 focus:ring-verusidx-mountain-blue focus:border-transparent outline-none transition-colors"
              required
            />
          </div>
          
          <div>
              <label for="port" class="block text-sm font-medium text-verusidx-stone-medium dark:text-verusidx-mountain-mist mb-1">Port</label>
            <input
              type="number"
              id="port"
              bind:value={formData.port}
              class="w-full px-3 py-2 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white rounded-lg focus:ring-2 focus:ring-verusidx-mountain-blue focus:border-transparent outline-none transition-colors"
              required
            />
          </div>
        </div>
        
        <div>
            <label for="username" class="block text-sm font-medium text-verusidx-stone-medium dark:text-verusidx-mountain-mist mb-1">RPC Username</label>
          <input
            type="text"
            id="username"
            bind:value={formData.username}
            class="w-full px-3 py-2 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white rounded-lg focus:ring-2 focus:ring-verusidx-mountain-blue focus:border-transparent outline-none transition-colors"
            required
          />
        </div>
        
        <div>
            <label for="password" class="block text-sm font-medium text-verusidx-stone-medium dark:text-verusidx-mountain-mist mb-1">RPC Password</label>
          <input
            type="password"
            id="password"
            bind:value={formData.password}
            class="w-full px-3 py-2 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white rounded-lg focus:ring-2 focus:ring-verusidx-mountain-blue focus:border-transparent outline-none transition-colors"
            required
          />
        </div>
        
        <div class="flex items-center">
          <input
            type="checkbox"
            id="remember"
            bind:checked={formData.rememberCredentials}
            class="h-4 w-4 text-verusidx-mountain-blue focus:ring-verusidx-mountain-blue border-gray-300 rounded"
          />
            <label for="remember" class="ml-2 block text-sm text-verusidx-stone-medium dark:text-verusidx-mountain-mist">
              Remember credentials
            </label>
        </div>
        
        <button
          type="submit"
          disabled={connectionState.isConnecting}
          class="w-full px-6 py-3 bg-verusidx-mountain-blue dark:bg-verusidx-turquoise-deep text-white rounded-lg hover:bg-verusidx-lake-blue dark:hover:bg-verusidx-turquoise-bright focus:ring-2 focus:ring-verusidx-mountain-blue dark:focus:ring-verusidx-turquoise-deep focus:ring-offset-2 transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {connectionState.isConnecting ? 'Connecting...' : 'Connect Remotely'}
        </button>
      </form>
    {/if}

    {#if connectionState.lastError}
          <div class="mt-4 p-4 bg-verusidx-lake-deep/10 dark:bg-verusidx-lake-deep/30 border border-verusidx-turquoise-light dark:border-verusidx-stone-medium rounded-lg">
            <p class="text-verusidx-lake-deep dark:text-verusidx-turquoise-light text-sm">{connectionState.lastError}</p>
          </div>
    {/if}

        </div>
        </div>
      </div>
    {/if}
  </div>
</div>

