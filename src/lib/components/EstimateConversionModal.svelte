<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Modal } from './cards';
  import { connectionStore, getChainParam } from "$lib/stores/connection";

  interface Props {
    isOpen: boolean;
    onClose: () => void;
  }

  let { isOpen = false, onClose }: Props = $props();

  // Form state
  let currency = $state('');
  let amount = $state('');
  let estimateResult = $state<any>(null);
  let isEstimating = $state(false);
  let error = $state<string | null>(null);

  // Connection state
  let connectionState = $state<any>(null);

  connectionStore.subscribe(state => {
    connectionState = state;
  });

  // Reset form when modal closes
  $effect(() => {
    if (!isOpen) {
      resetForm();
    }
  });

  function resetForm() {
    currency = '';
    amount = '';
    estimateResult = null;
    error = null;
  }

  async function handleEstimate(event: SubmitEvent) {
    event.preventDefault();

    if (!currency || !amount || parseFloat(amount) <= 0) {
      error = 'Please select a currency and enter a valid amount';
      return;
    }

    isEstimating = true;
    error = null;
    estimateResult = null;

    try {
      const chainParam = getChainParam(connectionState?.selectedChain);
      const result = await invoke('estimate_conversion', {
        currency: currency,
        amount: parseFloat(amount),
        convertto: 'VerusIDX',
        via: null,
        chain: chainParam
      });

      estimateResult = result;
      console.log('Estimate result:', result);

    } catch (err) {
      error = typeof err === 'string' ? err : 'Failed to estimate conversion';
      console.error('Estimation error:', err);
    } finally {
      isEstimating = false;
    }
  }
</script>

<Modal {isOpen} onclose={onClose} title="Estimate Conversion" size="md">
  <div class="p-6">
    <div class="mb-6">
      <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
        Calculate how much VerusIDX you'll receive for your VRSC or vUSDC.vETH
      </p>
    </div>

    <!-- Error Display -->
    {#if error}
      <div class="mb-4 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
        <p class="text-red-700 dark:text-red-300">{error}</p>
      </div>
    {/if}

    <!-- Form -->
    <form onsubmit={handleEstimate} class="space-y-4">
      <!-- Currency Selection -->
      <div>
        <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
          Currency to Convert *
        </label>
        <select
          bind:value={currency}
          required
          class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
        >
          <option value="">Select currency...</option>
          <option value="VRSC">VRSC</option>
          <option value="vUSDC.vETH">vUSDC.vETH</option>
        </select>
      </div>

      <!-- Amount Input -->
      <div>
        <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
          Amount *
        </label>
        <input
          type="number"
          step="any"
          bind:value={amount}
          required
          placeholder="Enter amount"
          class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
        />
      </div>

      <!-- Convert To (info) -->
      <div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded-lg">
        <div class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
          <div class="flex justify-between">
            <span>Converting to:</span>
            <span class="font-semibold text-verusidx-stone-dark dark:text-white">VerusIDX</span>
          </div>
        </div>
      </div>

      <!-- Estimate Button -->
      <button
        type="submit"
        disabled={isEstimating}
        class="w-full px-6 py-3 bg-verusidx-mountain-blue dark:bg-verusidx-turquoise-deep text-white rounded-lg hover:bg-verusidx-lake-blue dark:hover:bg-verusidx-turquoise-bright transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {isEstimating ? 'Estimating...' : 'Estimate'}
      </button>
    </form>

    <!-- Estimate Result -->
    {#if estimateResult}
      <div class="mt-6 p-4 bg-verusidx-forest-deep/10 dark:bg-verusidx-turquoise-deep/20 border border-verusidx-turquoise-light dark:border-verusidx-turquoise-bright rounded-lg">
        <h4 class="text-sm font-semibold text-verusidx-forest-deep dark:text-verusidx-turquoise-light mb-3">
          Estimated Result
        </h4>
        <div class="space-y-2 text-sm">
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">You send:</span>
            <span class="font-mono text-verusidx-stone-dark dark:text-white">
              {parseFloat(amount).toFixed(8)} {currency}
            </span>
          </div>
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">You receive (estimated):</span>
            <span class="font-mono font-semibold text-verusidx-forest-deep dark:text-verusidx-turquoise-light">
              ~{estimateResult.estimatedcurrencyout?.toFixed(8) || '0.00000000'} VerusIDX
            </span>
          </div>
          {#if estimateResult.conversionfees}
            <div class="flex justify-between items-center pt-2 border-t border-verusidx-turquoise-light dark:border-verusidx-turquoise-deep">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist text-xs">Conversion fees:</span>
              <span class="font-mono text-verusidx-mountain-grey dark:text-verusidx-mountain-mist text-xs">
                {estimateResult.conversionfees.toFixed(8)} VRSC
              </span>
            </div>
          {/if}
        </div>
        <p class="mt-3 text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
          This is an estimate. Actual amount may vary slightly based on blockchain conditions.
        </p>
      </div>
    {/if}

    <!-- Close Button -->
    <button
      onclick={onClose}
      class="mt-6 w-full px-4 py-2 bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-stone-medium dark:text-verusidx-mountain-mist rounded-lg hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-light transition-colors"
    >
      Close
    </button>
  </div>
</Modal>
