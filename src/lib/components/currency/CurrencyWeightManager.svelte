<script lang="ts">
  interface Props {
    currencies: string[];
    weights: number[];
    onUpdate: (currencies: string[], weights: number[]) => void;
    error?: string;
  }

  let { currencies, weights, onUpdate, error }: Props = $props();

  // Initialize with minimum entries only on mount, prevent during re-renders
  let isInitialized = $state(false);
  let prevCurrencyLength = $state(currencies.length);
  
  $effect(() => {
    // Only initialize if truly empty AND this isn't a re-render with existing data
    if (!isInitialized && currencies.length === 0 && weights.length === 0) {
      isInitialized = true;
      const newCurrencies = [''];
      const newWeights = [0];
      onUpdate(newCurrencies, newWeights);
    } else if (currencies.length > 0 || weights.length > 0) {
      // Mark as initialized if parent already has data
      isInitialized = true;
    }
    prevCurrencyLength = currencies.length;
  });

  // Calculate total weight
  let totalWeight = $derived(weights.reduce((sum, weight) => sum + (weight || 0), 0));
  
  // Safe grid column count to prevent CSS issues
  let gridColumns = $derived(`repeat(${Math.max(1, currencies.length)}, 1fr) auto`);

  // Validation
  let isValidTotal = $derived(Math.abs(totalWeight - 1.0) < 0.0001);
  let hasNonZeroWeights = $derived(weights.every(weight => weight > 0));
  let isValid = $derived(isValidTotal && hasNonZeroWeights);

  function addCurrency() {
    const newCurrencies = [...currencies, ''];
    const remainingWeight = Math.max(0, 1.0 - totalWeight);
    const newWeights = [...weights, remainingWeight];
    
    onUpdate(newCurrencies, newWeights);
  }

  function removeCurrency(index: number) {
    if (currencies.length > 1) {
      const newCurrencies = currencies.filter((_, i) => i !== index);
      const newWeights = weights.filter((_, i) => i !== index);
      
      onUpdate(newCurrencies, newWeights);
    }
  }

  function updateCurrency(index: number, value: string) {
    const newCurrencies = [...currencies];
    newCurrencies[index] = value;
    onUpdate(newCurrencies, weights);
  }

  function updateWeight(index: number, value: number) {
    const newWeights = [...weights];
    newWeights[index] = value || 0;
    onUpdate(currencies, newWeights);
  }

  // Auto-balance weights when adding currencies
  function balanceWeights() {
    const filledCount = currencies.filter(c => c.trim()).length;
    if (filledCount > 0) {
      const evenWeight = 1.0 / filledCount;
      const newWeights = weights.map((_, i) => currencies[i].trim() ? evenWeight : 0);
      onUpdate(currencies, newWeights);
    }
  }
</script>

<div class="space-y-4">
  <div>
    <div class="flex items-center justify-between mb-2">
      <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white">
        Reserve Currencies & Weights *
      </label>
      <button
        type="button"
        onclick={balanceWeights}
        class="text-xs text-verusidx-turquoise-deep hover:text-verusidx-turquoise-bright"
      >
        Auto-balance weights
      </button>
    </div>
    
    <!-- Currencies Row -->
    <div class="grid gap-2 mb-2" style="grid-template-columns: {gridColumns};">
      {#each currencies as currency, index}
        <div class="space-y-1">
          <label class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
            Currency {index + 1}
          </label>
          <input 
            type="text"
            value={currency}
            oninput={(e) => updateCurrency(index, (e.target as HTMLInputElement).value)}
            placeholder="Enter currency name"
            required
            class="w-full p-2 text-sm border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
          />
        </div>
      {/each}
      
      <!-- Add/Remove buttons column -->
      <div class="space-y-1">
        <label class="text-xs text-transparent">Actions</label>
        <div class="flex flex-col space-y-1">
          {#if currencies.length < 10}
            <button
              type="button"
              onclick={addCurrency}
              class="px-2 py-2 text-xs text-verusidx-turquoise-deep hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium rounded transition-colors"
              title="Add currency"
            >
              +
            </button>
          {/if}
          {#if currencies.length > 1}
            <button
              type="button"
              onclick={() => removeCurrency(currencies.length - 1)}
              class="px-2 py-2 text-xs text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-colors"
              title="Remove last currency"
            >
              −
            </button>
          {/if}
        </div>
      </div>
    </div>

    <!-- Weights Row -->
    <div class="grid gap-2" style="grid-template-columns: {gridColumns};">
      {#each weights as weight, index}
        <div class="space-y-1">
          <label class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
            Weight {index + 1}
          </label>
          <input 
            type="number" 
            value={weight}
            oninput={(e) => updateWeight(index, parseFloat((e.target as HTMLInputElement).value) || 0)}
            min="0"
            max="1"
            step="0.1"
            placeholder="0.0"
            class="w-full p-2 text-sm border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
          />
        </div>
      {/each}
      
      <!-- Total display column -->
      <div class="space-y-1">
        <label class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Total</label>
        <div class="p-2 text-sm text-center rounded-lg {isValid ? 'bg-green-50 dark:bg-green-900/20 text-green-700 dark:text-green-300' : 'bg-red-50 dark:bg-red-900/20 text-red-700 dark:text-red-300'}">
          {totalWeight.toFixed(3)}
        </div>
      </div>
    </div>

    <!-- Validation Messages -->
    <div class="mt-2 space-y-1">
      {#if error}
        <p class="text-xs text-red-600 dark:text-red-400">{error}</p>
      {/if}
      
      {#if !isValidTotal}
        <p class="text-xs text-red-600 dark:text-red-400">
          ⚠️ Total weight must equal 1.0 (currently {totalWeight.toFixed(3)})
        </p>
      {:else if !hasNonZeroWeights}
        <p class="text-xs text-red-600 dark:text-red-400">
          ⚠️ All currencies must have non-zero weights
        </p>
      {:else}
        <p class="text-xs text-green-600 dark:text-green-400">
          ✓ All weights valid
        </p>
      {/if}
      
      <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
        Reserve currencies define what backs your basket currency. Weights determine the ratio of each reserve.
      </p>
    </div>
  </div>
</div>