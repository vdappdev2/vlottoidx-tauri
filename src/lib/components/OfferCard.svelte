<script lang="ts">
  import { connectionStore, getChainParam } from '$lib/stores/connection';
  import { getIdentityFriendlyName, getCurrencyFriendlyName } from '$lib/stores/nameCache';
  import {
    extractOfferFromSearchResult,
    formatOfferSummary,
    formatCryptoAmount,
    getOfferTypeIcon,
    type ParsedOffer
  } from '$lib/utils/offerUtils';

  interface Props {
    offerResult: any; // Raw offer result from getoffers
    onViewDetails: (offer: ParsedOffer) => void;
    onTakeOffer: (offer: any) => void;
  }

  let { offerResult, onViewDetails, onTakeOffer }: Props = $props();

  // Use $derived to automatically recalculate when offerResult prop changes
  let parsedOffer = $derived(extractOfferFromSearchResult(offerResult));

  let displayNames = $state({
    offering: '',
    accepting: ''
  });
  let isLoadingNames = $state(true);
  let connectionState = $state<any>(null);

  connectionStore.subscribe(state => {
    connectionState = state;
  });

  // Use $effect to trigger side effects when parsedOffer changes
  $effect(() => {
    console.log('OfferCard received offerResult:', offerResult);
    console.log('OfferCard parsed offer:', parsedOffer);

    if (parsedOffer) {
      loadFriendlyNames();
    } else {
      console.error('Failed to parse offer:', offerResult);
      isLoadingNames = false;
    }
  });

  async function loadFriendlyNames() {
    if (!parsedOffer) return;

    const chainParam = getChainParam(connectionState?.selectedChain);
    
    try {
      // Load offering name
      if (parsedOffer.offering.type === 'identity' && parsedOffer.offering.identityAddress) {
        displayNames.offering = await getIdentityFriendlyName(parsedOffer.offering.identityAddress, chainParam);
      } else if (parsedOffer.offering.type === 'currency' && parsedOffer.offering.currencyAddress) {
        displayNames.offering = await getCurrencyFriendlyName(parsedOffer.offering.currencyAddress, chainParam);
      } else {
        displayNames.offering = parsedOffer.offering.name;
      }

      // Load accepting name
      if (parsedOffer.accepting.type === 'identity' && parsedOffer.accepting.identityAddress) {
        displayNames.accepting = await getIdentityFriendlyName(parsedOffer.accepting.identityAddress, chainParam);
      } else if (parsedOffer.accepting.type === 'currency' && parsedOffer.accepting.currencyAddress) {
        displayNames.accepting = await getCurrencyFriendlyName(parsedOffer.accepting.currencyAddress, chainParam);
      } else {
        displayNames.accepting = parsedOffer.accepting.name;
      }
    } catch (err) {
      console.warn('Failed to load friendly names:', err);
      // Fallback to original names
      displayNames.offering = parsedOffer.offering.name;
      displayNames.accepting = parsedOffer.accepting.name;
    } finally {
      isLoadingNames = false;
    }
  }

  function handleViewDetails() {
    console.log('OfferCard handleViewDetails called');
    console.log('parsedOffer:', parsedOffer);
    console.log('offerResult:', offerResult);
    
    if (parsedOffer) {
      onViewDetails(parsedOffer);
    } else {
      console.error('No parsed offer available');
    }
  }

  function handleTakeOffer() {
    onTakeOffer(offerResult);
  }

  function formatBlockExpiry(blockExpiry?: number): string {
    if (!blockExpiry) return 'No expiry';
    return `Block ${blockExpiry.toLocaleString()}`;
  }

  // Create derived display names instead of updating parsedOffer
  let displaySummary = $derived(() => {
    if (!parsedOffer || isLoadingNames) return null;
    
    const offeringWithName = { 
      ...parsedOffer.offering, 
      name: displayNames.offering || parsedOffer.offering.name 
    };
    const acceptingWithName = { 
      ...parsedOffer.accepting, 
      name: displayNames.accepting || parsedOffer.accepting.name 
    };
    
    return formatOfferSummary(offeringWithName, acceptingWithName);
  });
</script>

{#if parsedOffer}
  <div class="border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg p-4 bg-white dark:bg-verusidx-stone-dark hover:shadow-lg transition-shadow">
    <!-- Header with type and status -->
    <div class="flex items-center justify-between mb-3">
      <div class="flex items-center space-x-2">
        <span class="text-lg">{getOfferTypeIcon(parsedOffer.type)}</span>
        <span class="px-2 py-1 text-sm font-medium rounded bg-verusidx-mountain-blue text-white">
          {parsedOffer.type}
        </span>
      </div>
      
      <!-- Status Badge -->
      <span class="px-2 py-1 text-sm rounded-full {parsedOffer.isActive ? 'bg-green-100 text-green-700 dark:bg-green-900/20 dark:text-green-300' : 'bg-red-100 text-red-700 dark:bg-red-900/20 dark:text-red-300'}">
        {parsedOffer.isActive ? 'Active' : 'Expired'} ✓
      </span>
    </div>

    <!-- Main Content with Buttons on Right -->
    <div class="grid grid-cols-[1fr,auto] gap-6 mb-4">
      <!-- Left Content Area -->
      <div class="space-y-4">
        <!-- Offer Exchange Section -->
        <div class="grid grid-cols-2 gap-4">
          {#if isLoadingNames}
            <!-- Loading state -->
            <div class="space-y-1">
              <span class="text-sm font-semibold text-verusidx-turquoise-deep dark:text-verusidx-turquoise-bright">OFFERING</span>
              <div class="h-6 bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium rounded animate-pulse"></div>
              <div class="h-4 bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium rounded animate-pulse mt-1"></div>
            </div>
            
            <div class="flex items-center justify-center">
              <span class="text-2xl text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">→</span>
            </div>
            
            <div class="space-y-1">
              <span class="text-sm font-semibold text-verusidx-turquoise-deep dark:text-verusidx-turquoise-bright">ACCEPTING</span>
              <div class="h-6 bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium rounded animate-pulse"></div>
              <div class="h-4 bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium rounded animate-pulse mt-1"></div>
            </div>
          {:else}
            <!-- Offering Section -->
            <div class="space-y-1">
              <span class="text-sm font-semibold text-verusidx-turquoise-deep dark:text-verusidx-turquoise-bright flex items-center space-x-1">
                <span>{parsedOffer.offering.type === 'identity' ? '👤' : '💰'}</span>
                <span>OFFERING</span>
              </span>
              <!-- Main display with friendly name and amount -->
              <div class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">
                {#if parsedOffer.offering.amount !== undefined}
                  {formatCryptoAmount(parsedOffer.offering.amount)} {displayNames.offering || parsedOffer.offering.name}
                {:else}
                  {displayNames.offering || parsedOffer.offering.name}
                {/if}
              </div>
              <!-- i-address as secondary info -->
              {#if parsedOffer.offering.currencyAddress || parsedOffer.offering.identityAddress}
                <div class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist font-mono break-all">
                  {parsedOffer.offering.currencyAddress || parsedOffer.offering.identityAddress}
                </div>
              {/if}
            </div>
            
            <!-- Accepting Section -->
            <div class="space-y-1">
              <span class="text-sm font-semibold text-verusidx-turquoise-deep dark:text-verusidx-turquoise-bright flex items-center space-x-1">
                <span>{parsedOffer.accepting.type === 'identity' ? '👤' : '💰'}</span>
                <span>ACCEPTING</span>
              </span>
              <!-- Main display with friendly name and amount -->
              <div class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">
                {#if parsedOffer.accepting.amount !== undefined}
                  {formatCryptoAmount(parsedOffer.accepting.amount)} {displayNames.accepting || parsedOffer.accepting.name}
                {:else}
                  {displayNames.accepting || parsedOffer.accepting.name}
                {/if}
              </div>
              <!-- i-address as secondary info -->
              {#if parsedOffer.accepting.currencyAddress || parsedOffer.accepting.identityAddress}
                <div class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist font-mono break-all">
                  {parsedOffer.accepting.currencyAddress || parsedOffer.accepting.identityAddress}
                </div>
              {/if}
            </div>
          {/if}
        </div>

        <!-- Bottom Info Section - EXPIRES and PRICE -->
        <div class="grid grid-cols-2 gap-4 mt-3">
          <div class="text-sm font-semibold">
            <span class="text-verusidx-turquoise-deep dark:text-verusidx-turquoise-bright">⏳ EXPIRES:</span>
            <span class="text-verusidx-stone-dark dark:text-white ml-1">
              {parsedOffer.blockExpiry?.toLocaleString() || 'No expiry'}
            </span>
          </div>
          <div class="text-sm font-semibold">
            <span class="text-verusidx-turquoise-deep dark:text-verusidx-turquoise-bright">💎 PRICE:</span>
            <span class="text-verusidx-stone-dark dark:text-white ml-1">
              {#if parsedOffer.price !== undefined && parsedOffer.price > 0}
                {formatCryptoAmount(parsedOffer.price)}
              {:else}
                Not specified
              {/if}
            </span>
          </div>
        </div>

      </div>

      <!-- Right Side Action Buttons -->
      <div class="flex flex-col space-y-2 justify-center">
        <button
          onclick={handleViewDetails}
          class="px-3 py-2 text-sm font-medium border border-verusidx-mountain-mist dark:border-verusidx-stone-medium text-verusidx-stone-dark dark:text-white rounded-lg hover:bg-verusidx-mountain-mist dark:hover:bg-verusidx-stone-medium transition-colors whitespace-nowrap"
        >
          View Details
        </button>
        
        {#if parsedOffer.isActive}
          <button
            onclick={handleTakeOffer}
            class="px-3 py-2 text-sm font-medium bg-verusidx-turquoise-deep hover:bg-verusidx-turquoise-bright text-white rounded-lg transition-colors whitespace-nowrap"
          >
            Take Offer
          </button>
        {:else}
          <button
            disabled
            class="px-3 py-2 text-sm font-medium bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-mountain-grey dark:text-verusidx-mountain-mist rounded-lg cursor-not-allowed whitespace-nowrap"
          >
            Expired
          </button>
        {/if}
      </div>
    </div>
  </div>
{:else}
  <!-- Error state -->
  <div class="border border-red-200 dark:border-red-800 rounded-lg p-4 bg-red-50 dark:bg-red-900/20">
    <p class="text-red-700 dark:text-red-300 text-sm">Failed to parse offer data</p>
  </div>
{/if}