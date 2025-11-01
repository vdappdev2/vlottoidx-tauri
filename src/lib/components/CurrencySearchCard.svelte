<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { connectionStore, getChainParam } from "$lib/stores/connection";
  import { onMount, onDestroy } from "svelte";
  import CurrencyDetailsView from "./CurrencyDetailsView.svelte";

  let searchQuery = $state("");
  let searchResults = $state<any>(null);
  let isSearching = $state(false);
  let searchError = $state<string | null>(null);
  let hasSearched = $state(false);
  let connectionState = $state<any>(null);

  // Recent searches (could be enhanced with persistence)
  let recentSearches = $state<string[]>([]);

  connectionStore.subscribe(state => {
    connectionState = state;
  });

  async function handleSearch() {
    if (!searchQuery.trim()) {
      searchError = "Please enter a currency name to search";
      return;
    }

    isSearching = true;
    searchError = null;
    hasSearched = false;

    try {
      const chainParam = getChainParam(connectionState?.selectedChain);
      console.log("CurrencySearchCard: Searching for currency on chain:", connectionState?.selectedChain, "param:", chainParam);
      
      const result = await invoke("get_currency", { 
        currencyName: searchQuery.trim(),
        height: null,
        chain: chainParam
      });

      console.log("CurrencySearchCard: get_currency result:", result);
      searchResults = result;
      hasSearched = true;
      
      // Add to recent searches (avoid duplicates)
      if (!recentSearches.includes(searchQuery.trim())) {
        recentSearches = [searchQuery.trim(), ...recentSearches.slice(0, 4)]; // Keep only 5 recent
      }
    } catch (error) {
      console.error("Currency search failed:", error);
      searchError = typeof error === 'string' ? error : "Failed to find currency. Please check the name and try again.";
      searchResults = null;
      hasSearched = true;
    } finally {
      isSearching = false;
    }
  }

  function handleKeyPress(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      handleSearch();
    }
  }

  function searchRecent(currency: string) {
    searchQuery = currency;
    handleSearch();
  }

  function clearSearch() {
    searchQuery = "";
    searchResults = null;
    searchError = null;
    hasSearched = false;
  }

  // Chain change event handler - clear search results when chain changes
  function handleChainChanged(event: CustomEvent) {
    console.log("CurrencySearchCard: Chain changed to", event.detail.chainName);
    // Clear search results since they're from the previous chain
    if (hasSearched || searchResults) {
      clearSearch();
    }
  }

  // Set up chain change event listener
  onMount(() => {
    window.addEventListener('chainChanged', handleChainChanged);
  });

  onDestroy(() => {
    window.removeEventListener('chainChanged', handleChainChanged);
  });
</script>

<div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl">
  <!-- Search Header -->
  <div class="p-6 border-b border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-xl font-semibold text-verusidx-stone-dark dark:text-white">Currency Lookup</h2>
      {#if hasSearched}
        <button
          onclick={clearSearch}
          class="text-sm text-verusidx-mountain-grey hover:text-verusidx-stone-dark dark:hover:text-white transition-colors"
        >
          Clear Search
        </button>
      {/if}
    </div>

    <!-- Search Input -->
    <div class="relative">
      <div class="flex">
        <input
          type="text"
          bind:value={searchQuery}
          onkeypress={handleKeyPress}
          placeholder="Search any currency on the network..."
          class="flex-1 px-4 py-3 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-l-lg text-verusidx-stone-dark dark:text-white placeholder-verusidx-mountain-grey dark:placeholder-verusidx-mountain-mist focus:ring-2 focus:ring-verusidx-mountain-blue focus:border-transparent outline-none"
        />
        <button
          onclick={handleSearch}
          disabled={isSearching || !searchQuery.trim()}
          class="px-6 py-3 bg-verusidx-mountain-blue text-white rounded-r-lg hover:bg-verusidx-lake-blue disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          {#if isSearching}
            <svg class="animate-spin h-5 w-5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
          {:else}
            <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          {/if}
        </button>
      </div>
    </div>

    <!-- Recent Searches -->
    {#if recentSearches.length > 0 && !hasSearched}
      <div class="mt-4">
        <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-2">Recent searches:</p>
        <div class="flex flex-wrap gap-2">
          {#each recentSearches as recent}
            <button
              onclick={() => searchRecent(recent)}
              class="px-3 py-1 text-sm bg-verusidx-turquoise-light/20 text-verusidx-turquoise-deep dark:text-verusidx-turquoise-light rounded-full hover:bg-verusidx-turquoise-light/30 transition-colors"
            >
              {recent}
            </button>
          {/each}
        </div>
      </div>
    {/if}
  </div>

  <!-- Search Results -->
  <div class="p-6">
    {#if isSearching}
      <div class="flex items-center justify-center py-8">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-verusidx-mountain-blue"></div>
        <span class="ml-3 text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Searching currency...</span>
      </div>
    {:else if searchError}
      <div class="text-center py-8">
        <div class="text-red-600 dark:text-red-400 mb-2">
          <svg class="h-12 w-12 mx-auto mb-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.732-.833-2.5 0L4.268 16.5c-.77.833.192 2.5 1.732 2.5z" />
          </svg>
        </div>
        <p class="text-red-600 dark:text-red-400 font-medium">{searchError}</p>
        <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-2">
          Make sure the currency name is correct
        </p>
      </div>
    {:else if hasSearched && searchResults}
      <div class="space-y-4">
        <div class="flex items-center justify-between">
          <h3 class="text-lg font-medium text-verusidx-stone-dark dark:text-white">
            Search Results for "{searchQuery}"
          </h3>
          <span class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
            Found currency
          </span>
        </div>
        <CurrencyDetailsView currencyData={searchResults} />
      </div>
    {:else if hasSearched && !searchResults}
      <div class="text-center py-8">
        <div class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-2">
          <svg class="h-12 w-12 mx-auto mb-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.172 16.172a4 4 0 015.656 0M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
        </div>
        <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist font-medium">No results found</p>
        <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-2">
          The currency "{searchQuery}" could not be found on the network
        </p>
      </div>
    {:else}
      <div class="text-center py-8">
        <div class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-2">
          <svg class="h-12 w-12 mx-auto mb-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
        </div>
        <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist font-medium">Search for any currency on the network</p>
        <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-2">
          Enter a currency name (e.g., "VRSC", "Bridge.vETH") to view its details
        </p>
      </div>
    {/if}
  </div>
</div>