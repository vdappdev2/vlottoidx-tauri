<script lang="ts">
  // Props
  interface Props {
    offerType: 'id-for-id' | 'id-for-currency' | 'currency-for-id' | 'currency-for-currency';
    mode: 'make' | 'take';
    formData: {
      fromAddress: string;
      changeAddress?: string;
      expiryBlocks?: string;
      offerCurrency?: string;
      offerAmount?: string;
      offerIdentityName?: string;
      forCurrency?: string;
      forAmount?: string;
      forAddress?: string;
      forIdentity?: any;
      deliverCurrency?: string;
      deliverAmount?: string;
      deliverIdentity?: string;
      acceptCurrency?: string;
      acceptAmount?: string;
      acceptAddress?: string;
    };
    existingOffer?: any;
  }

  let { offerType, mode, formData, existingOffer }: Props = $props();

  // Computed preview data
  let previewData = $derived.by(() => {
    const data = {
      action: mode === 'make' ? 'Creating Offer' : 'Accepting Offer',
      offering: null as any,
      requesting: null as any,
      transaction: {
        fromAddress: formData.fromAddress,
        changeAddress: formData.changeAddress || formData.fromAddress,
        expiryBlocks: formData.expiryBlocks,
        txid: mode === 'take' ? (existingOffer?.offer?.txid || existingOffer?.txid) : undefined
      }
    };

    if (mode === 'make') {
      // Make offer flow
      switch (offerType) {
        case 'id-for-id':
          data.offering = { type: 'identity', data: { name: formData.offerIdentityName } };
          data.requesting = { type: 'identity', data: formData.forIdentity };
          break;
        case 'id-for-currency':
          data.offering = { type: 'identity', data: { name: formData.offerIdentityName } };
          data.requesting = { 
            type: 'currency', 
            data: { 
              currency: formData.forCurrency, 
              amount: formData.forAmount,
              address: formData.forAddress
            } 
          };
          break;
        case 'currency-for-id':
          data.offering = { 
            type: 'currency', 
            data: { 
              currency: formData.offerCurrency, 
              amount: formData.offerAmount 
            } 
          };
          data.requesting = { type: 'identity', data: formData.forIdentity };
          break;
        case 'currency-for-currency':
          data.offering = { 
            type: 'currency', 
            data: { 
              currency: formData.offerCurrency, 
              amount: formData.offerAmount 
            } 
          };
          data.requesting = { 
            type: 'currency', 
            data: { 
              currency: formData.forCurrency, 
              amount: formData.forAmount,
              address: formData.forAddress
            } 
          };
          break;
      }
    } else {
      // Take offer flow
      switch (offerType) {
        case 'id-for-id':
          data.offering = { type: 'identity', data: formData.forIdentity };
          data.requesting = { type: 'identity', data: { name: formData.deliverIdentity } };
          break;
        case 'id-for-currency':
          data.offering = { 
            type: 'currency', 
            data: { 
              currency: formData.deliverCurrency, 
              amount: formData.deliverAmount 
            } 
          };
          data.requesting = { type: 'identity', data: { name: formData.deliverIdentity } };
          break;
        case 'currency-for-id':
          data.offering = { type: 'identity', data: formData.forIdentity };
          data.requesting = { 
            type: 'currency', 
            data: { 
              currency: formData.acceptCurrency, 
              amount: formData.acceptAmount,
              address: formData.acceptAddress
            } 
          };
          break;
        case 'currency-for-currency':
          data.offering = { 
            type: 'currency', 
            data: { 
              currency: formData.deliverCurrency, 
              amount: formData.deliverAmount 
            } 
          };
          data.requesting = { 
            type: 'currency', 
            data: { 
              currency: formData.acceptCurrency, 
              amount: formData.acceptAmount,
              address: formData.acceptAddress
            } 
          };
          break;
      }
    }

    return data;
  });

  function formatIdentity(identityData: any): string {
    if (typeof identityData === 'string') return identityData;
    if (!identityData) return 'Unknown';
    
    // Handle simple name case (just the name@ string)
    if (identityData.name && !identityData.parent && !identityData.primaryAddresses) {
      return identityData.name.endsWith('@') ? identityData.name : identityData.name + '@';
    }
    
    // Handle complex identity case
    let result = identityData.name || 'Unnamed';
    if (identityData.parent) {
      result += `.${identityData.parent}`;
    }
    return result + (result.endsWith('@') ? '' : '@');
  }

  function getOfferTypeIcon(type: string): string {
    switch (type) {
      case 'id-for-id': return '👤↔️👤';
      case 'id-for-currency': return '👤💰';
      case 'currency-for-id': return '💰👤';
      case 'currency-for-currency': return '💰↔️💰';
      default: return '🔄';
    }
  }

  function getOfferTypeDescription(type: string): string {
    switch (type) {
      case 'id-for-id': return 'Identity → Identity';
      case 'id-for-currency': return 'Identity → Currency';
      case 'currency-for-id': return 'Currency → Identity';
      case 'currency-for-currency': return 'Currency → Currency';
      default: return 'Unknown Type';
    }
  }
</script>

<div class="space-y-6">
  <!-- Transaction Header -->
  <div class="text-center">
    <div class="text-4xl mb-2">{getOfferTypeIcon(offerType)}</div>
    <h3 class="text-xl font-bold text-verusidx-stone-dark dark:text-white">
      {previewData.action}
    </h3>
    <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
      {getOfferTypeDescription(offerType)}
    </p>
  </div>

  <!-- Transaction Flow -->
  <div class="grid grid-cols-1 md:grid-cols-3 gap-4 items-center">
    <!-- What You're Offering/Providing -->
    <div class="p-4 bg-verusidx-turquoise-light/20 dark:bg-verusidx-turquoise-deep/20 border border-verusidx-turquoise-light dark:border-verusidx-turquoise-deep rounded-lg">
      <h4 class="font-semibold text-verusidx-stone-dark dark:text-white mb-3 text-center">
        You {mode === 'make' ? 'Offer' : 'Provide'}:
      </h4>
      
      {#if previewData.offering?.type === 'identity'}
        <div class="text-center space-y-2">
          <div class="text-2xl">👤</div>
          <p class="font-medium text-verusidx-stone-dark dark:text-white">
            {formatIdentity(previewData.offering.data)}
          </p>
          {#if previewData.offering.data?.primaryAddresses?.length}
            <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
              {previewData.offering.data.primaryAddresses.filter((a: string) => a.trim()).length} addresses
            </p>
          {/if}
        </div>
      {:else if previewData.offering?.type === 'currency'}
        <div class="text-center space-y-2">
          <div class="text-2xl">💰</div>
          <p class="font-medium text-verusidx-stone-dark dark:text-white">
            {previewData.offering.data.amount} {previewData.offering.data.currency}
          </p>
        </div>
      {/if}
    </div>

    <!-- Arrow -->
    <div class="text-center">
      <div class="text-3xl text-verusidx-turquoise-deep">
        {mode === 'make' ? '⟶' : '⟵'}
      </div>
      <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-2">
        {mode === 'make' ? 'in exchange for' : 'to receive'}
      </p>
    </div>

    <!-- What You're Requesting/Receiving -->
    <div class="p-4 bg-verusidx-forest-light/20 dark:bg-verusidx-forest-deep/20 border border-verusidx-forest-light dark:border-verusidx-forest-deep rounded-lg">
      <h4 class="font-semibold text-verusidx-stone-dark dark:text-white mb-3 text-center">
        You {mode === 'make' ? 'Want' : 'Receive'}:
      </h4>
      
      {#if previewData.requesting?.type === 'identity'}
        <div class="text-center space-y-2">
          <div class="text-2xl">👤</div>
          <p class="font-medium text-verusidx-stone-dark dark:text-white">
            {formatIdentity(previewData.requesting.data)}
          </p>
          {#if previewData.requesting.data?.primaryAddresses?.length}
            <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
              {previewData.requesting.data.primaryAddresses.filter((a: string) => a.trim()).length} addresses
            </p>
          {/if}
        </div>
      {:else if previewData.requesting?.type === 'currency'}
        <div class="text-center space-y-2">
          <div class="text-2xl">💰</div>
          <p class="font-medium text-verusidx-stone-dark dark:text-white">
            {previewData.requesting.data.amount} {previewData.requesting.data.currency}
          </p>
          {#if previewData.requesting.data.address}
            <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist font-mono">
              → {previewData.requesting.data.address.substring(0, 12)}...
            </p>
          {/if}
        </div>
      {/if}
    </div>
  </div>

  <!-- Transaction Details -->
  <div class="p-4 bg-verusidx-snow-ice dark:bg-verusidx-stone-medium rounded-lg">
    <h4 class="font-semibold text-verusidx-stone-dark dark:text-white mb-3">Transaction Details</h4>
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
      <div class="space-y-2">
        <div>
          <span class="font-medium text-verusidx-stone-dark dark:text-white">From Address:</span>
          <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist font-mono text-xs">
            {previewData.transaction.fromAddress}
          </p>
        </div>
        
        {#if previewData.transaction.changeAddress !== previewData.transaction.fromAddress}
          <div>
            <span class="font-medium text-verusidx-stone-dark dark:text-white">Change Address:</span>
            <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist font-mono text-xs">
              {previewData.transaction.changeAddress}
            </p>
          </div>
        {/if}
      </div>

      <div class="space-y-2">
        {#if previewData.transaction.expiryBlocks && mode === 'make'}
          <div>
            <span class="font-medium text-verusidx-stone-dark dark:text-white">Expires at Block:</span>
            <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
              {previewData.transaction.expiryBlocks}
            </p>
          </div>
        {/if}
        
        {#if previewData.transaction.txid && mode === 'take'}
          <div>
            <span class="font-medium text-verusidx-stone-dark dark:text-white">Accepting Offer:</span>
            <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist font-mono text-xs">
              {previewData.transaction.txid.substring(0, 16)}...
            </p>
          </div>
        {/if}
      </div>
    </div>
  </div>

  <!-- Risk Warning for Take Offers -->
  {#if mode === 'take'}
    <div class="p-4 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg">
      <div class="flex items-start space-x-2">
        <div class="text-yellow-600 dark:text-yellow-400 text-lg">⚠️</div>
        <div>
          <h4 class="font-medium text-yellow-800 dark:text-yellow-200 mb-1">Important Notice</h4>
          <p class="text-sm text-yellow-700 dark:text-yellow-300">
            Taking this offer will immediately transfer your assets. Ensure you have sufficient balance and verify all details carefully.
            This transaction cannot be reversed once confirmed.
          </p>
        </div>
      </div>
    </div>
  {/if}

  <!-- Success Expectation -->
  <div class="p-4 bg-verusidx-lake-deep/10 dark:bg-verusidx-turquoise-deep/20 border border-verusidx-turquoise-light dark:border-verusidx-turquoise-deep rounded-lg">
    <h4 class="font-medium text-verusidx-stone-dark dark:text-white mb-2">✅ When This Transaction Succeeds:</h4>
    <ul class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist space-y-1">
      {#if mode === 'make'}
        <li>• Your offer will be published to the marketplace</li>
        <li>• Other users can discover and accept your offer</li>
        <li>• Funds/assets will be held in escrow until someone takes the offer</li>
        <li>• You can close the offer at any time if not yet accepted</li>
      {:else}
        <li>• You will receive the offered {previewData.requesting?.type === 'identity' ? 'identity' : 'currency'}</li>
        <li>• Your {previewData.offering?.type === 'identity' ? 'identity' : 'currency'} will be transferred to the offerer</li>
        <li>• The exchange will complete atomically (both sides transfer or neither does)</li>
        <li>• The original offer will be removed from the marketplace</li>
      {/if}
    </ul>
  </div>
</div>