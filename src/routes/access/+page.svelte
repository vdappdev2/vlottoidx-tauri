<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { connectionStore, getChainParam, type ConnectionState } from "$lib/stores/connection";
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";

  let isLoading = $state(true);
  let hasEnoughVerusIDX = $state(false);
  let currentVerusIDX = $state(0);
  let conversionRates = $state<{ vrsc: number; vusdc: number }>({ vrsc: 0, vusdc: 0 });
  let connectionState = $state<ConnectionState>({ current: null, isConnecting: false, lastError: null, availableChains: [], selectedChain: null });

  // Configuration
  const REQUIRED_VERUSIDX = 30;
  const CURRENCY_NAME = "VerusIDX";

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
      // Get wallet info for the current chain
      const chainParam = getChainParam(connectionState.selectedChain);
      console.log("Access page loading wallet info for chain:", connectionState.selectedChain, "param:", chainParam);

      const walletInfo = await invoke("get_wallet_info", { chain: chainParam });

      // Extract VerusIDX balance from reserve_balance
      currentVerusIDX = walletInfo.reserve_balance?.[CURRENCY_NAME] || 0;

      console.log(`Current ${CURRENCY_NAME} balance:`, currentVerusIDX);

      // Check if user has enough VerusIDX
      hasEnoughVerusIDX = currentVerusIDX >= REQUIRED_VERUSIDX;

      // If user has enough VerusIDX, redirect to dashboard
      if (hasEnoughVerusIDX) {
        goto("/dashboard");
      } else {
        // Load conversion rates for display
        await loadConversionRates();
      }
    } catch (error) {
      console.error("Failed to check access:", error);
    } finally {
      isLoading = false;
    }
  }

  async function loadConversionRates() {
    try {
      // Get VerusIDX currency info to extract reserve pricing
      const currencyInfo = await invoke("get_currency", {
        currencyName: CURRENCY_NAME
      });

      // Extract reserve currencies and their prices
      const reserveCurrencies = currencyInfo.reservecurrencies || [];

      // Find VRSC and vUSDC.vETH in reserves
      const vrscCurrencyId = "i5w5MuNik5NtLcYmNzcvaoixooEebB6MGV";
      const vusdcCurrencyId = "iS8TfRPfVpKo5FVfSUzfHBQxo9KuzpnqLU";

      const vrscReserve = reserveCurrencies.find(c => c.currencyid === vrscCurrencyId);
      const vusdcReserve = reserveCurrencies.find(c => c.currencyid === vusdcCurrencyId);

      // Calculate how much VRSC and vUSDC needed for REQUIRED_VERUSIDX
      if (vrscReserve) {
        conversionRates.vrsc = REQUIRED_VERUSIDX * vrscReserve.priceinreserve;
      }

      if (vusdcReserve) {
        conversionRates.vusdc = REQUIRED_VERUSIDX * vusdcReserve.priceinreserve;
      }

      console.log("Conversion rates loaded:", conversionRates);
    } catch (error) {
      console.error("Failed to load conversion rates:", error);
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
    {:else if !hasEnoughVerusIDX}
      <div class="bg-white dark:bg-verusidx-stone-dark rounded-2xl shadow-xl p-8">
        <div class="text-center mb-8">
          <div class="w-16 h-16 bg-verusidx-turquoise-bright/20 dark:bg-verusidx-lake-deep/50 rounded-full flex items-center justify-center mx-auto mb-4">
            <svg class="w-8 h-8 text-verusidx-turquoise-deep dark:text-verusidx-turquoise-light" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd"/>
            </svg>
          </div>
          <h1 class="text-3xl font-bold text-verusidx-stone-dark dark:text-white mb-2">Access Required</h1>
          <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
            You need {REQUIRED_VERUSIDX} {CURRENCY_NAME} to access the app on {connectionState?.chainName || 'this chain'}.
          </p>
        </div>

        <div class="bg-verusidx-sky-soft dark:bg-verusidx-lake-deep/20 rounded-lg p-6 mb-8">
          <h3 class="font-semibold text-verusidx-stone-dark dark:text-white mb-3">Current Balance</h3>
          <div class="space-y-2">
            <div class="flex justify-between items-center">
              <span class="text-verusidx-stone-medium dark:text-verusidx-mountain-mist">Your {CURRENCY_NAME}:</span>
              <span class="font-mono text-verusidx-stone-dark dark:text-white">{currentVerusIDX.toFixed(8)}</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-verusidx-stone-medium dark:text-verusidx-mountain-mist">Required:</span>
              <span class="font-mono text-verusidx-stone-dark dark:text-white">{REQUIRED_VERUSIDX.toFixed(1)}</span>
            </div>
            <div class="flex justify-between items-center pt-2 border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
              <span class="text-verusidx-stone-medium dark:text-verusidx-mountain-mist">Needed:</span>
              <span class="font-mono text-verusidx-lake-deep dark:text-verusidx-turquoise-light font-semibold">
                {Math.max(0, REQUIRED_VERUSIDX - currentVerusIDX).toFixed(8)}
              </span>
            </div>
          </div>
        </div>

        {#if conversionRates.vrsc > 0 || conversionRates.vusdc > 0}
          <div class="bg-verusidx-sky-soft dark:bg-verusidx-lake-deep/20 rounded-lg p-6 mb-8">
            <h3 class="font-semibold text-verusidx-stone-dark dark:text-white mb-3">How to Get {CURRENCY_NAME}</h3>
            <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-4">
              Convert from VRSC or vUSDC.vETH to get the required {CURRENCY_NAME}:
            </p>
            <ul class="space-y-2 text-sm">
              {#if conversionRates.vrsc > 0}
                <li class="flex items-start">
                  <span class="text-verusidx-turquoise-deep dark:text-verusidx-turquoise-light mr-2">•</span>
                  <span class="text-verusidx-stone-medium dark:text-verusidx-mountain-mist">
                    ~{conversionRates.vrsc.toFixed(4)} VRSC → {REQUIRED_VERUSIDX} {CURRENCY_NAME}
                  </span>
                </li>
              {/if}
              {#if conversionRates.vusdc > 0}
                <li class="flex items-start">
                  <span class="text-verusidx-turquoise-deep dark:text-verusidx-turquoise-light mr-2">•</span>
                  <span class="text-verusidx-stone-medium dark:text-verusidx-mountain-mist">
                    ~{conversionRates.vusdc.toFixed(4)} vUSDC.vETH → {REQUIRED_VERUSIDX} {CURRENCY_NAME}
                  </span>
                </li>
              {/if}
            </ul>
          </div>
        {/if}

        <div class="space-y-3">
          <a
            href="/onboard-verusidx"
            class="block w-full px-6 py-3 bg-verusidx-mountain-blue dark:bg-verusidx-turquoise-deep text-white text-center rounded-lg hover:bg-verusidx-lake-blue dark:hover:bg-verusidx-turquoise-bright transition-colors"
          >
            Get {CURRENCY_NAME}
          </a>

          <button
            onclick={() => checkAccess()}
            class="block w-full px-6 py-3 bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-stone-medium dark:text-verusidx-mountain-mist text-center rounded-lg hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-light transition-colors"
          >
            Check Again
          </button>
        </div>
      </div>
    {/if}
    </div>
  </div>
</div>
