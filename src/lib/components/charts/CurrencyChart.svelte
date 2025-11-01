<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { createChart, CandlestickSeries, LineSeries, type IChartApi, type ISeriesApi, ColorType } from 'lightweight-charts';

  // Props
  const { data = [], type = 'candlestick', pair = '' } = $props<{
    data?: any[];
    type?: 'candlestick' | 'line';
    pair?: string;
  }>();

  // Chart elements
  let chartContainer: HTMLDivElement;
  let chart: IChartApi | null = null;
  let series: ISeriesApi<'Candlestick'> | ISeriesApi<'Line'> | null = null;
  let isDarkMode = false;

  // Initialize chart on mount
  onMount(() => {
    isDarkMode = document.documentElement.classList.contains('dark');

    // Watch for theme changes
    const observer = new MutationObserver(() => {
      const newDarkMode = document.documentElement.classList.contains('dark');
      if (newDarkMode !== isDarkMode) {
        isDarkMode = newDarkMode;
        updateTheme();
      }
    });
    observer.observe(document.documentElement, { attributes: true, attributeFilter: ['class'] });

    // Create chart with autoSize
    const options = {
      autoSize: true,
      ...getChartOptions()
    };

    try {
      chart = createChart(chartContainer, options);
      createSeries();
    } catch (error) {
      console.error('[Chart] Error creating chart:', error);
    }

    return () => {
      observer.disconnect();
    };
  });

  // Cleanup on destroy
  onDestroy(() => {
    if (chart) chart.remove();
  });

  // Get chart options based on theme
  function getChartOptions() {
    return {
      layout: {
        attributionLogo: true,
        background: { type: ColorType.Solid, color: 'transparent' },
        textColor: isDarkMode ? '#D1D5DB' : '#1F2937',
        fontSize: 12,
        fontFamily: 'ui-sans-serif, system-ui, sans-serif'
      },
      grid: {
        vertLines: { color: isDarkMode ? '#374151' : '#E5E7EB' },
        horzLines: { color: isDarkMode ? '#374151' : '#E5E7EB' }
      },
      crosshair: {
        mode: 1,
        vertLine: { color: isDarkMode ? '#6B7280' : '#9CA3AF', style: 2 },
        horzLine: { color: isDarkMode ? '#6B7280' : '#9CA3AF', style: 2 }
      },
      timeScale: {
        timeVisible: true,
        secondsVisible: false,
        borderColor: isDarkMode ? '#4B5563' : '#D1D5DB'
      },
      rightPriceScale: {
        borderColor: isDarkMode ? '#4B5563' : '#D1D5DB',
        scaleMargins: { top: 0.1, bottom: 0.1 }
      }
    };
  }

  // Create series based on chart type
  function createSeries() {
    if (!chart) {
      console.error('[Chart] Cannot create series - chart is null');
      return;
    }

    // Remove existing series
    if (series) {
      chart.removeSeries(series);
      series = null;
    }

    // Create new series with v5 API
    try {
      if (type === 'candlestick') {
        series = chart.addSeries(CandlestickSeries, {
          upColor: '#10B981',
          downColor: '#EF4444',
          borderUpColor: '#10B981',
          borderDownColor: '#EF4444',
          wickUpColor: '#10B981',
          wickDownColor: '#EF4444',
          priceFormat: { type: 'price', precision: 8, minMove: 0.00000001 }
        });
      } else {
        series = chart.addSeries(LineSeries, {
          color: isDarkMode ? '#60A5FA' : '#3B82F6',
          lineWidth: 2,
          priceFormat: { type: 'price', precision: 8, minMove: 0.00000001 }
        });
      }

      // Set data if available
      if (data.length > 0) {
        updateData();
      }
    } catch (error) {
      console.error('[Chart] Error creating series:', error);
    }
  }

  // Update chart data
  function updateData() {
    if (!series || data.length === 0) {
      return;
    }

    try {
      let chartData;
      if (type === 'candlestick') {
        chartData = data.map(d => ({ time: d.time, open: d.open, high: d.high, low: d.low, close: d.close }));
      } else {
        chartData = data.map(d => ({ time: d.time, value: d.close }));
      }

      series.setData(chartData);
      chart?.timeScale().fitContent();
    } catch (error) {
      console.error('[Chart] Error setting data:', error);
    }
  }

  // Update theme colors
  function updateTheme() {
    if (!chart) return;
    chart.applyOptions(getChartOptions());
    if (type === 'line' && series) {
      series.applyOptions({ color: isDarkMode ? '#60A5FA' : '#3B82F6' });
    }
  }

  // Watch for prop changes
  $effect(() => {
    if (chart) {
      createSeries();
    }
  });
</script>

<div class="h-full w-full relative">
  {#if pair}
    <div class="absolute top-4 left-4 z-10 bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm rounded-md px-3 py-1.5 shadow-sm">
      <h3 class="text-sm font-semibold text-gray-900 dark:text-white">{pair}</h3>
      <p class="text-xs text-gray-600 dark:text-gray-400">{data.length} points</p>
    </div>
  {/if}

  <div bind:this={chartContainer} class="w-full h-full" style="min-height: 400px;"></div>

  <div class="absolute bottom-2 right-2 text-xs text-gray-500">
    <span>Powered by TradingView</span>
  </div>
</div>
