<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  // Export the fetch function to parent component
  export let fetchChartData: (params: {
    basket: string;
    denominator: string;
    numerator: string;
    timeframe: string;
  }) => Promise<any[]>;

  // Timeframe to block interval mapping
  const timeframeIntervals = {
    '1D': { step: 5, range: 1440 },
    '4D': { step: 15, range: 5760 },
    '2W': { step: 60, range: 20160 },
    '2M': { step: 240, range: 86400 },
    '6M': { step: 720, range: 259200 },
    '1Y': { step: 1440, range: 525600 },
    'ALL': { step: 10080, range: null }
  };

  // Extract chart data from RPC response
  function extractChartData(response: any[], denominator: string, numerator: string): any[] {
    const chartData: any[] = [];

    for (const item of response) {
      // Skip summary objects
      if (item.totalvolume !== undefined) continue;

      // Skip items without conversion data
      if (!item.conversiondata?.volumepairs) continue;

      // Find the matching currency pair
      const volumePair = item.conversiondata.volumepairs.find((pair: any) =>
        pair.currency === denominator && pair.convertto === numerator
      );

      if (volumePair) {
        chartData.push({
          time: item.blocktime,
          open: volumePair.open,
          high: volumePair.high,
          low: volumePair.low,
          close: volumePair.close,
          volume: volumePair.volume || 0
        });
      }
    }

    return chartData.sort((a, b) => a.time - b.time);
  }

  // Main data fetching function
  async function safeFetchChartData(params: {
    basket: string;
    denominator: string;
    numerator: string;
    timeframe: string;
  }): Promise<any[]> {
    const { basket, denominator, numerator, timeframe } = params;

    // Validate inputs
    if (!basket || !denominator || !numerator || !timeframe) {
      throw new Error('Missing required parameters');
    }

    if (denominator === numerator) {
      throw new Error('Denominator and numerator cannot be the same currency');
    }

    // Get timeframe configuration
    const intervalConfig = timeframeIntervals[timeframe as keyof typeof timeframeIntervals];
    if (!intervalConfig) {
      throw new Error(`Unsupported timeframe: ${timeframe}`);
    }

    try {
      // Get current block height
      const currentBlockResponse = await invoke('get_info', {});
      const currentHeight = (currentBlockResponse as any).longestchain;

      let startHeight: number;
      let heightRange: string;

      if (timeframe === 'ALL') {
        // Get basket currency start block
        const basketResponse = await invoke('get_currency', {
          currencyName: basket,
          height: null
        });

        const basketStartBlock = (basketResponse as any).startblock;
        if (!basketStartBlock) {
          throw new Error('Could not determine basket currency start block');
        }

        startHeight = basketStartBlock;
        heightRange = `${startHeight},${currentHeight},${intervalConfig.step}`;
      } else {
        // Regular timeframes: last N blocks
        if (!currentHeight || currentHeight < intervalConfig.range!) {
          throw new Error('Insufficient blockchain data for selected timeframe');
        }
        startHeight = currentHeight - intervalConfig.range!;
        heightRange = `${startHeight},${currentHeight},${intervalConfig.step}`;
      }

      // Make RPC call
      const response = await invoke('get_currency_state', {
        currencyName: basket,
        heightRange: heightRange,
        conversionCurrency: denominator
      });

      // Validate response
      if (!Array.isArray(response)) {
        throw new Error('Invalid RPC response format');
      }

      // Extract and transform chart data
      const chartData = extractChartData(response as any[], denominator, numerator);

      if (chartData.length === 0) {
        throw new Error(`No trading data found for ${numerator}/${denominator} pair in the selected timeframe`);
      }

      return chartData;

    } catch (error: any) {
      // User-friendly error messages
      if (error.message?.includes('No active RPC connection')) {
        throw new Error('No connection to Verus daemon. Please check your RPC connection.');
      } else if (error.message?.includes('not found')) {
        throw new Error(`Currency "${basket}" not found. Please verify the currency exists.`);
      } else if (error.message?.includes('timeout')) {
        throw new Error('Request timed out. Please try again or select a smaller timeframe.');
      } else {
        throw new Error(error.message || 'Failed to fetch chart data. Please try again.');
      }
    }
  }

  // Bind the function on mount
  onMount(() => {
    fetchChartData = safeFetchChartData;
  });
</script>

<!-- This component is headless - no visual output -->
