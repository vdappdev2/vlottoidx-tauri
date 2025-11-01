<script lang="ts">
  import { Modal } from '../cards';

  interface Props {
    isOpen: boolean;
    onClose: () => void;
    onSelect: (type: 'simple' | 'basket') => void;
    currencyClass: 'decentralized' | 'centralized';
  }

  let { isOpen = false, onClose, onSelect, currencyClass }: Props = $props();

  // Modal title and description based on currency class
  let title = $derived(currencyClass === 'decentralized' ? 'Create Decentralized Currency' : 'Create Centralized Currency');
  let subtitle = $derived(currencyClass === 'decentralized' 
    ? 'Static supply currency - no minting after launch' 
    : 'Root identity can mint and burn supply'
  );

  function handleSelect(type: 'simple' | 'basket') {
    onSelect(type);
    onClose();
  }
</script>

<Modal {isOpen} onclose={onClose} {title} size="lg">
  <div class="p-6">
    <!-- Currency Class Info -->
    <div class="mb-6 p-4 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
      <p class="text-blue-700 dark:text-blue-300 text-sm">
        {#if currencyClass === 'decentralized'}
          🪙 <strong>Decentralized Currency:</strong> {subtitle}. Choose the structure below.
        {:else}
          🏦 <strong>Centralized Currency:</strong> {subtitle}. Optionally set an end block to become decentralized later.
        {/if}
      </p>
    </div>

    <!-- Currency Type Selection -->
    <div class="mb-6">
      <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-4">
        Choose Currency Structure
      </h3>
      
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <!-- Simple Token Option -->
        <button
          onclick={() => handleSelect('simple')}
          class="group p-6 border-2 border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg hover:border-verusidx-forest-deep hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-all text-left"
        >
          <div class="text-3xl mb-3 group-hover:scale-110 transition-transform">
            🟡
          </div>
          <h4 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-2">
            Simple Token
          </h4>
          <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
            {#if currencyClass === 'decentralized'}
              Fixed supply currency without reserve backing. Launch with preconversions or preallocations.
            {:else}
              Currency you can mint and burn. Optionally allow preconversions for initial distribution.
            {/if}
          </p>
        </button>

        <!-- Basket Currency Option -->
        <button
          onclick={() => handleSelect('basket')}
          class="group p-6 border-2 border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg hover:border-verusidx-forest-deep hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-all text-left"
        >
          <div class="text-3xl mb-3 group-hover:scale-110 transition-transform">
            🧺
          </div>
          <h4 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-2">
            Basket Currency
          </h4>
          <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
            {#if currencyClass === 'decentralized'}
              AMM currency backed by reserve currencies. Automatic market-making with weight-based reserves.
            {:else}
              AMM currency you can mint/burn while controlling reserve ratios and pricing.
            {/if}
          </p>
        </button>
      </div>
    </div>

    <!-- Help Information -->
    <div class="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
      <h4 class="text-sm font-semibold text-yellow-800 dark:text-yellow-200 mb-2">
        💡 Not sure which to choose?
      </h4>
      <ul class="text-xs text-yellow-700 dark:text-yellow-300 space-y-1">
        <li><strong>Simple Token:</strong> Best for standard tokens, rewards, or fixed-supply currencies</li>
        <li><strong>Basket Currency:</strong> Best for stablecoins, multi-asset currencies, or AMM-based tokens</li>
      </ul>
    </div>

    <!-- Cancel Button -->
    <div class="flex justify-end pt-6 border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium mt-6">
      <button
        type="button"
        onclick={onClose}
        class="px-6 py-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium text-verusidx-stone-dark dark:text-white rounded-lg hover:bg-verusidx-mountain-mist dark:hover:bg-verusidx-stone-medium transition-colors"
      >
        Cancel
      </button>
    </div>
  </div>
</Modal>