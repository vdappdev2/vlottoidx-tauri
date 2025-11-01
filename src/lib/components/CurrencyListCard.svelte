<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { connectionStore, getChainParam } from "$lib/stores/connection";
  import { onMount, onDestroy } from "svelte";
  import CurrencyDetailsView from "./CurrencyDetailsView.svelte";
  import Modal from "./cards/Modal.svelte";

  let currencies = $state<any[]>([]);
  let isLoading = $state(true);
  let error = $state<string | null>(null);
  let selectedCurrency = $state<any>(null);
  let isViewingDetails = $state(false);
  let isLoadingDetails = $state(false);
  let detailsError = $state<string | null>(null);
  let connectionState = $state<any>(null);

  connectionStore.subscribe(state => {
    connectionState = state;
  });

  // Filter and search
  let searchFilter = $state("");
  let systemTypeFilter = $state("all");
  let launchStateFilter = $state("all");
  let currencyTypeFilter = $state("all");
  let fromSystemFilter = $state("");
  let provenanceFilter = $state("all");
  
  // Converter search
  let converterCurrency1 = $state("");
  let converterCurrency2 = $state("");
  let isConverterSearchActive = $state(false);

  // Pagination
  let currentPage = $state(1);
  let itemsPerPage = $state(10);

  // Sorting
  let sortField = $state<string>("name");
  let sortDirection = $state<"asc" | "desc">("asc");

  // State for filtered currencies
  let filteredCurrencies = $state<any[]>([]);
  let paginatedCurrencies = $state<any[]>([]);
  let totalPages = $state(0);

  // Combined effect to handle both filtering and pagination
  $effect(() => {
    if (!currencies || currencies.length === 0) {
      filteredCurrencies = [];
      totalPages = 0;
      paginatedCurrencies = [];
      return;
    }
    
    // Filter currencies
    const filtered = currencies.filter(currency => {
      // Extract currency definition - handle both nested and flat structures
      const currencyDef = currency.currencydefinition || currency;
      const name = currencyDef?.name || '';
      const currencyId = currencyDef?.currencyid || '';
      const systemId = currencyDef?.systemid || '';
      const parent = currencyDef?.parent || '';
      const launchState = currency?.launchstate || 'unknown';
      
      // Apply search filter
      const matchesSearch = !searchFilter || 
        name.toLowerCase().includes(searchFilter.toLowerCase()) ||
        currencyId.toLowerCase().includes(searchFilter.toLowerCase());
      
      // System type and launch state filtering is handled by RPC call
      // No client-side filtering needed for these
      
      // Apply currency type filter
      const currencyTypeKey = getCurrencyTypeKey(currency);
      const matchesCurrencyType = currencyTypeFilter === 'all' || currencyTypeKey === currencyTypeFilter;
      
      // Apply provenance filter
      const proofProtocol = currencyDef?.proofprotocol;
      const matchesProvenance = provenanceFilter === 'all' || String(proofProtocol) === provenanceFilter;
      
      return matchesSearch && matchesCurrencyType && matchesProvenance;
    });
    
    // Sort currencies
    const sorted = [...filtered].sort((a, b) => {
      const aDef = a.currencydefinition || a;
      const bDef = b.currencydefinition || b;
      
      let aValue: any, bValue: any;
      
      switch (sortField) {
        case 'name':
          aValue = (aDef?.name || '').toLowerCase();
          bValue = (bDef?.name || '').toLowerCase();
          break;
        case 'supply':
          aValue = a?.supply || 0;
          bValue = b?.supply || 0;
          break;
        case 'startblock':
          aValue = aDef?.startblock || 0;
          bValue = bDef?.startblock || 0;
          break;
        default:
          aValue = (aDef?.name || '').toLowerCase();
          bValue = (bDef?.name || '').toLowerCase();
      }
      
      if (aValue < bValue) return sortDirection === 'asc' ? -1 : 1;
      if (aValue > bValue) return sortDirection === 'asc' ? 1 : -1;
      return 0;
    });
    
    // Calculate pagination using sorted array
    const newTotalPages = Math.ceil(sorted.length / itemsPerPage);
    
    // Ensure current page is within bounds
    const validCurrentPage = Math.max(1, Math.min(currentPage, newTotalPages || 1));
    
    const startIndex = (validCurrentPage - 1) * itemsPerPage;
    const endIndex = startIndex + itemsPerPage;
    const paginated = sorted.slice(startIndex, endIndex);
    
    // Update all state at once
    filteredCurrencies = sorted; // Use sorted array for count
    totalPages = newTotalPages;
    paginatedCurrencies = paginated;
    
    // Reset to valid page if needed
    if (currentPage !== validCurrentPage) {
      currentPage = validCurrentPage;
    }
  });

  async function loadCurrencies() {
    isLoading = true;
    error = null;
    
    try {
      // Build converter array if converter search is active
      let converterArray: string[] | undefined = undefined;
      if (isConverterSearchActive && converterCurrency1.trim()) {
        converterArray = [converterCurrency1.trim()];
        if (converterCurrency2.trim()) {
          converterArray.push(converterCurrency2.trim());
        }
      }
      
      const chainParam = getChainParam(connectionState?.selectedChain);
      console.log("CurrencyListCard: Loading currencies for chain:", connectionState?.selectedChain, "param:", chainParam);
      
      const result = await invoke("list_currencies", { 
        query: null,
        verbose: true,
        launchstate: launchStateFilter !== "all" ? launchStateFilter : undefined,
        systemtype: systemTypeFilter !== "all" ? systemTypeFilter : undefined,
        fromsystem: fromSystemFilter.trim() || undefined,
        converter: converterArray,
        chain: chainParam
      });
      
      currencies = Array.isArray(result) ? result : [];
    } catch (err) {
      console.error("Failed to load currencies:", err);
      error = typeof err === 'string' ? err : "Failed to load currencies. Please try again.";
      currencies = [];
    } finally {
      isLoading = false;
    }
  }

  async function viewCurrencyDetails(currency: any) {
    // Use currencyid for reliable lookups
    const currencyDef = currency.currencydefinition || currency;
    const currencyId = currencyDef?.currencyid;
    
    if (!currencyId) {
      detailsError = "Invalid currency ID";
      return;
    }

    isViewingDetails = true;
    isLoadingDetails = true;
    detailsError = null;
    selectedCurrency = null;

    try {
      const chainParam = getChainParam(connectionState?.selectedChain);
      console.log("CurrencyListCard: Loading currency details for chain:", connectionState?.selectedChain, "param:", chainParam);
      
      const result = await invoke("get_currency", { 
        currencyName: currencyId,
        height: null,
        chain: chainParam
      });

      selectedCurrency = result;
    } catch (err) {
      console.error("Failed to load currency details:", err);
      detailsError = typeof err === 'string' ? err : "Failed to load currency details. Please try again.";
    } finally {
      isLoadingDetails = false;
    }
  }

  function closeDetailsModal() {
    isViewingDetails = false;
    selectedCurrency = null;
    detailsError = null;
  }

  function getSystemTypeDisplay(currency: any): string {
    const currencyDef = currency.currencydefinition || currency;
    const systemId = currencyDef?.systemid || '';
    const parent = currencyDef?.parent || '';
    
    if (systemId === parent) return 'Local';
    return 'Imported';
  }

  function getCurrencyType(currency: any): string {
    const currencyDef = currency.currencydefinition || currency;
    const options = currencyDef?.options || 0;
    
    // ID Control Token (special single satoshi token)
    if (options === 2080) return 'ID Control Token';
    
    // Gateway types
    if (options === 545) return 'Gateway Converter';
    if (options === 128 || options === 136) return 'Gateway';
    
    // PBaaS Chain
    if (options === 264 || options === 268) return 'PBaaS Chain';
    
    // Basket currencies
    if ([33, 35, 41, 43, 49, 51, 57, 59].includes(options)) return 'Basket Currency';
    
    // Simple tokens
    if ([32, 34, 40, 42, 48, 50, 56, 58].includes(options)) return 'Simple Token';
    
    // Default fallback
    return 'Unknown Type';
  }

  function getCurrencyTypeKey(currency: any): string {
    const type = getCurrencyType(currency);
    switch (type) {
      case 'Gateway': return 'gateway';
      case 'Gateway Converter': return 'gateway-converter';
      case 'PBaaS Chain': return 'pbaas';
      case 'Basket Currency': return 'basket';
      case 'Simple Token': return 'simple';
      case 'ID Control Token': return 'id-control';
      case 'Unknown Type': return 'unknown';
      default: return 'unknown';
    }
  }

  function getCurrencyTypeColor(type: string) {
    switch (type) {
      case 'Gateway':
        return 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200';
      case 'Gateway Converter':
        return 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200';
      case 'PBaaS Chain':
        return 'bg-indigo-100 text-indigo-800 dark:bg-indigo-900 dark:text-indigo-200';
      case 'Basket Currency':
        return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200';
      case 'Simple Token':
        return 'bg-verusidx-turquoise-light/20 text-verusidx-turquoise-deep dark:text-verusidx-turquoise-light';
      case 'ID Control Token':
        return 'bg-rose-100 text-rose-800 dark:bg-rose-900 dark:text-rose-200';
      case 'Unknown Type':
        return 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200';
      default:
        return 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200';
    }
  }

  function formatInteger(num: number): string {
    return Math.floor(num).toLocaleString();
  }

  function formatCurrency(num: number): string {
    return num.toFixed(8);
  }


  function goToPage(page: number) {
    if (page >= 1 && page <= totalPages) {
      currentPage = page;
    }
  }

  function nextPage() {
    if (currentPage < totalPages) {
      currentPage++;
    }
  }

  function prevPage() {
    if (currentPage > 1) {
      currentPage--;
    }
  }

  // Helper functions to handle filter changes and reset page
  function handleFilterChange() {
    currentPage = 1;
    loadCurrencies();
  }

  // Converter search functions
  async function performConverterSearch() {
    if (!converterCurrency1.trim()) {
      return; // Need at least one currency for converter search
    }
    
    isConverterSearchActive = true;
    currentPage = 1;
    await loadCurrencies();
  }

  function clearConverterSearch() {
    converterCurrency1 = "";
    converterCurrency2 = "";
    isConverterSearchActive = false;
    currentPage = 1;
    loadCurrencies();
  }

  function clearAllFilters() {
    searchFilter = "";
    systemTypeFilter = "all";
    launchStateFilter = "all";
    currencyTypeFilter = "all";
    fromSystemFilter = "";
    provenanceFilter = "all";
    converterCurrency1 = "";
    converterCurrency2 = "";
    isConverterSearchActive = false;
    currentPage = 1;
    loadCurrencies();
  }

  function handleKeyPressConverter(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      performConverterSearch();
    }
  }

  // Sorting functions
  function handleSort(field: string) {
    if (sortField === field) {
      // Toggle direction if same field
      sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
    } else {
      // New field, default to ascending
      sortField = field;
      sortDirection = 'asc';
    }
    currentPage = 1; // Reset to first page when sorting
  }

  // Chain change event handler - reload currencies when chain changes
  function handleChainChanged(event: CustomEvent) {
    console.log("CurrencyListCard: Chain changed to", event.detail.chainName);
    loadCurrencies();
  }

  // Load currencies when component mounts
  onMount(() => {
    loadCurrencies();
    window.addEventListener('chainChanged', handleChainChanged);
  });

  onDestroy(() => {
    window.removeEventListener('chainChanged', handleChainChanged);
  });
</script>

<div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl">
  <!-- Header -->
  <div class="p-6 border-b border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-xl font-semibold text-verusidx-stone-dark dark:text-white">Network Currencies</h2>
      <div class="flex items-center space-x-2">
        <button
          onclick={clearAllFilters}
          class="px-3 py-1 text-sm bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-stone-dark dark:text-white rounded hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-dark transition-colors"
          title="Clear all filters"
        >
          Clear Filters
        </button>
        <button
          onclick={loadCurrencies}
          class="p-2 text-verusidx-mountain-grey hover:text-verusidx-stone-dark dark:hover:text-white transition-colors"
          title="Refresh currencies"
        >
          <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Filters -->
    <div class="flex flex-col sm:flex-row gap-4">
      <div class="flex-1">
        <input
          type="text"
          bind:value={searchFilter}
          onchange={handleFilterChange}
          placeholder="Search currencies..."
          class="w-full px-3 py-2 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg text-verusidx-stone-dark dark:text-white placeholder-verusidx-mountain-grey dark:placeholder-verusidx-mountain-mist focus:ring-2 focus:ring-verusidx-mountain-blue focus:border-transparent outline-none"
        />
      </div>
      <div class="sm:w-40">
        <select
          bind:value={systemTypeFilter}
          onchange={handleFilterChange}
          class="w-full px-3 py-2 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg text-verusidx-stone-dark dark:text-white focus:ring-2 focus:ring-verusidx-mountain-blue focus:border-transparent outline-none"
        >
          <option value="all">All Systems</option>
          <option value="local">Local</option>
          <option value="imported">Imported</option>
          <option value="gateway">Gateway</option>
          <option value="pbaas">PBaaS</option>
        </select>
      </div>
      <div class="sm:w-40">
        <select
          bind:value={currencyTypeFilter}
          onchange={handleFilterChange}
          class="w-full px-3 py-2 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg text-verusidx-stone-dark dark:text-white focus:ring-2 focus:ring-verusidx-mountain-blue focus:border-transparent outline-none"
        >
          <option value="all">All Types</option>
          <option value="simple">Simple Token</option>
          <option value="basket">Basket Currency</option>
          <option value="gateway">Gateway</option>
          <option value="gateway-converter">Gateway Converter</option>
          <option value="pbaas">PBaaS Chain</option>
          <option value="id-control">ID Control Token</option>
          <option value="unknown">Unknown Type</option>
        </select>
      </div>
      <div class="sm:w-40">
        <select
          bind:value={launchStateFilter}
          onchange={handleFilterChange}
          class="w-full px-3 py-2 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg text-verusidx-stone-dark dark:text-white focus:ring-2 focus:ring-verusidx-mountain-blue focus:border-transparent outline-none"
        >
          <option value="all">All States</option>
          <option value="launched">Launched</option>
          <option value="prelaunch">Pre-launch</option>
          <option value="refund">Refund</option>
          <option value="complete">Complete</option>
        </select>
      </div>
    </div>

    <!-- Additional Filters -->
    <div class="flex flex-col sm:flex-row gap-4 mt-4">
      <div class="flex-1">
        <input
          type="text"
          bind:value={fromSystemFilter}
          onchange={handleFilterChange}
          placeholder="Filter by system (e.g., vETH, vARRR)..."
          class="w-full px-3 py-2 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg text-verusidx-stone-dark dark:text-white placeholder-verusidx-mountain-grey dark:placeholder-verusidx-mountain-mist focus:ring-2 focus:ring-verusidx-mountain-blue focus:border-transparent outline-none"
        />
      </div>
      <div class="sm:w-40">
        <select
          bind:value={provenanceFilter}
          onchange={handleFilterChange}
          class="w-full px-3 py-2 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg text-verusidx-stone-dark dark:text-white focus:ring-2 focus:ring-verusidx-mountain-blue focus:border-transparent outline-none"
        >
          <option value="all">All Provenance</option>
          <option value="1">Decentralized</option>
          <option value="2">Centralized</option>
          <option value="3">Mapped Ethereum</option>
        </select>
      </div>
    </div>

    <!-- Converter Search -->
    <div class="mt-6 pt-6 border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
      <div class="flex items-center justify-between mb-4">
        <div>
          <h3 class="text-lg font-medium text-verusidx-stone-dark dark:text-white">Find Basket Currencies</h3>
          <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
            Find currencies that allow conversion between specific reserve currencies
          </p>
        </div>
        {#if isConverterSearchActive}
          <button
            onclick={clearConverterSearch}
            class="text-sm text-verusidx-mountain-grey hover:text-verusidx-stone-dark dark:hover:text-white transition-colors"
          >
            Clear Converter Search
          </button>
        {/if}
      </div>
      
      <div class="flex flex-col sm:flex-row gap-4 items-end">
        <div class="flex-1">
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-1">
            Currency 1 (required)
          </label>
          <input
            type="text"
            bind:value={converterCurrency1}
            onkeypress={handleKeyPressConverter}
            placeholder="e.g., VRSC"
            class="w-full px-3 py-2 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg text-verusidx-stone-dark dark:text-white placeholder-verusidx-mountain-grey dark:placeholder-verusidx-mountain-mist focus:ring-2 focus:ring-verusidx-mountain-blue focus:border-transparent outline-none"
          />
        </div>
        <div class="flex-1">
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-1">
            Currency 2 (optional)
          </label>
          <input
            type="text"
            bind:value={converterCurrency2}
            onkeypress={handleKeyPressConverter}
            placeholder="e.g., DAI.vETH"
            class="w-full px-3 py-2 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg text-verusidx-stone-dark dark:text-white placeholder-verusidx-mountain-grey dark:placeholder-verusidx-mountain-mist focus:ring-2 focus:ring-verusidx-mountain-blue focus:border-transparent outline-none"
          />
        </div>
        <div class="sm:w-40">
          <button
            onclick={performConverterSearch}
            disabled={!converterCurrency1.trim()}
            class="w-full px-4 py-2 bg-verusidx-turquoise-deep text-white rounded-lg hover:bg-verusidx-turquoise-bright disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            Find Baskets
          </button>
        </div>
      </div>
      
      {#if isConverterSearchActive}
        <div class="mt-3 px-3 py-2 bg-verusidx-turquoise-light/10 border border-verusidx-turquoise-light rounded-lg">
          <p class="text-sm text-verusidx-turquoise-deep dark:text-verusidx-turquoise-light">
            <span class="font-medium">Active:</span> 
            Finding basket currencies that contain 
            <span class="font-mono">{converterCurrency1}</span>
            {#if converterCurrency2.trim()}
              and <span class="font-mono">{converterCurrency2}</span>
            {/if}
          </p>
        </div>
      {/if}
    </div>
  </div>

  <!-- Content -->
  <div class="p-6">
    {#if isLoading}
      <div class="flex items-center justify-center py-8">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-verusidx-mountain-blue"></div>
        <span class="ml-3 text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Loading currencies...</span>
      </div>
    {:else if error}
      <div class="text-center py-8">
        <div class="text-red-600 dark:text-red-400 mb-2">
          <svg class="h-12 w-12 mx-auto mb-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.732-.833-2.5 0L4.268 16.5c-.77.833.192 2.5 1.732 2.5z" />
          </svg>
        </div>
        <p class="text-red-600 dark:text-red-400 font-medium">{error}</p>
        <button
          onclick={loadCurrencies}
          class="mt-4 px-4 py-2 bg-verusidx-mountain-blue text-white rounded-lg hover:bg-verusidx-lake-blue transition-colors"
        >
          Try Again
        </button>
      </div>
    {:else if paginatedCurrencies.length === 0}
      <div class="text-center py-8">
        <div class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-2">
          <svg class="h-12 w-12 mx-auto mb-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1" />
          </svg>
        </div>
        <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist font-medium">
          {currencies.length === 0 ? 'No currencies found' : 'No currencies match your filters'}
        </p>
        <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-2">
          {currencies.length === 0 ? 'The network may be loading or no currencies are available' : 'Try adjusting your search or filter criteria'}
        </p>
      </div>
    {:else}
      <div class="space-y-4">
        <!-- Sorting Controls -->
        <div class="flex justify-between items-center">
          <div class="flex items-center space-x-4">
            <span class="text-sm font-medium text-verusidx-stone-dark dark:text-white">Sort by:</span>
            <div class="flex space-x-2">
              <button
                onclick={() => handleSort('name')}
                class="px-3 py-1 text-sm rounded border transition-colors {sortField === 'name' ? 'bg-verusidx-mountain-blue text-white border-verusidx-mountain-blue' : 'bg-white dark:bg-verusidx-stone-medium text-verusidx-stone-dark dark:text-white border-verusidx-mountain-mist dark:border-verusidx-stone-medium hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-dark'}"
              >
                Name
                {#if sortField === 'name'}
                  <span class="ml-1">{sortDirection === 'asc' ? '↑' : '↓'}</span>
                {/if}
              </button>
              <button
                onclick={() => handleSort('supply')}
                class="px-3 py-1 text-sm rounded border transition-colors {sortField === 'supply' ? 'bg-verusidx-mountain-blue text-white border-verusidx-mountain-blue' : 'bg-white dark:bg-verusidx-stone-medium text-verusidx-stone-dark dark:text-white border-verusidx-mountain-mist dark:border-verusidx-stone-medium hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-dark'}"
              >
                Supply
                {#if sortField === 'supply'}
                  <span class="ml-1">{sortDirection === 'asc' ? '↑' : '↓'}</span>
                {/if}
              </button>
              <button
                onclick={() => handleSort('startblock')}
                class="px-3 py-1 text-sm rounded border transition-colors {sortField === 'startblock' ? 'bg-verusidx-mountain-blue text-white border-verusidx-mountain-blue' : 'bg-white dark:bg-verusidx-stone-medium text-verusidx-stone-dark dark:text-white border-verusidx-mountain-mist dark:border-verusidx-stone-medium hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-dark'}"
              >
                Start Block
                {#if sortField === 'startblock'}
                  <span class="ml-1">{sortDirection === 'asc' ? '↑' : '↓'}</span>
                {/if}
              </button>
            </div>
          </div>
        </div>
        
        <!-- Results Info -->
        <div class="flex justify-between items-center text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
          <span>Showing {((currentPage - 1) * itemsPerPage) + 1} to {Math.min(currentPage * itemsPerPage, filteredCurrencies.length)} of {filteredCurrencies.length} currencies</span>
          <span>Page {currentPage} of {totalPages}</span>
        </div>
        
        <div class="grid gap-4">
          {#each paginatedCurrencies as currency}
            {@const currencyDef = currency.currencydefinition || currency}
            {@const name = currencyDef?.fullyqualifiedname || currencyDef?.name || 'Unknown'}
            {@const currencyId = currencyDef?.currencyid || 'Unknown'}
            {@const startBlock = currencyDef?.startblock || 0}
            {@const systemType = getSystemTypeDisplay(currency)}
            {@const currencyType = getCurrencyType(currency)}
            {@const supply = currency?.bestcurrencystate?.supply || 0}
            {@const flags = currencyDef?.bestcurrencystate?.flags || currency?.bestcurrencystate?.flags}
            
            <div class="border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg p-4 hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors">
              <div class="flex items-start justify-between">
                <div class="flex-1">
                  <div class="flex items-center space-x-3 mb-3">
                    <h3 class="text-lg font-medium text-verusidx-stone-dark dark:text-white">{name}</h3>
                    <span class="px-2 py-1 text-xs rounded-full {getCurrencyTypeColor(currencyType)}">
                      {currencyType}
                    </span>
                    {#if flags === 37}
                      <span class="px-2 py-1 text-xs rounded-full bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200">
                        Bricked
                      </span>
                    {/if}
                    {#if flags === 2 || flags === 3}
                      <span class="px-2 py-1 text-xs rounded-full bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200">
                        Preconversion
                      </span>
                    {/if}
                  </div>
                  
                  <div class="space-y-1 text-sm">
                    <div>
                      <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Currency ID:</span>
                      <span class="font-mono text-verusidx-stone-dark dark:text-white ml-2">{currencyId}</span>
                    </div>
                    <div>
                      <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Start Block:</span>
                      <span class="text-verusidx-stone-dark dark:text-white ml-2">{formatInteger(startBlock)}</span>
                    </div>
                    <div>
                      <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Supply:</span>
                      <span class="text-verusidx-stone-dark dark:text-white ml-2">{formatCurrency(supply)}</span>
                    </div>
                  </div>
                </div>
                
                <div class="flex items-center space-x-2 ml-4">
                  <button
                    onclick={() => viewCurrencyDetails(currency)}
                    class="px-3 py-1 text-sm bg-verusidx-mountain-blue text-white rounded hover:bg-verusidx-lake-blue transition-colors"
                  >
                    View Details
                  </button>
                </div>
              </div>
            </div>
          {/each}
        </div>

        <!-- Pagination Controls -->
        {#if totalPages > 1}
          <div class="flex items-center justify-between mt-6">
            <div class="flex items-center space-x-2">
              <button
                onclick={prevPage}
                disabled={currentPage === 1}
                class="px-3 py-1 text-sm bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-stone-dark dark:text-white rounded hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-dark disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                Previous
              </button>
              
              <!-- Page numbers -->
              <div class="flex items-center space-x-1">
                {#each Array(Math.min(5, totalPages)) as _, i}
                  {@const pageNum = Math.max(1, Math.min(totalPages - 4, currentPage - 2)) + i}
                  {#if pageNum <= totalPages}
                    <button
                      onclick={() => goToPage(pageNum)}
                      class="px-3 py-1 text-sm rounded transition-colors {currentPage === pageNum ? 'bg-verusidx-mountain-blue text-white' : 'bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-stone-dark dark:text-white hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-dark'}"
                    >
                      {pageNum}
                    </button>
                  {/if}
                {/each}
              </div>
              
              <button
                onclick={nextPage}
                disabled={currentPage === totalPages}
                class="px-3 py-1 text-sm bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-stone-dark dark:text-white rounded hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-dark disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                Next
              </button>
            </div>
            
            <div class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
              {itemsPerPage} per page
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>

<!-- Currency Details Modal -->
{#if isViewingDetails}
  <Modal isOpen={isViewingDetails} title="Currency Details" size="lg" onclose={closeDetailsModal}>
    <CurrencyDetailsView 
      currencyData={selectedCurrency} 
      isLoading={isLoadingDetails}
      error={detailsError}
    />
  </Modal>
{/if}