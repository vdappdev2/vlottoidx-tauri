<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { connectionStore, getChainParam, type ConnectionState } from "$lib/stores/connection";
  import { goto } from "$app/navigation";
  import { onMount, onDestroy } from "svelte";
  import { BlockHeightHeader, OfferCard, OfferDetailsModal } from "$lib/components";
  import MarketplaceOfferModal from "$lib/components/MarketplaceOfferModal.svelte";
  import IdentityModal from "$lib/components/IdentityModal.svelte";
  import CurrencyModal from "$lib/components/CurrencyModal.svelte";
  import { clearNameCache } from "$lib/stores/nameCache";
  import type { ParsedOffer } from "$lib/utils/offerUtils";
  import { showSuccess, showError } from "$lib/services/notifications";

  let connectionState = $state<ConnectionState>({ current: null, isConnecting: false, lastError: null, availableChains: [], selectedChain: null });
  let userOffers = $state<any[]>([]);
  let searchResults = $state<any[]>([]);
  let isLoadingOffers = $state(true);
  let isSearching = $state(false);
  let error = $state<string | null>(null);

  // Enhanced offer management state
  let selectedOffers = $state<Set<string>>(new Set());
  let isClosingOffer = $state<string | null>(null);
  let isClosingMultiple = $state(false);
  let offerFilter = $state<'all' | 'active' | 'expired'>('all');
  let offerSort = $state<'newest' | 'expiry' | 'amount'>('newest');

  // Search form state
  let searchForm = $state({
    type: 'identity', // 'identity' or 'currency' for marketplace offer search
    query: '',
    showExpired: true
  });
  
  // New sorting and filtering state
  let sortField = $state<'expiry' | 'price' | null>(null);
  let sortDirection = $state<'asc' | 'desc'>('asc');
  let nameFilter = $state('');
  
  // Search results management
  let searchResultsPage = $state(1);
  let searchResultsPerPage = $state(10);
  let hasSearched = $state(false);

  // User offers pagination
  let userOffersPage = $state(1);
  let userOffersPerPage = $state(10);

  // Modal state
  let isOfferModalOpen = $state(false);
  let offerModalMode = $state<'make' | 'take'>('make');
  let selectedOffer = $state<any>(null);
  
  // Offer details modal state
  let isOfferDetailsModalOpen = $state(false);
  let selectedOfferForDetails = $state<any>(null);

  // Identity/Currency modal state (moved from OfferDetailsModal)
  let showIdentityModal = $state(false);
  let showCurrencyModal = $state(false);
  let selectedIdentityAddress = $state('');
  let selectedCurrencyAddress = $state('');

  // Subscribe to connection state
  connectionStore.subscribe(state => {
    connectionState = state;
    if (!state.current?.isConnected) {
      goto("/");
    } else if (state.selectedChain) {
      loadUserOffers();
    }
  });

  async function loadUserOffers() {
    if (!connectionState.current?.isConnected || !connectionState.selectedChain) return;

    isLoadingOffers = true;
    error = null;
    
    try {
      const chainParam = getChainParam(connectionState.selectedChain);
      console.log("Marketplace loading user offers for chain:", connectionState.selectedChain, "param:", chainParam);
      const result = await invoke('list_open_offers', { chain: chainParam });
      console.log('User offers result:', result);
      userOffers = Array.isArray(result) ? result : [];
    } catch (err) {
      console.error('Failed to load user offers:', err);
      error = typeof err === 'string' ? err : 'Failed to load offers';
      userOffers = [];
    } finally {
      isLoadingOffers = false;
    }
  }

  async function handleSearch(event: SubmitEvent) {
    event.preventDefault();
    if (!searchForm.query.trim() || !connectionState.selectedChain) return;

    isSearching = true;
    searchResults = [];
    error = null;
    searchResultsPage = 1; // Reset to first page
    hasSearched = false;

    try {
      let result: any;
      
      // Search for marketplace offers by identity or currency
      const chainParam = getChainParam(connectionState.selectedChain);
      console.log("Marketplace searching offers for chain:", connectionState.selectedChain, "param:", chainParam);
      const isCurrency = searchForm.type === 'currency';
      result = await invoke('get_offers', { 
        currencyOrId: searchForm.query, 
        isCurrency: isCurrency,
        withTx: false,
        chain: chainParam
      });
      
      let offers: any[] = [];
      
      // Process getoffers result which returns categories of offers
      // Inject category into each offer for context
      if (result && typeof result === 'object') {
        Object.entries(result).forEach(([category, categoryOffers]) => {
          if (Array.isArray(categoryOffers)) {
            categoryOffers.forEach(offer => {
              offers.push({
                ...offer,
                _category: category  // Inject category for parsing context
              });
            });
          }
        });
      }
      
      searchResults = offers;
      console.log('Raw search results with injected category:', searchResults);
      
      // Log the structure of the first offer for debugging
      if (searchResults.length > 0) {
        console.log('First offer structure:', searchResults[0]);
        console.log('Offer.offer:', searchResults[0].offer);
        console.log('Offer.offer.txid:', searchResults[0].offer?.txid);
      }
      
      console.log('Search results:', searchResults);
      hasSearched = true;
    } catch (err) {
      console.error('Search failed:', err);
      error = typeof err === 'string' ? err : 'Search failed';
      hasSearched = true;
    } finally {
      isSearching = false;
    }
  }

  function isOfferExpiredFromSearch(offer: any): boolean {
    // Check if offer is expired - this would need proper block height comparison
    return offer.offer?.expired === true;
  }

  // Reactive computed values for filtering, sorting, and pagination
  let filteredAndSortedSearchResults = $derived.by(() => {
    let filtered = searchResults.filter(offer => {
      // Filter by expiry
      if (!searchForm.showExpired && isOfferExpiredFromSearch(offer)) {
        return false;
      }

      return true;
    });

    // Filter by name if nameFilter is provided
    if (nameFilter.trim()) {
      const searchTerm = nameFilter.toLowerCase();
      filtered = filtered.filter(offer => {
        // Check identity names
        if (offer.identityid && offer.identityid.toLowerCase().includes(searchTerm)) return true;
        if (offer.offer?.name && offer.offer.name.toLowerCase().includes(searchTerm)) return true;
        // Check currency names and i-addresses
        if (offer.currencyid && offer.currencyid.toLowerCase().includes(searchTerm)) return true;
        // Check any i-address in the offer/accept objects
        const offerObj = offer.offer?.offer || {};
        const acceptObj = offer.offer?.accept || {};
        const allKeys = [...Object.keys(offerObj), ...Object.keys(acceptObj)];
        return allKeys.some(key => key.toLowerCase().includes(searchTerm));
      });
    }

    // Sort offers based on toggle buttons
    if (sortField) {
      filtered.sort((a, b) => {
        let result = 0;
        if (sortField === 'expiry') {
          const aExpiry = a.offer?.blockexpiry || 0;
          const bExpiry = b.offer?.blockexpiry || 0;
          result = aExpiry - bExpiry;
        } else if (sortField === 'price') {
          result = (a.price || 0) - (b.price || 0);
        }
        return sortDirection === 'desc' ? -result : result;
      });
    } else {
      // Default sort by newest (txid)
      filtered.sort((a, b) => (b.offer?.txid || '').localeCompare(a.offer?.txid || ''));
    }

    return filtered;
  });

  let paginatedSearchResults = $derived(
    filteredAndSortedSearchResults.slice(
      (searchResultsPage - 1) * searchResultsPerPage,
      searchResultsPage * searchResultsPerPage
    )
  );

  let totalSearchPages = $derived(Math.ceil(filteredAndSortedSearchResults.length / searchResultsPerPage));

  function clearSearch() {
    searchForm.query = '';
    searchResults = [];
    error = null;
    hasSearched = false;
    // Reset filters
    sortField = null;
    sortDirection = 'asc';
    nameFilter = '';
  }

  function toggleSort(field: 'expiry' | 'price') {
    if (sortField === field) {
      // Same field - toggle direction
      sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
    } else {
      // New field - set to ascending
      sortField = field;
      sortDirection = 'asc';
    }
    // Reset to first page when sorting changes
    searchResultsPage = 1;
  }

  function handleNameFilterChange() {
    // Reset to first page when filter changes
    searchResultsPage = 1;
  }

  // Reset to page 1 when results per page changes
  $effect(() => {
    searchResultsPerPage;
    searchResultsPage = 1;
  });

  // Reset to page 1 when user offers per page changes
  $effect(() => {
    userOffersPerPage;
    userOffersPage = 1;
  });

  async function closeOffer(txid: string) {
    if (!connectionState.selectedChain || isClosingOffer === txid) return;
    
    isClosingOffer = txid;
    error = null;
    
    try {
      const chainParam = getChainParam(connectionState.selectedChain);
      console.log("Marketplace closing offer for chain:", connectionState.selectedChain, "param:", chainParam);
      const result = await invoke('close_offers', { 
        offerTxIds: [txid],
        destinationAddress: null,
        chain: chainParam
      });
      console.log('Close offer result:', result);
      
      // Show success toast
      showSuccess('Success: offer closed');
      
      // Remove from selected offers if it was selected
      selectedOffers.delete(txid);
      selectedOffers = new Set(selectedOffers);
      
      // Reload offers after closing
      await loadUserOffers();
    } catch (err) {
      console.error('Failed to close offer:', err);
      error = typeof err === 'string' ? err : 'Failed to close offer';
      showError(error);
    } finally {
      isClosingOffer = null;
    }
  }

  async function closeSelectedOffers() {
    if (!connectionState.selectedChain || selectedOffers.size === 0 || isClosingMultiple) return;
    
    isClosingMultiple = true;
    error = null;
    
    try {
      const chainParam = getChainParam(connectionState.selectedChain);
      console.log("Marketplace closing selected offers for chain:", connectionState.selectedChain, "param:", chainParam);
      const offerIds = Array.from(selectedOffers);
      const result = await invoke('close_offers', { 
        offerTxIds: offerIds,
        destinationAddress: null,
        chain: chainParam
      });
      console.log('Close multiple offers result:', result);
      
      // Show success toast
      showSuccess('Success: offers closed');
      
      // Clear selected offers
      selectedOffers.clear();
      selectedOffers = new Set(selectedOffers);
      
      // Reload offers after closing
      await loadUserOffers();
    } catch (err) {
      console.error('Failed to close selected offers:', err);
      error = typeof err === 'string' ? err : 'Failed to close selected offers';
      showError(error);
    } finally {
      isClosingMultiple = false;
    }
  }

  function toggleOfferSelection(txid: string) {
    if (selectedOffers.has(txid)) {
      selectedOffers.delete(txid);
    } else {
      selectedOffers.add(txid);
    }
    selectedOffers = new Set(selectedOffers);
  }

  function selectAllOffers() {
    filteredAndSortedOffers.forEach(offer => selectedOffers.add(offer.txid));
    selectedOffers = new Set(selectedOffers);
  }

  function clearSelection() {
    selectedOffers.clear();
    selectedOffers = new Set(selectedOffers);
  }

  // Utility functions to parse listopenoffers data structure (for user offers)
  function extractOffering(offerData: any): { type: 'identity' | 'currency' | 'native', name: string, amount?: number, identityAddress?: string } {
    const offer = offerData.offer;
    
    // Check for identity being offered
    if (offer?.identityprimary?.name) {
      return { 
        type: 'identity', 
        name: offer.identityprimary.name,
        identityAddress: offer.identityprimary.identityaddress
      };
    }
    
    // Check for currency being offered via reserve_balance
    if (offer?.reserve_balance) {
      const entries = Object.entries(offer.reserve_balance);
      if (entries.length > 0) {
        const [currency, amount] = entries[0];
        return { type: 'currency', name: currency, amount: amount as number };
      }
    }
    
    // Check for currency being offered via commitmenthash
    if (offer?.commitmenthash?.currencyvalues) {
      const entries = Object.entries(offer.commitmenthash.currencyvalues);
      if (entries.length > 0) {
        const [currencyId, amount] = entries[0];
        // Try to get friendly name from reserve_balance if available
        const friendlyName = offer.reserve_balance ? Object.keys(offer.reserve_balance)[0] : currencyId;
        return { type: 'currency', name: friendlyName, amount: amount as number };
      }
    }
    
    // Check for native currency being offered
    if (offer?.nativeout && offer.nativeout > 0) {
      return { type: 'native', name: 'VRSC', amount: offer.nativeout };
    }
    
    return { type: 'currency', name: 'Unknown' };
  }

  // Note: Parsing functions for offers have been moved to offerUtils.ts

  function extractAccepting(offerData: any): { type: 'identity' | 'currency' | 'native', name: string, amount?: number, identityAddress?: string } {
    const forData = offerData.for;
    
    // Check for identity being accepted
    if (forData?.identityprimary?.name) {
      return { 
        type: 'identity', 
        name: forData.identityprimary.name,
        identityAddress: forData.identityprimary.identityaddress
      };
    }
    
    // Check for currency being accepted via reserve_balance
    if (forData?.reserve_balance) {
      const entries = Object.entries(forData.reserve_balance);
      if (entries.length > 0) {
        const [currency, amount] = entries[0];
        return { type: 'currency', name: currency, amount: amount as number };
      }
    }
    
    // Check for currency being accepted via reserveoutput
    if (forData?.reserveoutput?.currencyvalues) {
      const entries = Object.entries(forData.reserveoutput.currencyvalues);
      if (entries.length > 0) {
        const [currencyId, amount] = entries[0];
        // Try to get friendly name from reserve_balance if available
        const friendlyName = forData.reserve_balance ? Object.keys(forData.reserve_balance)[0] : currencyId;
        return { type: 'currency', name: friendlyName, amount: amount as number };
      }
    }
    
    // Check for native currency being accepted
    if (forData?.nativeout && forData.nativeout > 0) {
      return { type: 'native', name: 'VRSC', amount: forData.nativeout };
    }
    
    return { type: 'currency', name: 'Unknown' };
  }

  function formatCryptoAmount(amount: number): string {
    // For very small amounts (less than 0.0001), show full precision up to 8 decimal places
    if (amount < 0.0001 && amount > 0) {
      // Use toFixed(8) and manually remove trailing zeros to avoid scientific notation
      let formatted = amount.toFixed(8);
      // Remove trailing zeros and unnecessary decimal point
      formatted = formatted.replace(/\.?0+$/, '');
      // If we removed all decimals, ensure we keep at least one zero for clarity
      if (!formatted.includes('.')) {
        formatted += '.0';
      }
      return formatted;
    }
    // For larger amounts, use normal locale formatting
    return amount.toLocaleString();
  }

  function formatOfferDisplay(offering: any, accepting: any): string {
    const offeringText = offering.amount ? `${formatCryptoAmount(offering.amount)} ${offering.name}` : offering.name;
    const acceptingText = accepting.amount ? `${formatCryptoAmount(accepting.amount)} ${accepting.name}` : accepting.name;
    return `${offeringText} ↔ ${acceptingText}`;
  }

  function formatTimeRemaining(blockExpiry: number): string {
    // This is a simplified calculation - in reality you'd need current block height
    // and average block time to calculate actual time remaining
    return blockExpiry > 0 ? `Block ${blockExpiry.toLocaleString()}` : 'Expired';
  }

  function getOfferType(offer: any): 'ID→ID' | 'ID→Currency' | 'Currency→ID' | 'Currency→Currency' | 'Unknown' {
    if (!offer) return 'Unknown';
    
    const offering = extractOffering(offer);
    const accepting = extractAccepting(offer);
    
    if (offering.type === 'identity' && accepting.type === 'identity') return 'ID→ID';
    if (offering.type === 'identity' && (accepting.type === 'currency' || accepting.type === 'native')) return 'ID→Currency';
    if ((offering.type === 'currency' || offering.type === 'native') && accepting.type === 'identity') return 'Currency→ID';
    if ((offering.type === 'currency' || offering.type === 'native') && (accepting.type === 'currency' || accepting.type === 'native')) return 'Currency→Currency';
    
    return 'Unknown';
  }

  function isOfferExpired(offer: any): boolean {
    if (!offer?.expires) return false;
    // For now, simplified check - in reality would compare with current block height
    // We could add logic here to compare with actual current block height when available
    return false; // All offers from listopenoffers are considered active unless we have block height comparison
  }

  // Reactive computed values for user offers filtering, sorting, and pagination
  let filteredAndSortedOffers = $derived.by(() => {
    let filtered = userOffers.filter(offer => {
      if (offerFilter === 'active') return !isOfferExpired(offer);
      if (offerFilter === 'expired') return isOfferExpired(offer);
      return true; // 'all'
    });

    return filtered.sort((a, b) => {
      switch (offerSort) {
        case 'expiry':
          const aExpiry = a.expires || 0;
          const bExpiry = b.expires || 0;
          return aExpiry - bExpiry;
        case 'amount':
          // Get amount from offering side
          const aOffering = extractOffering(a);
          const bOffering = extractOffering(b);
          const aAmount = aOffering.amount || 0;
          const bAmount = bOffering.amount || 0;
          return bAmount - aAmount;
        case 'newest':
        default:
          // Sort by transaction ID as a proxy for newest (could be improved with timestamps)
          return (b.txid || '').localeCompare(a.txid || '');
      }
    });
  });

  let paginatedUserOffers = $derived(
    filteredAndSortedOffers.slice(
      (userOffersPage - 1) * userOffersPerPage,
      userOffersPage * userOffersPerPage
    )
  );

  let totalUserOffersPages = $derived(Math.ceil(filteredAndSortedOffers.length / userOffersPerPage));

  function openMakeOfferModal() {
    offerModalMode = 'make';
    selectedOffer = null;
    isOfferModalOpen = true;
  }

  function openTakeOfferModal(offer: any) {
    offerModalMode = 'take';
    selectedOffer = offer;
    isOfferModalOpen = true;
  }

  function closeOfferModal() {
    isOfferModalOpen = false;
    selectedOffer = null;
  }

  function openOfferDetailsModal(offerResult: any) {
    selectedOfferForDetails = offerResult;
    isOfferDetailsModalOpen = true;
  }

  function closeOfferDetailsModal() {
    isOfferDetailsModalOpen = false;
    selectedOfferForDetails = null;
  }

  // Identity/Currency modal functions (moved from OfferDetailsModal)
  function openIdentityModal(identityAddress: string) {
    selectedIdentityAddress = identityAddress;
    showIdentityModal = true;
  }

  function closeIdentityModal() {
    showIdentityModal = false;
    selectedIdentityAddress = '';
  }

  function openCurrencyModal(currencyAddress: string) {
    selectedCurrencyAddress = currencyAddress;
    showCurrencyModal = true;
  }

  function closeCurrencyModal() {
    showCurrencyModal = false;
    selectedCurrencyAddress = '';
  }

  function handleViewOfferDetails(parsedOffer: ParsedOffer) {
    // Find the original offer result that matches this parsed offer
    console.log('Looking for offer with txId:', parsedOffer.txId);
    console.log('searchResults:', searchResults);
    
    const offerResult = searchResults.find(offer => {
      const txid = offer.offer?.txid;
      console.log('Checking offer txid:', txid, 'against:', parsedOffer.txId);
      return txid === parsedOffer.txId;
    });
    
    if (offerResult) {
      console.log('Found matching offer:', offerResult);
      openOfferDetailsModal(offerResult);
    } else {
      console.error('Could not find offer with txId:', parsedOffer.txId);
      console.error('Available txids:', searchResults.map(o => o.offer?.txid));
    }
  }

  function handleOfferSuccess() {
    // Reload offers after successful offer operation
    loadUserOffers();
    // Optionally refresh search results if they exist
    if (searchResults.length > 0) {
      handleSearch(new Event('submit') as SubmitEvent);
    }
  }

  // Chain change event handler for global chain switching
  function handleChainChanged(event: CustomEvent) {
    console.log("Marketplace: Chain changed to", event.detail.chainName);
    // Clear cached friendly names since they're chain-specific
    clearNameCache();
    // Reload user offers and search results for new chain
    loadUserOffers();
    if (searchResults.length > 0) {
      handleSearch(new Event('submit') as SubmitEvent);
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

<svelte:head>
  <title>Marketplace Hub - VerusIDX</title>
  <meta name="description" content="Trading hub for identities and currencies" />
</svelte:head>

<div class="bg-verusidx-sky-soft dark:bg-verusidx-lake-deep min-h-screen max-h-screen overflow-y-auto">
  <!-- Block Height Header -->
  <BlockHeightHeader />

  <div class="max-w-7xl mx-auto p-8 pb-16 space-y-8">
    <!-- Page Header -->
    <div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl p-6">
      <div class="flex justify-between items-center">
        <div>
          <h1 class="text-3xl font-bold text-verusidx-stone-dark dark:text-white">Marketplace Hub</h1>
          <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
            Trade identities and currencies • View and manage your offers
          </p>
        </div>
        <a 
          href="/dashboard"
          class="px-4 py-2 bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-stone-medium dark:text-verusidx-mountain-mist rounded-lg hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-dark transition-colors"
        >
          Back to Dashboard
        </a>
      </div>
    </div>

    <!-- Error Display -->
    {#if error}
      <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
        <p class="text-red-700 dark:text-red-300">{error}</p>
      </div>
    {/if}

    <!-- Your Active Offers (Primary Display) -->
    <div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl p-6">
      <div class="flex justify-between items-center mb-6">
        <h2 class="text-xl font-bold text-verusidx-stone-dark dark:text-white">
          Your Active Offers ({filteredAndSortedOffers.length})
        </h2>
        <div class="flex items-center space-x-4">
          <!-- Results per page -->
          {#if userOffers.length > 0}
            <div class="flex items-center space-x-2">
              <label class="text-sm text-verusidx-stone-dark dark:text-white">Show:</label>
              <select
                bind:value={userOffersPerPage}
                class="px-2 py-1 text-sm border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
              >
                <option value={5}>5</option>
                <option value={10}>10</option>
                <option value={20}>20</option>
                <option value={50}>50</option>
              </select>
              <span class="text-sm text-verusidx-stone-dark dark:text-white">per page</span>
            </div>
          {/if}
          <button
            onclick={loadUserOffers}
            disabled={isLoadingOffers}
            class="px-4 py-2 text-sm bg-verusidx-mountain-blue hover:bg-verusidx-lake-blue text-white rounded-lg transition-colors disabled:opacity-50"
          >
            {isLoadingOffers ? 'Loading...' : 'Refresh'}
          </button>
        </div>
      </div>

      <!-- Filter and Sort Controls -->
      {#if userOffers.length > 0}
        <div class="flex flex-wrap gap-4 mb-6 p-4 bg-verusidx-snow-ice dark:bg-verusidx-stone-medium rounded-lg">
          <!-- Filter -->
          <div class="flex items-center space-x-2">
            <label class="text-sm font-medium text-verusidx-stone-dark dark:text-white">Filter:</label>
            <select 
              bind:value={offerFilter}
              class="px-3 py-1 text-sm border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
            >
              <option value="all">All Offers</option>
              <option value="active">Active Only</option>
              <option value="expired">Expired Only</option>
            </select>
          </div>

          <!-- Sort -->
          <div class="flex items-center space-x-2">
            <label class="text-sm font-medium text-verusidx-stone-dark dark:text-white">Sort:</label>
            <select 
              bind:value={offerSort}
              class="px-3 py-1 text-sm border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
            >
              <option value="newest">Newest First</option>
              <option value="expiry">By Expiry</option>
              <option value="amount">By Amount</option>
            </select>
          </div>

          <!-- Selection Controls -->
          {#if filteredAndSortedOffers.length > 0}
            <div class="flex items-center space-x-2 ml-auto">
              <button
                onclick={selectAllOffers}
                class="px-3 py-1 text-sm text-verusidx-turquoise-deep hover:text-verusidx-turquoise-bright"
              >
                Select All
              </button>
              {#if selectedOffers.size > 0}
                <button
                  onclick={clearSelection}
                  class="px-3 py-1 text-sm text-verusidx-mountain-grey hover:text-verusidx-stone-dark dark:hover:text-white"
                >
                  Clear ({selectedOffers.size})
                </button>
                <button
                  onclick={closeSelectedOffers}
                  disabled={isClosingMultiple}
                  class="px-3 py-1 text-sm bg-red-100 hover:bg-red-200 text-red-700 dark:bg-red-900/20 dark:hover:bg-red-900/40 dark:text-red-300 rounded transition-colors disabled:opacity-50"
                >
                  {isClosingMultiple ? 'Closing...' : `Close Selected (${selectedOffers.size})`}
                </button>
              {/if}
            </div>
          {/if}
        </div>
      {/if}

      {#if isLoadingOffers}
        <div class="text-center py-8">
          <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-verusidx-turquoise-deep mx-auto"></div>
          <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-2">Loading your offers...</p>
        </div>
      {:else if userOffers.length === 0}
        <!-- Empty State -->
        <div class="text-center py-12">
          <div class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-6">
            <svg class="h-16 w-16 mx-auto mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
            </svg>
            <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-2">No Active Offers</h3>
            <p>You currently have no offers on the marketplace.</p>
          </div>
          <div class="space-x-4">
            <button
              onclick={openMakeOfferModal}
              class="px-6 py-3 bg-verusidx-mountain-blue hover:bg-verusidx-lake-blue text-white rounded-lg transition-colors"
            >
              Make New Offer
            </button>
            <button
              onclick={() => goto('/identities')}
              class="px-6 py-3 bg-verusidx-turquoise-deep hover:bg-verusidx-turquoise-bright text-white rounded-lg transition-colors"
            >
              Manage Identities
            </button>
          </div>
        </div>
      {:else}
        <!-- Enhanced Offers List -->
        <div class="space-y-4">
          {#each paginatedUserOffers as offer (offer.txid)}
            <div class="border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg p-4 {selectedOffers.has(offer.txid) ? 'ring-2 ring-verusidx-turquoise-deep bg-verusidx-turquoise-light/10 dark:bg-verusidx-turquoise-deep/10' : ''}">
              <div class="flex items-start space-x-4">
                <!-- Selection Checkbox -->
                <label class="flex items-center mt-1">
                  <input
                    type="checkbox"
                    checked={selectedOffers.has(offer.txid)}
                    onchange={() => toggleOfferSelection(offer.txid)}
                    class="w-4 h-4 text-verusidx-turquoise-deep bg-white border-verusidx-mountain-mist rounded focus:ring-verusidx-turquoise-deep dark:bg-verusidx-stone-dark dark:border-verusidx-stone-medium"
                  />
                </label>

                <!-- Offer Content -->
                <div class="flex-1">
                  {#snippet offerData()}
                    {@const offering = extractOffering(offer)}
                    {@const accepting = extractAccepting(offer)}
                    
                    <div class="flex items-center space-x-3 mb-3">
                      <!-- Offer Type Badge -->
                      <span class="px-2 py-1 text-xs font-medium rounded bg-verusidx-mountain-blue text-white">
                        {getOfferType(offer)}
                      </span>
                      
                      <!-- Status Badge -->
                      <span class="px-2 py-1 text-xs rounded-full {isOfferExpired(offer) ? 'bg-red-100 text-red-700 dark:bg-red-900/20 dark:text-red-300' : 'bg-green-100 text-green-700 dark:bg-green-900/20 dark:text-green-300'}">
                        {isOfferExpired(offer) ? 'Expired' : 'Active'}
                      </span>
                      
                      <!-- Transaction ID with copy button -->
                      <div class="flex items-center space-x-1">
                        <span class="text-xs font-mono text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
                          {offer.txid?.substring(0, 8)}...
                        </span>
                        <button
                          onclick={() => navigator.clipboard?.writeText(offer.txid)}
                          class="text-xs text-verusidx-turquoise-deep hover:text-verusidx-turquoise-bright p-1"
                          title="Copy full transaction ID"
                        >
                          📋
                        </button>
                      </div>
                    </div>
                    
                    <!-- Main Offer Display -->
                    <div class="mb-4">
                      <div class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">
                        {formatOfferDisplay(offering, accepting)}
                      </div>
                    </div>
                    
                    <!-- Detailed Information -->
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
                      <!-- Offering Side -->
                      <div class="space-y-1">
                        <h4 class="font-medium text-verusidx-turquoise-deep">Offering:</h4>
                        <p><span class="font-medium">Type:</span> {offering.type === 'identity' ? 'Identity' : offering.type === 'native' ? 'Native Currency' : 'Currency'}</p>
                        <p><span class="font-medium">Name:</span> {offering.name}</p>
                        {#if offering.identityAddress}
                          <p><span class="font-medium">i-address:</span> {offering.identityAddress}</p>
                        {/if}
                        {#if offering.amount !== undefined}
                          <p><span class="font-medium">Amount:</span> {formatCryptoAmount(offering.amount)}</p>
                        {/if}
                      </div>
                      
                      <!-- Accepting Side -->
                      <div class="space-y-1">
                        <h4 class="font-medium text-verusidx-turquoise-deep">Accepting:</h4>
                        <p><span class="font-medium">Type:</span> {accepting.type === 'identity' ? 'Identity' : accepting.type === 'native' ? 'Native Currency' : 'Currency'}</p>
                        <p><span class="font-medium">Name:</span> {accepting.name}</p>
                        {#if accepting.identityAddress}
                          <p><span class="font-medium">i-address:</span> {accepting.identityAddress}</p>
                        {/if}
                        {#if accepting.amount !== undefined}
                          <p><span class="font-medium">Amount:</span> {formatCryptoAmount(accepting.amount)}</p>
                        {/if}
                      </div>
                    </div>
                    
                    <div class="mt-3 text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
                      <p><span class="font-medium">Expires:</span> {formatTimeRemaining(offer.expires)}</p>
                      <p><span class="font-medium">Full Transaction ID:</span> <span class="font-mono break-all">{offer.txid}</span></p>
                    </div>
                  {/snippet}
                  
                  {@render offerData()}
                </div>
                
                <!-- Action Buttons -->
                <div class="flex flex-col space-y-2">
                  <button
                    onclick={() => closeOffer(offer.txid)}
                    disabled={isClosingOffer === offer.txid}
                    class="px-3 py-1 text-sm bg-red-100 hover:bg-red-200 text-red-700 dark:bg-red-900/20 dark:hover:bg-red-900/40 dark:text-red-300 rounded transition-colors disabled:opacity-50"
                  >
                    {isClosingOffer === offer.txid ? 'Closing...' : 'Close'}
                  </button>
                </div>
              </div>
            </div>
          {/each}
        </div>

        <!-- No offers after filtering -->
        {#if filteredAndSortedOffers.length === 0}
          <div class="text-center py-8">
            <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
              No offers match the current filter criteria.
            </p>
          </div>
        {/if}

        <!-- Pagination -->
        {#if totalUserOffersPages > 1}
          <div class="flex justify-center items-center space-x-4 mt-6">
            <button
              onclick={() => userOffersPage = Math.max(1, userOffersPage - 1)}
              disabled={userOffersPage === 1}
              class="px-3 py-2 text-sm bg-verusidx-mountain-mist hover:bg-verusidx-mountain-grey text-verusidx-stone-dark dark:bg-verusidx-stone-medium dark:hover:bg-verusidx-stone-light dark:text-white rounded transition-colors disabled:opacity-50"
            >
              Previous
            </button>

            <span class="text-sm text-verusidx-stone-dark dark:text-white">
              Page {userOffersPage} of {totalUserOffersPages}
            </span>

            <button
              onclick={() => userOffersPage = Math.min(totalUserOffersPages, userOffersPage + 1)}
              disabled={userOffersPage === totalUserOffersPages}
              class="px-3 py-2 text-sm bg-verusidx-mountain-mist hover:bg-verusidx-mountain-grey text-verusidx-stone-dark dark:bg-verusidx-stone-medium dark:hover:bg-verusidx-stone-light dark:text-white rounded transition-colors disabled:opacity-50"
            >
              Next
            </button>
          </div>
        {/if}
      {/if}
    </div>

    <!-- Marketplace Search (Secondary) -->
    <div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl p-6">
      <div class="flex justify-between items-center mb-6">
        <h2 class="text-xl font-bold text-verusidx-stone-dark dark:text-white">Search Marketplace Offers</h2>
        <div class="flex items-center space-x-4">
          {#if hasSearched}
            <button
              onclick={clearSearch}
              class="text-sm text-verusidx-mountain-grey hover:text-verusidx-stone-dark dark:hover:text-white transition-colors"
            >
              Clear Search
            </button>
          {/if}
          <button
            onclick={openMakeOfferModal}
            class="px-4 py-2 bg-verusidx-mountain-blue hover:bg-verusidx-lake-blue text-white rounded-lg transition-colors"
          >
            Make New Offer
          </button>
        </div>
      </div>
      
      <!-- Search Form -->
      <form onsubmit={handleSearch} class="space-y-4">
        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
          <!-- Search Type -->
          <div>
            <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
              Search Type
            </label>
            <select 
              bind:value={searchForm.type}
              class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
            >
              <option value="identity">Search by Identity</option>
              <option value="currency">Search by Currency</option>
            </select>
          </div>

          <!-- Search Query -->
          <div>
            <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
              {searchForm.type === 'identity' ? 'Identity Name' : 'Currency Name'}
            </label>
            <input 
              type="text" 
              bind:value={searchForm.query}
              placeholder={searchForm.type === 'identity' ? 'e.g. myname@' : 'e.g. VRSC, DAI.vETH'}
              required
              class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
            />
          </div>

          <!-- Search Button -->
          <div class="flex items-end">
            <button
              type="submit"
              disabled={isSearching}
              class="w-full px-6 py-3 bg-verusidx-turquoise-deep hover:bg-verusidx-turquoise-bright text-white rounded-lg transition-colors disabled:opacity-50"
            >
              {isSearching ? 'Searching...' : 'Search'}
            </button>
          </div>
        </div>

      </form>

      <!-- Search Results -->
      {#if searchResults.length > 0}
        <div class="mt-6">
          <div class="flex justify-between items-center mb-4">
            <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">
              Search Results ({filteredAndSortedSearchResults.length} found)
            </h3>
            
            <!-- Results per page -->
            <div class="flex items-center space-x-2">
              <label class="text-sm text-verusidx-stone-dark dark:text-white">Show:</label>
              <select 
                bind:value={searchResultsPerPage}
                class="px-2 py-1 text-sm border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
              >
                <option value={5}>5</option>
                <option value={10}>10</option>
                <option value={20}>20</option>
                <option value={50}>50</option>
              </select>
              <span class="text-sm text-verusidx-stone-dark dark:text-white">per page</span>
            </div>
          </div>
          
          <!-- Sort and Filter Controls -->
          <div class="flex flex-wrap items-center gap-4 mb-6 p-4 bg-verusidx-snow-ice dark:bg-verusidx-stone-medium/20 rounded-lg">
            <!-- Sort Buttons -->
            <div class="flex items-center space-x-3">
              <span class="text-sm font-medium text-verusidx-stone-dark dark:text-white">Sort by:</span>
              <button
                onclick={() => toggleSort('expiry')}
                class="px-3 py-1 text-sm rounded transition-colors {sortField === 'expiry' ? 'bg-verusidx-turquoise-deep text-white' : 'bg-verusidx-mountain-mist hover:bg-verusidx-mountain-grey text-verusidx-stone-dark dark:bg-verusidx-stone-medium dark:hover:bg-verusidx-stone-light dark:text-white'}"
              >
                Expiry {sortField === 'expiry' ? (sortDirection === 'asc' ? '↑' : '↓') : ''}
              </button>
              <button
                onclick={() => toggleSort('price')}
                class="px-3 py-1 text-sm rounded transition-colors {sortField === 'price' ? 'bg-verusidx-turquoise-deep text-white' : 'bg-verusidx-mountain-mist hover:bg-verusidx-mountain-grey text-verusidx-stone-dark dark:bg-verusidx-stone-medium dark:hover:bg-verusidx-stone-light dark:text-white'}"
              >
                Price {sortField === 'price' ? (sortDirection === 'asc' ? '↑' : '↓') : ''}
              </button>
            </div>
            
            <!-- Name Filter -->
            <div class="flex items-center space-x-2 ml-auto">
              <label class="text-sm font-medium text-verusidx-stone-dark dark:text-white">Filter by name:</label>
              <input
                type="text"
                bind:value={nameFilter}
                oninput={handleNameFilterChange}
                placeholder="Enter name to filter..."
                class="px-3 py-1 text-sm border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white placeholder-verusidx-mountain-grey dark:placeholder-verusidx-mountain-mist"
              />
              {#if nameFilter || sortField}
                <button
                  onclick={() => { nameFilter = ''; sortField = null; searchResultsPage = 1; }}
                  class="px-2 py-1 text-xs text-verusidx-mountain-grey hover:text-verusidx-stone-dark dark:hover:text-white"
                  title="Clear filters"
                >
                  Clear
                </button>
              {/if}
            </div>
          </div>

          <div class="flex flex-col gap-6 max-w-4xl mx-auto">
            {#each paginatedSearchResults as offer (offer.offer?.txid)}
              <OfferCard
                offerResult={offer}
                onViewDetails={handleViewOfferDetails}
                onTakeOffer={openTakeOfferModal}
              />
            {/each}
          </div>

          <!-- Pagination -->
          {#if totalSearchPages > 1}
            <div class="flex justify-center items-center space-x-4 mt-6">
              <button
                onclick={() => searchResultsPage = Math.max(1, searchResultsPage - 1)}
                disabled={searchResultsPage === 1}
                class="px-3 py-2 text-sm bg-verusidx-mountain-mist hover:bg-verusidx-mountain-grey text-verusidx-stone-dark dark:bg-verusidx-stone-medium dark:hover:bg-verusidx-stone-light dark:text-white rounded transition-colors disabled:opacity-50"
              >
                Previous
              </button>

              <span class="text-sm text-verusidx-stone-dark dark:text-white">
                Page {searchResultsPage} of {totalSearchPages}
              </span>

              <button
                onclick={() => searchResultsPage = Math.min(totalSearchPages, searchResultsPage + 1)}
                disabled={searchResultsPage === totalSearchPages}
                class="px-3 py-2 text-sm bg-verusidx-mountain-mist hover:bg-verusidx-mountain-grey text-verusidx-stone-dark dark:bg-verusidx-stone-medium dark:hover:bg-verusidx-stone-light dark:text-white rounded transition-colors disabled:opacity-50"
              >
                Next
              </button>
            </div>
          {/if}
        </div>
      {/if}
    </div>

  </div>
</div>

<!-- Marketplace Offer Modal -->
<MarketplaceOfferModal 
  isOpen={isOfferModalOpen}
  mode={offerModalMode}
  existingOffer={selectedOffer}
  onClose={closeOfferModal}
  onSuccess={handleOfferSuccess}
/>

<!-- Offer Details Modal -->
<OfferDetailsModal
  isOpen={isOfferDetailsModalOpen}
  offerResult={selectedOfferForDetails}
  onClose={closeOfferDetailsModal}
  onTakeOffer={openTakeOfferModal}
  onOpenIdentityModal={openIdentityModal}
  onOpenCurrencyModal={openCurrencyModal}
/>

<!-- Identity Modal for drill-down (moved from OfferDetailsModal) -->
<IdentityModal 
  isOpen={showIdentityModal}
  identityAddress={selectedIdentityAddress}
  onclose={closeIdentityModal}
/>

<!-- Currency Modal for drill-down (moved from OfferDetailsModal) -->
<CurrencyModal 
  isOpen={showCurrencyModal}
  currencyAddress={selectedCurrencyAddress}
  onclose={closeCurrencyModal}
/>