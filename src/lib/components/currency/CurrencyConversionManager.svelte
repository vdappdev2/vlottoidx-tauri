<script lang="ts">
  interface Props {
    currencies: string[];
    conversions: number[];
    onUpdate: (currencies: string[], conversions: number[]) => void;
    error?: string;
  }

  let { currencies, conversions, onUpdate, error }: Props = $props();

  // Ensure arrays are same length
  function syncArrays() {
    while (conversions.length < currencies.length) {
      conversions.push(0);
    }
    while (currencies.length < conversions.length) {
      currencies.push('');
    }
  }

  // Initialize if empty
  $effect(() => {
    if (currencies.length === 0 && conversions.length === 0) {
      currencies.push('');
      conversions.push(0);
    }
    syncArrays();
  });

  function addCurrency() {
    const newCurrencies = [...currencies, ''];
    const newConversions = [...conversions, 0];
    
    currencies.splice(0, currencies.length, ...newCurrencies);
    conversions.splice(0, conversions.length, ...newConversions);
    onUpdate(currencies, conversions);
  }

  function removeCurrency(index: number) {
    if (currencies.length > 1) {
      const newCurrencies = currencies.filter((_, i) => i !== index);
      const newConversions = conversions.filter((_, i) => i !== index);
      
      currencies.splice(0, currencies.length, ...newCurrencies);
      conversions.splice(0, conversions.length, ...newConversions);
      onUpdate(currencies, conversions);
    }
  }

  function updateCurrency(index: number, value: string) {
    currencies[index] = value;
    syncArrays();
    onUpdate(currencies, conversions);
  }

  function updateConversion(index: number, value: number) {
    conversions[index] = value || 0;
    syncArrays();
    onUpdate(currencies, conversions);
  }

  function clearAll() {
    currencies.splice(0, currencies.length);
    conversions.splice(0, conversions.length);
    onUpdate(currencies, conversions);
  }
</script>

<div class="space-y-4">
  <div>
    <div class="flex items-center justify-between mb-2">
      <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white">
        Preconversion Currencies & Rates
      </label>
      <button
        type="button"
        onclick={clearAll}
        class="text-xs text-red-600 hover:text-red-700 dark:text-red-400 dark:hover:text-red-300"
      >
        Clear all
      </button>
    </div>
    
    <!-- Currencies Row -->
    <div class="grid gap-2 mb-2" style="grid-template-columns: repeat({Math.max(currencies.length, 1)}, 1fr) auto;">
      {#each currencies as currency, index}
        <div class="space-y-1">
          <label class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
            Currency {index + 1}
          </label>
          <input 
            type="text"
            value={currency}
            oninput={(e) => updateCurrency(index, e.target.value)}
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

    <!-- Conversion Rates Row -->
    <div class="grid gap-2" style="grid-template-columns: repeat({Math.max(conversions.length, 1)}, 1fr) auto;">
      {#each conversions as conversion, index}
        <div class="space-y-1">
          <label class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
            Rate {index + 1}
          </label>
          <input 
            type="number" 
            value={conversion}
            oninput={(e) => updateConversion(index, parseFloat(e.target.value) || 0)}
            min="0"
            step="0.00000001"
            placeholder="0.00000000"
            class="w-full p-2 text-sm border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
          />
        </div>
      {/each}
      
      <!-- Info column -->
      <div class="space-y-1">
        <label class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Info</label>
        <div class="p-2 text-xs text-center bg-verusidx-sky-soft dark:bg-verusidx-stone-medium rounded-lg text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
          {currencies.length} pairs
        </div>
      </div>
    </div>

    <!-- Validation Messages -->
    <div class="mt-2 space-y-1">
      {#if error}
        <p class="text-xs text-red-600 dark:text-red-400">{error}</p>
      {/if}
      
      {#if currencies.length !== conversions.length}
        <p class="text-xs text-red-600 dark:text-red-400">
          ⚠️ Number of currencies and conversion rates must match
        </p>
      {:else if currencies.length > 0}
        <p class="text-xs text-green-600 dark:text-green-400">
          ✓ {currencies.length} currency/rate pairs configured
        </p>
      {/if}
      
      <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
        For simple tokens, people can preconvert these currencies to receive your token at launch. 
        Conversion rate determines how much of your token they receive per unit of the preconversion currency.
      </p>
      
      {#if currencies.length > 0 && conversions.length > 0}
        <div class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
          <strong>Example:</strong> Rate 0.1 means 1 {currencies[0] || 'currency'} → 10 of your tokens
        </div>
      {/if}
    </div>
  </div>
</div>