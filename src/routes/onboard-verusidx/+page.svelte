<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { connectionStore, getChainParam, type ConnectionState } from "$lib/stores/connection";
  import { goto } from "$app/navigation";
  import { onMount, onDestroy } from "svelte";
  import { BlockHeightHeader, OnboardingConversionModal } from "$lib/components";
  import EstimateConversionModal from "$lib/components/EstimateConversionModal.svelte";

  let connectionState = $state<ConnectionState>({ current: null, isConnecting: false, lastError: null, availableChains: [], selectedChain: null });

  // Balance state
  let currentVerusIDX = $state(0);
  let currentVRSC = $state(0);
  let currentVUSDC = $state(0);
  let isLoadingBalances = $state(false);

  // Modal state
  let showEstimateModal = $state(false);
  let showConversionModal = $state(false);

  // Error state
  let errorMessage = $state('');

  // Auto-refresh interval
  let balanceCheckInterval = $state<number | null>(null);

  // Configuration
  const REQUIRED_VERUSIDX = 30;
  const CURRENCY_NAME = "VerusIDX";

  connectionStore.subscribe(state => {
    connectionState = state;
    if (!state.current?.isConnected) {
      goto("/");
    }
  });

  // Disconnect and go home function
  function disconnectAndGoHome() {
    connectionStore.update(() => ({
      current: null,
      isConnecting: false,
      lastError: null,
      availableChains: [],
      selectedChain: null
    }));
    goto("/");
  }

  async function checkBalances() {
    if (!connectionState.current?.isConnected) return;

    isLoadingBalances = true;
    errorMessage = '';

    try {
      // Get chain parameter for RPC calls
      const chainParam = getChainParam(connectionState.selectedChain);

      // Get wallet info for the selected chain
      const walletInfo = await invoke("get_wallet_info", { chain: chainParam });

      // Extract current balances
      currentVerusIDX = walletInfo.reserve_balance?.[CURRENCY_NAME] || 0;
      currentVUSDC = walletInfo.reserve_balance?.["vUSDC.vETH"] || 0;
      currentVRSC = walletInfo.balance || 0;

      console.log(`Current balances - ${CURRENCY_NAME}: ${currentVerusIDX}, VRSC: ${currentVRSC}, vUSDC.vETH: ${currentVUSDC}`);

    } catch (error) {
      errorMessage = `Failed to check balances: ${error}`;
      console.error("Failed to check balances:", error);
    } finally {
      isLoadingBalances = false;
    }
  }

  // Auto-check balances on mount and every 15 seconds
  onMount(() => {
    checkBalances();

    balanceCheckInterval = setInterval(() => {
      checkBalances();
    }, 15000); // 15 seconds
  });

  // Cleanup interval on unmount
  onDestroy(() => {
    if (balanceCheckInterval) {
      clearInterval(balanceCheckInterval);
      balanceCheckInterval = null;
    }
  });

  // Check balances when connection changes
  $effect(() => {
    if (connectionState.current?.isConnected && connectionState.selectedChain) {
      checkBalances();
    }
  });

  function closeConversionModal() {
    showConversionModal = false;
  }

  function handleConversionSuccess() {
    // Refresh balances after conversion
    checkBalances();
  }
</script>

<div class="min-h-screen max-h-screen overflow-y-auto bg-verusidx-sky-soft dark:bg-verusidx-lake-deep">
  <!-- Block Height Header -->
  <BlockHeightHeader />

  <div class="max-w-6xl mx-auto p-8">
    <!-- Header -->
    <div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl p-6 mb-8">
      <div class="flex justify-between items-center">
        <div>
          <h1 class="text-3xl font-bold text-verusidx-stone-dark dark:text-white">Get {CURRENCY_NAME}</h1>
          <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Convert VRSC or vUSDC.vETH to gain access</p>
        </div>
        <button
          onclick={disconnectAndGoHome}
          class="px-4 py-2 bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-stone-medium dark:text-verusidx-mountain-mist rounded-lg hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-dark transition-colors"
        >
          Back to Home
        </button>
      </div>
    </div>

    {#if errorMessage}
      <div class="bg-verusidx-lake-deep/10 dark:bg-verusidx-lake-deep/30 border border-verusidx-turquoise-light dark:border-verusidx-stone-medium rounded-lg p-4 mb-8">
        <p class="text-verusidx-lake-deep dark:text-verusidx-turquoise-light">{errorMessage}</p>
      </div>
    {/if}

    <!-- Progress Indicator -->
    <div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl p-6 mb-8">
      <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-4">Your Progress</h3>
      <div class="space-y-2">
        <div class="flex justify-between items-center">
          <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Current {CURRENCY_NAME}:</span>
          <span class="font-mono text-verusidx-stone-dark dark:text-white font-semibold">{currentVerusIDX.toFixed(8)}</span>
        </div>
        <div class="flex justify-between items-center">
          <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Required:</span>
          <span class="font-mono text-verusidx-stone-dark dark:text-white">{REQUIRED_VERUSIDX.toFixed(8)}</span>
        </div>
        <div class="flex justify-between items-center pt-2 border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
          <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Still need:</span>
          <span class="font-mono text-verusidx-lake-deep dark:text-verusidx-turquoise-light font-semibold">
            {Math.max(0, REQUIRED_VERUSIDX - currentVerusIDX).toFixed(8)}
          </span>
        </div>
        <!-- Progress bar -->
        <div class="mt-4">
          <div class="w-full bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium rounded-full h-3">
            <div
              class="bg-verusidx-turquoise-deep dark:bg-verusidx-turquoise-light rounded-full h-3 transition-all duration-500"
              style="width: {Math.min(100, (currentVerusIDX / REQUIRED_VERUSIDX) * 100)}%"
            ></div>
          </div>
          <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist text-center mt-1">
            {Math.min(100, Math.round((currentVerusIDX / REQUIRED_VERUSIDX) * 100))}% complete
          </p>
        </div>
      </div>
    </div>

    <!-- 3-Card Grid -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
      <!-- Card 1: View Balances -->
      <div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl p-6">
        <div class="flex items-center justify-between mb-4">
          <span class="text-xs font-semibold text-verusidx-mountain-grey dark:text-verusidx-mountain-mist uppercase tracking-wide">
            Step 1
          </span>
        </div>
        <h3 class="text-xl font-bold text-verusidx-stone-dark dark:text-white mb-2">
          View Balances
        </h3>
        <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-4">
          Check your current holdings
        </p>
        <div class="space-y-2 text-sm mb-4">
          <div class="flex justify-between items-center p-2 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium rounded">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">{CURRENCY_NAME}</span>
            <span class="font-mono text-verusidx-stone-dark dark:text-white">{currentVerusIDX.toFixed(8)}</span>
          </div>
          <div class="flex justify-between items-center p-2 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium rounded">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">VRSC</span>
            <span class="font-mono text-verusidx-stone-dark dark:text-white">{currentVRSC.toFixed(8)}</span>
          </div>
          <div class="flex justify-between items-center p-2 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium rounded">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">vUSDC.vETH</span>
            <span class="font-mono text-verusidx-stone-dark dark:text-white">{currentVUSDC.toFixed(8)}</span>
          </div>
        </div>
        <button
          onclick={checkBalances}
          disabled={isLoadingBalances}
          class="w-full px-4 py-2 bg-verusidx-mountain-blue text-white rounded-lg hover:bg-verusidx-lake-blue transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {isLoadingBalances ? 'Refreshing...' : 'Refresh Balances'}
        </button>
      </div>

      <!-- Card 2: Estimate Conversion -->
      <button
        onclick={() => showEstimateModal = true}
        class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl p-6 text-left hover:shadow-2xl hover:scale-[1.02] transition-all duration-200 border border-transparent hover:border-verusidx-turquoise-light dark:hover:border-verusidx-turquoise-deep"
      >
        <div class="flex items-center justify-between mb-4">
          <span class="text-xs font-semibold text-verusidx-mountain-grey dark:text-verusidx-mountain-mist uppercase tracking-wide">
            Step 2
          </span>
        </div>
        <h3 class="text-xl font-bold text-verusidx-stone-dark dark:text-white mb-2">
          Estimate Conversion
        </h3>
        <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
          Calculate how much {CURRENCY_NAME} you'll receive
        </p>
      </button>

      <!-- Card 3: Get VerusIDX -->
      <button
        onclick={() => showConversionModal = true}
        class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl p-6 text-left hover:shadow-2xl hover:scale-[1.02] transition-all duration-200 border border-transparent hover:border-verusidx-turquoise-light dark:hover:border-verusidx-turquoise-deep {currentVerusIDX >= REQUIRED_VERUSIDX ? 'ring-2 ring-verusidx-turquoise-light dark:ring-verusidx-turquoise-deep' : ''}"
      >
        <div class="flex items-center justify-between mb-4">
          <span class="text-xs font-semibold text-verusidx-mountain-grey dark:text-verusidx-mountain-mist uppercase tracking-wide">
            Step 3
          </span>
          {#if currentVerusIDX >= REQUIRED_VERUSIDX}
            <span class="px-2 py-1 bg-verusidx-forest-deep/20 text-verusidx-forest-deep dark:bg-verusidx-turquoise-deep/30 dark:text-verusidx-turquoise-light text-xs rounded font-semibold">
              Complete
            </span>
          {/if}
        </div>
        <h3 class="text-xl font-bold text-verusidx-stone-dark dark:text-white mb-2">
          Get {CURRENCY_NAME}
        </h3>
        <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
          Convert VRSC or vUSDC.vETH to {CURRENCY_NAME}
        </p>
      </button>
    </div>

    <!-- Access Granted Section -->
    {#if currentVerusIDX >= REQUIRED_VERUSIDX}
      <div class="bg-verusidx-forest-deep/10 dark:bg-verusidx-turquoise-deep/20 border border-verusidx-turquoise-light dark:border-verusidx-turquoise-bright rounded-lg p-6">
        <div class="flex items-center space-x-3 mb-4">
          <svg class="w-8 h-8 text-verusidx-forest-deep dark:text-verusidx-turquoise-light" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"></path>
          </svg>
          <div>
            <h3 class="text-lg font-semibold text-verusidx-forest-deep dark:text-verusidx-turquoise-light">
              Access Granted!
            </h3>
            <p class="text-verusidx-forest-deep dark:text-verusidx-turquoise-light">
              You have enough {CURRENCY_NAME} to access the marketplace.
            </p>
          </div>
        </div>
        <a
          href="/dashboard"
          class="block w-full px-6 py-3 bg-verusidx-mountain-blue dark:bg-verusidx-turquoise-deep text-white text-center rounded-lg hover:bg-verusidx-lake-blue dark:hover:bg-verusidx-turquoise-bright transition-colors font-semibold"
        >
          Continue to Dashboard
        </a>
      </div>
    {/if}
  </div>
</div>

<!-- Estimate Conversion Modal -->
<EstimateConversionModal
  isOpen={showEstimateModal}
  onClose={() => showEstimateModal = false}
/>

<!-- Conversion Modal -->
<OnboardingConversionModal
  isOpen={showConversionModal}
  onClose={closeConversionModal}
  onSuccess={handleConversionSuccess}
/>
