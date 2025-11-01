<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { connectionStore, getChainParam, type ConnectionState } from "$lib/stores/connection";
  import { goto } from "$app/navigation";
  import { onMount, onDestroy } from "svelte";
  import { 
    BlockHeightHeader, 
    AddressCard, 
    RAddressListCard,
    AddressDetailsModal, 
    TransactionCard, 
    OperationCard,
    CurrenciesCard
  } from "$lib/components";
  import { Modal } from "$lib/components/cards";

  let connectionState = $state<ConnectionState>({ current: null, isConnecting: false, lastError: null, availableChains: [], selectedChain: null });
  let walletInfo = $state<any>(null);
  let transparentAddresses = $state<any[]>([]);
  let privateAddresses = $state<string[]>([]);
  let transactions = $state<any[]>([]);
  let operations = $state<any[]>([]);
  let isLoading = $state(true);
  let isGeneratingAddress = $state(false);
  let errorMessage = $state('');
  let loadingErrors = $state<string[]>([]);
  
  // Modal state
  let showAddressDetails = $state(false);
  let selectedAddress = $state('');
  let showNewAddressModal = $state(false);
  let newGeneratedAddress = $state('');
  let newAddressType = $state<'transparent' | 'private'>('transparent');

  connectionStore.subscribe(state => {
    connectionState = state;
    if (!state.current?.isConnected) {
      goto("/");
    }
  });

  async function loadWalletData() {
    if (!connectionState.current?.isConnected) return;

    isLoading = true;
    errorMessage = '';
    loadingErrors = [];

    try {
      // Use the chain parameter helper to get correct chain value
      const chainParam = getChainParam(connectionState.selectedChain);
      console.log("Wallet loading data for chain:", connectionState.selectedChain, "param:", chainParam);

      // Load wallet info for currency balance summary
      try {
        const walletInfoResult = await invoke("get_wallet_info", { chain: chainParam });
        walletInfo = walletInfoResult;
      } catch (error) {
        loadingErrors.push(`Failed to load wallet info: ${error}`);
        console.error("Failed to load wallet info:", error);
      }

      // Load transparent addresses
      try {
        const addressGroupings = await invoke("list_address_groupings", { chain: chainParam });
        transparentAddresses = [];
        
        // listaddressgroupings returns groups of address objects: [[{address, amount, account}, ...], ...]
        // Each top-level array contains groups, each group contains address objects
        if (Array.isArray(addressGroupings)) {
          for (let i = 0; i < addressGroupings.length; i++) {
            const group = addressGroupings[i];
            
            if (Array.isArray(group)) {
              for (let j = 0; j < group.length; j++) {
                const addressEntry = group[j];
                
                // addressEntry is {address, amount, account}
                if (addressEntry && typeof addressEntry === 'object' && addressEntry.address) {
                  transparentAddresses.push(addressEntry);
                }
              }
            }
          }
        }
      } catch (error) {
        loadingErrors.push(`Failed to load transparent addresses: ${error}`);
        console.error("Failed to load transparent addresses:", error);
      }

      // Load private addresses
      try {
        const privAddresses = await invoke("z_list_addresses", { 
          include_watchonly: false, 
          chain: chainParam 
        });
        privateAddresses = privAddresses as string[];
      } catch (error) {
        loadingErrors.push(`Failed to load private addresses: ${error}`);
        console.error("Failed to load private addresses:", error);
      }

      // Load recent transactions
      try {
        const recentTxs = await invoke("list_transactions", {
          account: null,
          count: 20,
          from: null,
          include_watchonly: false,
          chain: chainParam
        });
        transactions = recentTxs as any[];
      } catch (error) {
        loadingErrors.push(`Failed to load transactions: ${error}`);
        console.error("Failed to load transactions:", error);
      }

      // Load pending operations
      try {
        const pendingOps = await invoke("z_get_operation_status", {
          operation_ids: null,
          chain: chainParam
        });
        operations = pendingOps as any[];
      } catch (error) {
        loadingErrors.push(`Failed to load operations: ${error}`);
        console.error("Failed to load operations:", error);
      }

    } catch (error) {
      errorMessage = `Failed to load wallet data: ${error}`;
      console.error("Failed to load wallet data:", error);
    } finally {
      isLoading = false;
    }
  }

  async function generateNewAddress(type: 'transparent' | 'private') {
    if (!connectionState.current?.isConnected) return;

    isGeneratingAddress = true;
    try {
      const chainParam = getChainParam(connectionState.selectedChain);
      let newAddress: string;
      
      if (type === 'transparent') {
        newAddress = await invoke("get_new_address", { account: null, chain: chainParam });
      } else {
        newAddress = await invoke("z_get_new_address", { address_type: null, chain: chainParam });
      }
      
      // Store the new address and type, then show modal
      newGeneratedAddress = newAddress;
      newAddressType = type;
      showNewAddressModal = true;
      
      // Only reload the specific address list to avoid closing modals
      if (type === 'private') {
        try {
          const privAddresses = await invoke("z_list_addresses", { 
            include_watchonly: false, 
            chain: chainParam 
          });
          privateAddresses = privAddresses as string[];
        } catch (error) {
          console.error("Failed to reload private addresses:", error);
        }
      } else {
        // For transparent addresses, reload address groupings
        try {
          const addressGroupings = await invoke("list_address_groupings", { chain: chainParam });
          transparentAddresses = [];
          
          if (Array.isArray(addressGroupings)) {
            for (let i = 0; i < addressGroupings.length; i++) {
              const group = addressGroupings[i];
              
              if (Array.isArray(group)) {
                for (let j = 0; j < group.length; j++) {
                  const addressEntry = group[j];
                  
                  if (addressEntry && typeof addressEntry === 'object' && addressEntry.address) {
                    transparentAddresses.push(addressEntry);
                  }
                }
              }
            }
          }
        } catch (error) {
          console.error("Failed to reload transparent addresses:", error);
        }
      }
    } catch (error) {
      errorMessage = `Failed to generate ${type} address: ${error}`;
      console.error("Failed to generate address:", error);
    } finally {
      isGeneratingAddress = false;
    }
  }

  function handleAddressClick(address: string) {
    selectedAddress = address;
    showAddressDetails = true;
  }

  function closeAddressDetails() {
    showAddressDetails = false;
    selectedAddress = '';
  }

  function closeNewAddressModal() {
    showNewAddressModal = false;
    newGeneratedAddress = '';
  }

  async function copyToClipboard(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      // Could add a toast notification here
      console.log("Copied to clipboard:", text);
    } catch (err) {
      console.error("Failed to copy:", err);
    }
  }

  // Chain change event handler for global chain switching
  function handleChainChanged(event: CustomEvent) {
    console.log("Wallet: Chain changed to", event.detail.chainName);
    loadWalletData();
  }

  // Load wallet data when component mounts and connection is available
  $effect(() => {
    if (connectionState.current?.isConnected && connectionState.selectedChain) {
      loadWalletData();
    }
  });

  // Set up chain change event listener
  onMount(() => {
    window.addEventListener('chainChanged', handleChainChanged);
  });

  onDestroy(() => {
    window.removeEventListener('chainChanged', handleChainChanged);
  });

</script>

<div class="bg-verusidx-sky-soft dark:bg-verusidx-lake-deep min-h-screen max-h-screen overflow-y-auto">
  <!-- Block Height Header -->
  <BlockHeightHeader />

  <div class="max-w-7xl mx-auto p-8 pb-16 space-y-8">
    <!-- Page Header -->
    <div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl p-6">
      <div class="flex justify-between items-center">
        <div>
          <h1 class="text-3xl font-bold text-verusidx-stone-dark dark:text-white">Wallet</h1>
          <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
            Manage your addresses, transactions, and currency balances
          </p>
        </div>
        <a 
          href="/dashboard"
          class="px-4 py-2 bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-stone-medium dark:text-verusidx-mountain-mist rounded-lg hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-dark transition-colors"
        >
          Back to Dashboard
        </a>
      </div>
    </div>

    <!-- Error Messages -->
    {#if errorMessage}
      <div class="bg-verusidx-lake-deep/10 dark:bg-verusidx-lake-deep/30 border border-verusidx-turquoise-light dark:border-verusidx-stone-medium rounded-lg p-4">
        <p class="text-verusidx-lake-deep dark:text-verusidx-turquoise-light">{errorMessage}</p>
      </div>
    {/if}

    {#if loadingErrors.length > 0}
      <div class="bg-verusidx-turquoise-bright/10 dark:bg-verusidx-turquoise-deep/30 border border-verusidx-turquoise-light dark:border-verusidx-stone-medium rounded-lg p-4">
        <h3 class="text-sm font-medium text-verusidx-turquoise-deep dark:text-verusidx-turquoise-light mb-2">Loading Issues:</h3>
        <ul class="text-sm text-verusidx-turquoise-deep dark:text-verusidx-turquoise-light space-y-1">
          {#each loadingErrors as error}
            <li>• {error}</li>
          {/each}
        </ul>
      </div>
    {/if}

    {#if isLoading}
      <div class="text-center py-12">
        <div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-verusidx-mountain-blue"></div>
        <p class="mt-4 text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Loading wallet data...</p>
      </div>
    {:else}
      <!-- Top Section: Currency Balance Summary and R-address List -->
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
        <!-- Currency Balance Summary -->
        <CurrenciesCard 
          {walletInfo}
          cardClass="bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
        />

        <!-- R-address List -->
        <RAddressListCard />
      </div>

      <!-- Address Management and Transactions Grid -->
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
        <!-- Transparent Address Balances -->
        <AddressCard
          title="Transparent Address Balances"
          type="transparent"
          addresses={transparentAddresses}
          onAddressClick={handleAddressClick}
          cardClass="bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
        />

        <!-- Z-address & Private Balances -->
        <AddressCard
          title="Z-address & Private Balances"
          type="private"
          addresses={privateAddresses}
          onAddressClick={handleAddressClick}
          onGenerateNew={() => generateNewAddress('private')}
          isGenerating={isGeneratingAddress}
          cardClass="bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
        />
      </div>

      <!-- Transaction History and Operations -->
      <div class="space-y-8 mb-16">
        <h2 class="text-2xl font-bold text-verusidx-stone-dark dark:text-white mb-4">Transaction History & Operations</h2>
        
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
          <!-- Recent Transactions -->
          <TransactionCard
            {transactions}
            cardClass="bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
          />

          <!-- Pending Operations -->
          <OperationCard
            {operations}
            cardClass="bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
          />
        </div>
      </div>
    {/if}
  </div>
</div>

<!-- Address Details Modal -->
<AddressDetailsModal
  address={selectedAddress}
  isOpen={showAddressDetails}
  onClose={closeAddressDetails}
/>

<!-- New Address Generated Modal -->
<Modal isOpen={showNewAddressModal} title="New {newAddressType === 'transparent' ? 'R-Address' : 'Z-Address'} Generated!" size="md" onclose={closeNewAddressModal}>
  <div class="space-y-4">
    <div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-4 rounded-lg">
      <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-2">Your new {newAddressType} address:</p>
      <p class="font-mono text-sm text-verusidx-stone-dark dark:text-white break-all select-all">
        {newGeneratedAddress}
      </p>
    </div>
    
    <div class="flex gap-3 justify-end">
      <button
        onclick={() => copyToClipboard(newGeneratedAddress)}
        class="px-4 py-2 bg-verusidx-turquoise-deep text-white rounded-lg hover:bg-verusidx-turquoise-bright transition-colors"
      >
        Copy Address
      </button>
      <button
        onclick={closeNewAddressModal}
        class="px-4 py-2 bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-stone-medium dark:text-verusidx-mountain-mist rounded-lg hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-dark transition-colors"
      >
        Close
      </button>
    </div>
  </div>
</Modal>