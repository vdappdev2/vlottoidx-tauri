<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Modal } from "$lib/components/cards";

  interface Props {
    address: string;
    isOpen: boolean;
    onClose: () => void;
  }

  let { address, isOpen, onClose }: Props = $props();
  
  let currencyBalances = $state<any>(null);
  let isLoading = $state(false);
  let error = $state<string | null>(null);

  async function copyToClipboard(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      console.log("Copied to clipboard:", text);
    } catch (err) {
      console.error("Failed to copy:", err);
    }
  }

  async function loadCurrencyBalances() {
    if (!address) return;

    isLoading = true;
    error = null;
    
    try {
      const balances = await invoke("get_currency_balance", {
        address: address,
        minconf: 1,
        friendly_names: true,
        include_shared: true,
        chain: null
      });
      currencyBalances = balances;
    } catch (err) {
      error = `Failed to load currency balances: ${err}`;
      console.error("Failed to load currency balances:", err);
    } finally {
      isLoading = false;
    }
  }

  // Load balances when modal opens
  $effect(() => {
    if (isOpen && address) {
      loadCurrencyBalances();
    } else {
      // Reset state when closed
      currencyBalances = null;
      error = null;
    }
  });

  function formatBalance(amount: number | string): string {
    const num = typeof amount === 'string' ? parseFloat(amount) : amount;
    return num.toFixed(8);
  }
</script>

<Modal {isOpen} onclose={onClose} size="lg" title="Address Details">
  <div class="p-6">
    <h2 class="text-2xl font-bold text-verusidx-stone-dark dark:text-white mb-4">
      Address Details
    </h2>
    
    <!-- Address Display -->
    <div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-4 rounded-lg mb-6">
      <div class="flex justify-between items-center">
        <div class="flex-1">
          <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-1">Address</p>
          <p class="font-mono text-sm text-verusidx-stone-dark dark:text-white break-all">
            {address}
          </p>
        </div>
        <button
          onclick={() => copyToClipboard(address)}
          class="ml-4 px-3 py-1 bg-verusidx-mountain-blue dark:bg-verusidx-turquoise-deep text-white text-sm rounded hover:bg-verusidx-lake-blue dark:hover:bg-verusidx-turquoise-bright transition-colors"
        >
          Copy
        </button>
      </div>
    </div>

    <!-- Currency Balances -->
    <div>
      <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-3">
        Currency Balances
      </h3>
      
      {#if isLoading}
        <div class="text-center py-8">
          <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-verusidx-mountain-blue"></div>
          <p class="mt-2 text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Loading balances...</p>
        </div>
      {:else if error}
        <div class="bg-verusidx-lake-deep/10 dark:bg-verusidx-lake-deep/30 border border-verusidx-turquoise-light dark:border-verusidx-stone-medium rounded-lg p-4">
          <p class="text-verusidx-lake-deep dark:text-verusidx-turquoise-light">{error}</p>
        </div>
      {:else if currencyBalances}
        <div class="space-y-3 max-h-96 overflow-y-auto">
          {#if Object.keys(currencyBalances).length === 0}
            <p class="text-center text-verusidx-mountain-grey dark:text-verusidx-mountain-mist py-4">
              No currency balances found for this address
            </p>
          {:else}
            {#each Object.entries(currencyBalances) as [currency, amount]}
              <div class="flex justify-between items-center p-3 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium rounded-lg">
                <span class="font-medium text-verusidx-stone-dark dark:text-white">
                  {currency}
                </span>
                <span class="font-mono text-verusidx-stone-dark dark:text-white">
                  {formatBalance(amount as number)}
                </span>
              </div>
            {/each}
          {/if}
        </div>
      {:else}
        <p class="text-center text-verusidx-mountain-grey dark:text-verusidx-mountain-mist py-4">
          No balance data available
        </p>
      {/if}
    </div>

    <!-- Close Button -->
    <div class="mt-6 flex justify-end">
      <button
        onclick={onClose}
        class="px-6 py-2 bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-stone-medium dark:text-verusidx-mountain-mist rounded-lg hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-dark transition-colors"
      >
        Close
      </button>
    </div>
  </div>
</Modal>