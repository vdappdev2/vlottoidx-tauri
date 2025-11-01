<script lang="ts">
  import { onMount } from 'svelte';
  import { connectionStore } from '$lib/stores/connection';
  import BlockHeightHeader from '$lib/components/BlockHeightHeader.svelte';
  import ChartControls from '$lib/components/charts/ChartControls.svelte';
  import CurrencyChart from '$lib/components/charts/CurrencyChart.svelte';
  import ChartDataManager from '$lib/components/charts/ChartDataManager.svelte';

  // Chart state
  let chartData: any[] = [];
  let isLoading = false;
  let error: string | null = null;
  let currentParams: any = null;

  // Chart configuration
  let selectedBasket = '';
  let selectedDenominator = '';
  let selectedNumerator = '';
  let selectedTimeframe = '1D';
  let chartType: 'candlestick' | 'line' = 'candlestick';

  // Data fetching function
  async function handleFetchChart(params: {
    basket: string;
    denominator: string;
    numerator: string;
    timeframe: string;
  }) {
    isLoading = true;
    error = null;
    
    try {
      const data = await fetchChartData(params);
      chartData = data;
      currentParams = params;
    } catch (e: any) {
      error = e.message || 'Failed to fetch chart data';
      chartData = [];
    } finally {
      isLoading = false;
    }
  }

  // Refresh current chart
  async function handleRefresh() {
    if (currentParams) {
      await handleFetchChart(currentParams);
    }
  }

  // Chart data fetching (delegated to ChartDataManager)
  let fetchChartData: (params: any) => Promise<any[]>;
</script>

<div class="min-h-screen bg-white dark:bg-gray-900">
  <!-- Block Height Header -->
  <BlockHeightHeader />
  
  <!-- Main Charts Container -->
  <div class="flex flex-col h-[calc(100vh-4rem)]">
    <!-- Chart Controls Bar -->
    <div class="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 p-4">
      <ChartControls
        bind:selectedBasket
        bind:selectedDenominator
        bind:selectedNumerator
        bind:selectedTimeframe
        bind:chartType
        {isLoading}
        onFetch={handleFetchChart}
        onRefresh={handleRefresh}
        canRefresh={currentParams !== null}
      />
    </div>

    <!-- Error Display -->
    {#if error}
      <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md p-4 m-4">
        <div class="flex">
          <div class="text-red-800 dark:text-red-200">
            <h3 class="text-sm font-medium">Chart Data Error</h3>
            <p class="mt-1 text-sm">{error}</p>
          </div>
        </div>
      </div>
    {/if}

    <!-- Chart Container -->
    <div class="flex-1 bg-white dark:bg-gray-900 p-4">
      {#if isLoading}
        <div class="flex items-center justify-center h-full">
          <div class="text-center">
            <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600 mx-auto"></div>
            <p class="mt-4 text-gray-600 dark:text-gray-400">Loading chart data...</p>
          </div>
        </div>
      {:else if chartData.length > 0}
        <CurrencyChart
          data={chartData}
          type={chartType}
          pair={currentParams ? `${currentParams.numerator}/${currentParams.denominator}` : ''}
        />
      {:else if !error}
        <div class="flex items-center justify-center h-full">
          <div class="text-center text-gray-500 dark:text-gray-400">
            <svg class="mx-auto h-16 w-16 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" 
                    d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 00-2 2h2a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v14" />
            </svg>
            <h3 class="text-lg font-medium mb-2">Select Currency Pair</h3>
            <p class="text-sm">Choose a basket currency, denominator, and numerator to view price charts</p>
          </div>
        </div>
      {/if}
    </div>
  </div>

  <!-- Data Manager Component (handles RPC calls) -->
  <ChartDataManager bind:fetchChartData />
</div>