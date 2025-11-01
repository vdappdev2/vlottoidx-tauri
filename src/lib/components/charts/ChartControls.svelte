<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { connectionStore } from '$lib/stores/connection';

  // Bound props from parent
  export let selectedBasket = '';
  export let selectedDenominator = '';
  export let selectedNumerator = '';
  export let selectedTimeframe = '1D';
  export let chartType: 'candlestick' | 'line' = 'candlestick';
  export let isLoading = false;
  export let onFetch: (params: any) => Promise<void>;
  export let onRefresh: () => Promise<void>;
  export let canRefresh = false;

  // Currency lists
  let basketCurrencies: any[] = [];
  let reserveCurrencies: any[] = [];
  let isLoadingCurrencies = false;
  let currencyError: string | null = null;

  // Timeframe options
  const timeframes = [
    { value: '1D', label: '1 Day' },
    { value: '4D', label: '4 Days' },
    { value: '2W', label: '2 Weeks' },
    { value: '2M', label: '2 Months' },
    { value: '6M', label: '6 Months' },
    { value: '1Y', label: '1 Year' },
    { value: 'ALL', label: 'All Time' }
  ];

  // Chart type options
  const chartTypes = [
    { value: 'candlestick', label: 'Candlestick' },
    { value: 'line', label: 'Line' }
  ];

  // Load basket currencies on mount
  onMount(async () => {
    await loadBasketCurrencies();
  });

  // Load basket currencies (filter for basket type)
  async function loadBasketCurrencies() {
    isLoadingCurrencies = true;
    currencyError = null;

    try {
      const response = await invoke('list_currencies', {
        query: null,
        verbose: true
      });

      // Filter for basket currencies using bitmask values, excluding bricked ones
      basketCurrencies = (response as any[]).filter((currency: any) => {
        const options = currency.currencydefinition?.options || 0;
        const flags = currency.flags || 0;
        // Basket currency bitmask values (same as in CurrencyListCard), excluding bricked (flags === 37)
        return [33, 35, 41, 43, 49, 51, 57, 59].includes(options) && flags !== 37;
      });
    } catch (error: any) {
      console.error('Failed to load basket currencies:', error);
      currencyError = 'Failed to load basket currencies';
      basketCurrencies = [];
    } finally {
      isLoadingCurrencies = false;
    }
  }

  // Load reserve currencies for selected basket
  async function loadReserveCurrencies(basketName: string) {
    if (!basketName) {
      reserveCurrencies = [];
      return;
    }

    try {
      const response = await invoke('get_currency', {
        currencyName: basketName,
        height: null
      });

      const basketCurrency = response as any;

      // The currencynames field is at the root level of get_currency response
      if (basketCurrency?.currencynames) {
        // currencynames is an object mapping i-addresses to friendly names
        reserveCurrencies = Object.entries(basketCurrency.currencynames).map(([iAddress, friendlyName]) => ({
          currencyid: iAddress,
          fullyqualifiedname: iAddress,
          name: friendlyName as string
        }));

        // Add the basket currency itself using the basketName we already have
        reserveCurrencies.push({
          currencyid: basketName,
          fullyqualifiedname: basketName,
          name: basketName  // The name we're already using in the selection
        });
      } else {
        reserveCurrencies = [];
      }
    } catch (error: any) {
      console.error('Failed to load reserve currencies:', error);
      reserveCurrencies = [];
    }
  }

  // Handle basket currency change
  function handleBasketChange() {
    // Reset denominator and numerator when basket changes
    selectedDenominator = '';
    selectedNumerator = '';
    loadReserveCurrencies(selectedBasket);
  }

  // Validation
  $: canFetch = selectedBasket && selectedDenominator && selectedNumerator && 
                selectedDenominator !== selectedNumerator && !isLoading;

  // Handle fetch chart
  async function handleFetch() {
    if (!canFetch) return;

    await onFetch({
      basket: selectedBasket,
      denominator: selectedDenominator,
      numerator: selectedNumerator,
      timeframe: selectedTimeframe
    });
  }

  // Handle refresh
  async function handleRefreshClick() {
    if (!canRefresh || isLoading) return;
    await onRefresh();
  }
</script>

<div class="space-y-4">
  <!-- Currency Selection Row -->
  <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
    <!-- Basket Currency -->
    <div>
      <label for="basket-select" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
        Basket Currency
      </label>
      <select
        id="basket-select"
        bind:value={selectedBasket}
        on:change={handleBasketChange}
        disabled={isLoadingCurrencies}
        class="block w-full rounded-md border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white shadow-sm focus:border-primary-500 focus:ring-primary-500 disabled:opacity-50"
      >
        <option value="">
          {isLoadingCurrencies ? 'Loading...' : 'Select Basket Currency'}
        </option>
        {#each basketCurrencies as currency}
          <option value={currency.currencydefinition?.fullyqualifiedname || currency.currencydefinition?.name}>
            {currency.currencydefinition?.name || currency.currencydefinition?.fullyqualifiedname || 'Unknown'}
          </option>
        {/each}
      </select>
    </div>

    <!-- Numerator -->
    <div>
      <label for="numerator-select" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
        Numerator (Base)
      </label>
      <select
        id="numerator-select"
        bind:value={selectedNumerator}
        disabled={!selectedBasket || reserveCurrencies.length === 0}
        class="block w-full rounded-md border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white shadow-sm focus:border-primary-500 focus:ring-primary-500 disabled:opacity-50"
      >
        <option value="">Select Numerator</option>
        {#each reserveCurrencies as currency}
          <option value={currency.name} disabled={currency.name === selectedDenominator}>
            {currency.name}
          </option>
        {/each}
      </select>
    </div>

    <!-- Denominator -->
    <div>
      <label for="denominator-select" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
        Denominator (Quote)
      </label>
      <select
        id="denominator-select"
        bind:value={selectedDenominator}
        disabled={!selectedBasket || reserveCurrencies.length === 0}
        class="block w-full rounded-md border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white shadow-sm focus:border-primary-500 focus:ring-primary-500 disabled:opacity-50"
      >
        <option value="">Select Denominator</option>
        {#each reserveCurrencies as currency}
          <option value={currency.name}>
            {currency.name}
          </option>
        {/each}
      </select>
    </div>
  </div>

  <!-- Controls Row -->
  <div class="flex flex-wrap items-center gap-4">
    <!-- Timeframe -->
    <div>
      <label for="timeframe-select" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
        Timeframe
      </label>
      <select
        id="timeframe-select"
        bind:value={selectedTimeframe}
        class="block rounded-md border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white shadow-sm focus:border-primary-500 focus:ring-primary-500"
      >
        {#each timeframes as timeframe}
          <option value={timeframe.value}>{timeframe.label}</option>
        {/each}
      </select>
    </div>

    <!-- Chart Type -->
    <div>
      <label for="chart-type-select" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
        Chart Type
      </label>
      <select
        id="chart-type-select"
        bind:value={chartType}
        class="block rounded-md border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white shadow-sm focus:border-primary-500 focus:ring-primary-500"
      >
        {#each chartTypes as type}
          <option value={type.value}>{type.label}</option>
        {/each}
      </select>
    </div>

    <!-- Action Buttons -->
    <div class="flex items-end gap-2">
      <!-- Fetch Chart Button -->
      <button
        type="button"
        on:click={handleFetch}
        disabled={!canFetch}
        class="inline-flex items-center px-4 py-2 bg-verusidx-mountain-blue hover:bg-verusidx-lake-blue text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {#if isLoading}
          <svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          Loading...
        {:else}
          Fetch Chart
        {/if}
      </button>

      <!-- Refresh Button -->
      <button
        type="button"
        on:click={handleRefreshClick}
        disabled={!canRefresh || isLoading}
        class="inline-flex items-center px-3 py-2 border border-gray-300 dark:border-gray-600 text-sm font-medium rounded-md text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 hover:bg-gray-50 dark:hover:bg-gray-600 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500 disabled:opacity-50 disabled:cursor-not-allowed"
        title="Refresh current chart"
      >
        <svg class="h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
        </svg>
      </button>
    </div>
  </div>

  <!-- Error Display -->
  {#if currencyError}
    <div class="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-md p-3">
      <div class="text-yellow-800 dark:text-yellow-200 text-sm">
        {currencyError}
      </div>
    </div>
  {/if}

  <!-- Validation Messages -->
  {#if selectedDenominator === selectedNumerator && selectedDenominator}
    <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md p-3">
      <div class="text-red-800 dark:text-red-200 text-sm">
        Denominator and numerator cannot be the same currency.
      </div>
    </div>
  {/if}
</div>