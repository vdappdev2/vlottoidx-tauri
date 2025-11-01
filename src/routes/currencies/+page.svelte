<script lang="ts">
  import { connectionStore } from "$lib/stores/connection";
  import { goto } from "$app/navigation";
  import { 
    BlockHeightHeader, 
    CurrencySearchCard, 
    CurrencyListCard
  } from "$lib/components";
  
  // Import the type selector and specific modals
  import CurrencyTypeSelector from "$lib/components/currency/CurrencyTypeSelector.svelte";
  import DecentralizedSimpleTokenModal from "$lib/components/currency/DecentralizedSimpleTokenModal.svelte";
  import DecentralizedBasketModal from "$lib/components/currency/DecentralizedBasketModal.svelte";
  import CentralizedSimpleTokenModal from "$lib/components/currency/CentralizedSimpleTokenModal.svelte";
  import CentralizedBasketModal from "$lib/components/currency/CentralizedBasketModal.svelte";
  import EthereumMappedModal from "$lib/components/currency/EthereumMappedModal.svelte";
  import IdControlTokenModal from "$lib/components/currency/IdControlTokenModal.svelte";
  import AdvancedCurrencyModal from "$lib/components/currency/AdvancedCurrencyModal.svelte";
  import DefiOperationModal from "$lib/components/DefiOperationModal.svelte";

  // Modal state management - two-stage flow
  let isTypeSelectorOpen = $state(false);
  let selectedCurrencyClass = $state<'decentralized' | 'centralized'>('decentralized');
  
  // Specific modal states
  let isDecentralizedSimpleModalOpen = $state(false);
  let isDecentralizedBasketModalOpen = $state(false);
  let isCentralizedSimpleModalOpen = $state(false);
  let isCentralizedBasketModalOpen = $state(false);
  let isEthereumMappedModalOpen = $state(false);
  let isIdControlTokenModalOpen = $state(false);
  let isAdvancedModalOpen = $state(false);
  let isExportModalOpen = $state(false);

  connectionStore.subscribe(state => {
    if (!state.current?.isConnected) {
      goto("/");
    }
  });

  // Modal opening functions - two-stage flow
  function openDecentralizedModal() {
    selectedCurrencyClass = 'decentralized';
    isTypeSelectorOpen = true;
  }

  function openCentralizedModal() {
    selectedCurrencyClass = 'centralized';
    isTypeSelectorOpen = true;
  }

  function openEthereumMappedModal() {
    isEthereumMappedModalOpen = true;
  }

  function openIdControlTokenModal() {
    isIdControlTokenModalOpen = true;
  }

  function openAdvancedModal() {
    isAdvancedModalOpen = true;
  }

  function openExportModal() {
    isExportModalOpen = true;
  }

  // Type selector handlers
  function handleTypeSelection(type: 'simple' | 'basket') {
    if (selectedCurrencyClass === 'decentralized') {
      if (type === 'simple') {
        isDecentralizedSimpleModalOpen = true;
      } else {
        isDecentralizedBasketModalOpen = true;
      }
    } else {
      if (type === 'simple') {
        isCentralizedSimpleModalOpen = true;
      } else {
        isCentralizedBasketModalOpen = true;
      }
    }
  }

  // Modal closing functions
  function closeTypeSelector() {
    isTypeSelectorOpen = false;
  }

  function closeDecentralizedSimpleModal() {
    isDecentralizedSimpleModalOpen = false;
  }

  function closeDecentralizedBasketModal() {
    isDecentralizedBasketModalOpen = false;
  }

  function closeCentralizedSimpleModal() {
    isCentralizedSimpleModalOpen = false;
  }

  function closeCentralizedBasketModal() {
    isCentralizedBasketModalOpen = false;
  }

  function closeEthereumMappedModal() {
    isEthereumMappedModalOpen = false;
  }

  function closeIdControlTokenModal() {
    isIdControlTokenModalOpen = false;
  }

  function closeAdvancedModal() {
    isAdvancedModalOpen = false;
  }

  function closeExportModal() {
    isExportModalOpen = false;
  }

  function handleCreationSuccess() {
    // Currency list should refresh itself - close all modals
    isTypeSelectorOpen = false;
    isDecentralizedSimpleModalOpen = false;
    isDecentralizedBasketModalOpen = false;
    isCentralizedSimpleModalOpen = false;
    isCentralizedBasketModalOpen = false;
    isEthereumMappedModalOpen = false;
    isIdControlTokenModalOpen = false;
    isAdvancedModalOpen = false;
    isExportModalOpen = false;
  }
</script>

<svelte:head>
  <title>Currency Management - VerusIDX</title>
  <meta name="description" content="Search and manage currencies on the Verus network" />
</svelte:head>

<div class="bg-verusidx-sky-soft dark:bg-verusidx-lake-deep min-h-screen max-h-screen overflow-y-auto">
  <!-- Block Height Header -->
  <BlockHeightHeader />

  <div class="max-w-7xl mx-auto p-8 pb-16 space-y-8">
    <!-- Page Header -->
    <div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl p-6 mb-8">
      <div class="flex justify-between items-center">
        <div>
          <h1 class="text-3xl font-bold text-verusidx-stone-dark dark:text-white">Currency Management</h1>
          <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
            Search and explore currencies on the network
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

    <!-- Currency Search Section -->
    <div class="mb-8">
      <CurrencySearchCard />
    </div>

    <!-- Currency List Section -->
    <div class="mb-8">
      <CurrencyListCard />
    </div>

    <!-- Currency Creation Section -->
    <div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl p-6">
      <div class="text-center mb-8">
        <div class="text-verusidx-forest-deep dark:text-verusidx-turquoise-light mb-4">
          <svg class="h-16 w-16 mx-auto" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1" />
          </svg>
        </div>
        <h3 class="text-2xl font-bold text-verusidx-stone-dark dark:text-white mb-2">Create New Currency</h3>
        <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-6">
          Choose the type of currency you want to define
        </p>
      </div>
      
      <!-- Currency Creation Cards Grid -->
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 mb-6">
        <!-- Decentralized Currency Card -->
        <button
          onclick={openDecentralizedModal}
          class="group p-6 bg-verusidx-mountain-blue hover:bg-verusidx-lake-blue text-white rounded-lg shadow-lg hover:shadow-xl transition-all duration-200 transform hover:scale-105 text-left"
        >
          <div class="text-3xl mb-3 group-hover:scale-110 transition-transform">
            🪙
          </div>
          <h4 class="text-lg font-semibold mb-2">Decentralized Currency</h4>
          <p class="text-sm opacity-90">
            Static supply currency - no minting after launch. Choose between simple token or basket currency.
          </p>
        </button>

        <!-- Centralized Currency Card -->
        <button
          onclick={openCentralizedModal}
          class="group p-6 bg-verusidx-turquoise-deep hover:bg-verusidx-turquoise-bright text-white rounded-lg shadow-lg hover:shadow-xl transition-all duration-200 transform hover:scale-105 text-left"
        >
          <div class="text-3xl mb-3 group-hover:scale-110 transition-transform">
            🏦
          </div>
          <h4 class="text-lg font-semibold mb-2">Centralized Currency</h4>
          <p class="text-sm opacity-90">
            Root identity can mint and burn supply. Optional end block to become decentralized later.
          </p>
        </button>

        <!-- Ethereum-Mapped Currency Card -->
        <button
          onclick={openEthereumMappedModal}
          class="group p-6 bg-verusidx-forest-deep hover:bg-verusidx-turquoise-deep text-white rounded-lg shadow-lg hover:shadow-xl transition-all duration-200 transform hover:scale-105 text-left"
        >
          <div class="text-3xl mb-3 group-hover:scale-110 transition-transform">
            🌉
          </div>
          <h4 class="text-lg font-semibold mb-2">Ethereum-Mapped Currency</h4>
          <p class="text-sm opacity-90">
            Map existing ERC-20, ERC-721, or ERC-1155 tokens to Verus via the bridge.
          </p>
        </button>

        <!-- ID Control Token Card -->
        <button
          onclick={openIdControlTokenModal}
          class="group p-6 bg-purple-900 hover:bg-purple-800 text-white rounded-lg shadow-lg hover:shadow-xl transition-all duration-200 transform hover:scale-105 text-left"
        >
          <div class="text-3xl mb-3 group-hover:scale-110 transition-transform">
            👑
          </div>
          <h4 class="text-lg font-semibold mb-2">ID Control Token</h4>
          <p class="text-sm opacity-90">
            Single satoshi token that grants revoke/recover authority over the currency's root identity.
          </p>
        </button>

        <!-- Export Currency Definition Card -->
        <button
          onclick={openExportModal}
          class="group p-6 bg-verusidx-forest-deep hover:bg-verusidx-turquoise-deep text-white rounded-lg shadow-lg hover:shadow-xl transition-all duration-200 transform hover:scale-105 text-left"
        >
          <div class="text-3xl mb-3 group-hover:scale-110 transition-transform">
            📋
          </div>
          <h4 class="text-lg font-semibold mb-2">Export Currency Definition</h4>
          <p class="text-sm opacity-90">
            Export currency definition to another chain.
          </p>
        </button>

        <!-- Advanced Mode Card -->
        <button
          onclick={openAdvancedModal}
          class="group p-6 bg-verusidx-stone-dark hover:bg-verusidx-mountain-grey text-white rounded-lg shadow-lg hover:shadow-xl transition-all duration-200 transform hover:scale-105 text-left"
        >
          <div class="text-3xl mb-3 group-hover:scale-110 transition-transform">
            🔧
          </div>
          <h4 class="text-lg font-semibold mb-2">Advanced Mode</h4>
          <p class="text-sm opacity-90">
            Raw JSON interface with full definecurrency parameter control for experts.
          </p>
        </button>
      </div>

      <!-- Cost Information -->
      <div class="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
        <p class="text-yellow-700 dark:text-yellow-300 text-sm">
          💰 <strong>Cost:</strong> Ensure your identity has sufficient funds before proceeding.
        </p>
      </div>
    </div>

  </div>
</div>

<!-- Currency Creation Modals - Two-stage flow -->

<!-- Stage 1: Type Selector -->
{#if isTypeSelectorOpen}
  <CurrencyTypeSelector
    isOpen={isTypeSelectorOpen}
    onClose={closeTypeSelector}
    onSelect={handleTypeSelection}
    currencyClass={selectedCurrencyClass}
  />
{/if}

<!-- Stage 2: Specific Currency Modals -->
{#if isDecentralizedSimpleModalOpen}
  <DecentralizedSimpleTokenModal
    isOpen={isDecentralizedSimpleModalOpen}
    onClose={closeDecentralizedSimpleModal}
    onSuccess={handleCreationSuccess}
  />
{/if}

{#if isDecentralizedBasketModalOpen}
  <DecentralizedBasketModal
    isOpen={isDecentralizedBasketModalOpen}
    onClose={closeDecentralizedBasketModal}
    onSuccess={handleCreationSuccess}
  />
{/if}

{#if isCentralizedSimpleModalOpen}
  <CentralizedSimpleTokenModal
    isOpen={isCentralizedSimpleModalOpen}
    onClose={closeCentralizedSimpleModal}
    onSuccess={handleCreationSuccess}
  />
{/if}

{#if isCentralizedBasketModalOpen}
  <CentralizedBasketModal
    isOpen={isCentralizedBasketModalOpen}
    onClose={closeCentralizedBasketModal}
    onSuccess={handleCreationSuccess}
  />
{/if}

{#if isEthereumMappedModalOpen}
  <EthereumMappedModal
    isOpen={isEthereumMappedModalOpen}
    onClose={closeEthereumMappedModal}
    onSuccess={handleCreationSuccess}
  />
{/if}

{#if isIdControlTokenModalOpen}
  <IdControlTokenModal
    isOpen={isIdControlTokenModalOpen}
    onClose={closeIdControlTokenModal}
    onSuccess={handleCreationSuccess}
  />
{/if}

{#if isAdvancedModalOpen}
  <AdvancedCurrencyModal
    isOpen={isAdvancedModalOpen}
    onClose={closeAdvancedModal}
    onSuccess={handleCreationSuccess}
  />
{/if}

{#if isExportModalOpen}
  <DefiOperationModal
    isOpen={isExportModalOpen}
    operationType="export-definition"
    onClose={closeExportModal}
  />
{/if}