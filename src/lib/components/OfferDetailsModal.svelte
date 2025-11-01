<script lang="ts">
  import { onMount } from 'svelte';
  import Modal from './cards/Modal.svelte';
  import { connectionStore, getChainParam } from '$lib/stores/connection';
  import { getIdentityFriendlyName, getCurrencyFriendlyName } from '$lib/stores/nameCache';
  import { 
    extractOfferFromSearchResult, 
    formatCryptoAmount,
    getOfferTypeIcon,
    type ParsedOffer 
  } from '$lib/utils/offerUtils';

  interface Props {
    isOpen: boolean;
    offerResult: any; // Raw offer result from getoffers
    onClose: () => void;
    onTakeOffer?: (offer: any) => void;
    onOpenIdentityModal?: (identityAddress: string) => void;
    onOpenCurrencyModal?: (currencyAddress: string) => void;
  }

  let { isOpen = false, offerResult, onClose, onTakeOffer, onOpenIdentityModal, onOpenCurrencyModal }: Props = $props();

  let parsedOffer = $state<ParsedOffer | null>(null);
  let displayNames = $state({
    offering: '',
    accepting: ''
  });
  let isLoadingNames = $state(true);
  let connectionState = $state<any>(null);

  connectionStore.subscribe(state => {
    connectionState = state;
  });

  // Effect to parse offer data when modal opens
  $effect(() => {
    if (isOpen && offerResult) {
      console.log('OfferDetailsModal received offerResult:', offerResult);
      const newParsedOffer = extractOfferFromSearchResult(offerResult);
      console.log('OfferDetailsModal parsed offer:', newParsedOffer);
      
      // Only update if we have a new offer or if parsedOffer is null
      if (newParsedOffer && (!parsedOffer || parsedOffer.txId !== newParsedOffer.txId)) {
        parsedOffer = newParsedOffer;
        loadFriendlyNames();
      } else if (!newParsedOffer) {
        console.error('Failed to parse offer in details modal:', offerResult);
        isLoadingNames = false;
      }
    } else if (!isOpen) {
      // Reset state when modal closes
      parsedOffer = null;
      displayNames = { offering: '', accepting: '' };
      isLoadingNames = true;
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


  function handleTakeOffer() {
    if (onTakeOffer) {
      onTakeOffer(offerResult);
    }
  }

  function formatBlockExpiry(blockExpiry?: number): string {
    if (!blockExpiry) return 'No expiry';
    return `Block ${blockExpiry.toLocaleString()}`;
  }

  function copyToClipboard(text: string) {
    navigator.clipboard?.writeText(text);
  }

  // Create dynamic title with emoji and offer type
  let modalTitle = $derived(parsedOffer ? `Offer Details ${getOfferTypeIcon(parsedOffer.type)} ${parsedOffer.type}` : "Offer Details");
</script>

<Modal {isOpen} onclose={onClose} title={modalTitle} size="xl" zIndex="z-[10]">
  {#if parsedOffer}
    
    <div class="space-y-3">
      <!-- Offering and Accepting Side-by-Side -->
      <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <!-- Offering Section -->
        <div class="space-y-3">
          <h4 class="font-semibold text-verusidx-turquoise-deep text-lg flex items-center space-x-2">
            <span>{parsedOffer.offering.type === 'identity' ? '👤' : '💰'}</span>
            <span>OFFERING</span>
          </h4>
          
          {#if isLoadingNames}
            <div class="animate-pulse space-y-2">
              <div class="h-4 bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium rounded w-3/4"></div>
              <div class="h-4 bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium rounded w-1/2"></div>
            </div>
          {:else}
            <div class="space-y-2 text-verusidx-stone-dark dark:text-white">
              <div>
                <span class="text-sm font-medium text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Type:</span>
                <span class="ml-2 font-medium">{parsedOffer.offering.type === 'identity' ? 'Identity' : 'Currency'}</span>
              </div>
              
              <div>
                <span class="text-sm font-medium text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Name:</span>
                <span class="ml-2 font-semibold text-lg">{displayNames.offering}</span>
              </div>
              
              {#if parsedOffer.offering.identityAddress}
                <div>
                  <span class="text-sm font-medium text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">i-address:</span>
                  <button
                    onclick={() => onOpenIdentityModal?.(parsedOffer.offering.identityAddress!)}
                    class="ml-2 text-verusidx-turquoise-deep hover:text-verusidx-turquoise-bright underline cursor-pointer font-mono text-sm break-all"
                  >
                    {parsedOffer.offering.identityAddress}
                  </button>
                </div>
              {/if}
              
              {#if parsedOffer.offering.currencyAddress}
                <div>
                  <span class="text-sm font-medium text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">i-address:</span>
                  <button
                    onclick={() => onOpenCurrencyModal?.(parsedOffer.offering.currencyAddress!)}
                    class="ml-2 text-verusidx-turquoise-deep hover:text-verusidx-turquoise-bright underline cursor-pointer font-mono text-sm break-all"
                  >
                    {parsedOffer.offering.currencyAddress}
                  </button>
                </div>
              {/if}
              
              {#if parsedOffer.offering.amount !== undefined}
                <div>
                  <span class="text-sm font-medium text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Amount:</span>
                  <span class="ml-2 font-semibold text-lg">{formatCryptoAmount(parsedOffer.offering.amount)}</span>
                </div>
              {/if}
            </div>
          {/if}
        </div>

        <!-- Accepting Section -->
        <div class="space-y-3">
          <h4 class="font-semibold text-verusidx-turquoise-deep text-lg flex items-center space-x-2">
            <span>{parsedOffer.accepting.type === 'identity' ? '👤' : '💰'}</span>
            <span>ACCEPTING</span>
          </h4>
          
          {#if isLoadingNames}
            <div class="animate-pulse space-y-2">
              <div class="h-4 bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium rounded w-3/4"></div>
              <div class="h-4 bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium rounded w-1/2"></div>
            </div>
          {:else}
            <div class="space-y-2 text-verusidx-stone-dark dark:text-white">
              <div>
                <span class="text-sm font-medium text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Type:</span>
                <span class="ml-2 font-medium">{parsedOffer.accepting.type === 'identity' ? 'Identity' : 'Currency'}</span>
              </div>
              
              <div>
                <span class="text-sm font-medium text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Name:</span>
                <span class="ml-2 font-semibold text-lg">{displayNames.accepting}</span>
              </div>
              
              {#if parsedOffer.accepting.identityAddress}
                <div>
                  <span class="text-sm font-medium text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">i-address:</span>
                  <button
                    onclick={() => onOpenIdentityModal?.(parsedOffer.accepting.identityAddress!)}
                    class="ml-2 text-verusidx-turquoise-deep hover:text-verusidx-turquoise-bright underline cursor-pointer font-mono text-sm break-all"
                  >
                    {parsedOffer.accepting.identityAddress}
                  </button>
                </div>
              {/if}
              
              {#if parsedOffer.accepting.currencyAddress}
                <div>
                  <span class="text-sm font-medium text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">i-address:</span>
                  <button
                    onclick={() => onOpenCurrencyModal?.(parsedOffer.accepting.currencyAddress!)}
                    class="ml-2 text-verusidx-turquoise-deep hover:text-verusidx-turquoise-bright underline cursor-pointer font-mono text-sm break-all"
                  >
                    {parsedOffer.accepting.currencyAddress}
                  </button>
                </div>
              {/if}
              
              {#if parsedOffer.accepting.amount !== undefined}
                <div>
                  <span class="text-sm font-medium text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Amount:</span>
                  <span class="ml-2 font-semibold text-lg">{formatCryptoAmount(parsedOffer.accepting.amount)}</span>
                </div>
              {/if}
            </div>
          {/if}
        </div>
      </div>

      <!-- Transaction Details in Two Columns -->
      <div class="grid grid-cols-1 md:grid-cols-2 gap-6 bg-verusidx-snow-ice dark:bg-verusidx-stone-medium/20 rounded-lg p-4">
        <!-- Left Column: Expiry Height & Price (swapped order) -->
        <div class="space-y-3">
          <div>
            <span class="text-sm font-medium text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Expiry Height:</span>
            <span class="ml-2 font-semibold text-verusidx-stone-dark dark:text-white">{parsedOffer.blockExpiry?.toLocaleString() || 'No expiry'}</span>
          </div>
          {#if parsedOffer.price !== undefined && parsedOffer.price > 0}
            <div>
              <span class="text-sm font-medium text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Price:</span>
              <span class="ml-2 font-semibold text-lg text-verusidx-stone-dark dark:text-white">{parsedOffer.price}</span>
            </div>
          {/if}
        </div>

        <!-- Right Column: Status & Transaction ID -->
        <div class="space-y-3">
          <div>
            <span class="text-sm font-medium text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Status:</span>
            <span class="ml-2 font-semibold {parsedOffer.isActive ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}">
              {parsedOffer.isActive ? 'Active ✓' : 'Expired ✗'}
            </span>
          </div>
          {#if parsedOffer.txId}
            <div>
              <span class="text-sm font-medium text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Transaction ID:</span>
              <div class="mt-1 flex items-center space-x-2">
                <div class="overflow-x-auto min-w-0 flex-1">
                  <span class="font-mono text-sm text-verusidx-stone-dark dark:text-white whitespace-nowrap block">{parsedOffer.txId}</span>
                </div>
                <button
                  onclick={() => copyToClipboard(parsedOffer.txId || '')}
                  class="text-verusidx-turquoise-deep hover:text-verusidx-turquoise-bright p-1 flex-shrink-0"
                  title="Copy transaction ID"
                >
                  📋
                </button>
              </div>
            </div>
          {/if}
        </div>
      </div>

      <!-- Action Buttons -->
      <div class="flex justify-between space-x-4">
        <button
          onclick={onClose}
          class="px-6 py-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium text-verusidx-stone-dark dark:text-white rounded-lg hover:bg-verusidx-mountain-mist dark:hover:bg-verusidx-stone-medium transition-colors"
        >
          Close
        </button>
        
        {#if parsedOffer.isActive && onTakeOffer}
          <button
            onclick={handleTakeOffer}
            class="px-6 py-3 bg-verusidx-turquoise-deep hover:bg-verusidx-turquoise-bright text-white rounded-lg transition-colors"
          >
            Take This Offer
          </button>
        {/if}
      </div>
    </div>
  {:else}
    <!-- Error state -->
    <div class="text-center py-8">
      <p class="text-red-600 dark:text-red-400">Failed to parse offer data</p>
    </div>
  {/if}
</Modal>

