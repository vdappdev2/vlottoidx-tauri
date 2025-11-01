<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Modal } from './cards';
  import { connectionStore, getChainParam } from '$lib/stores/connection';
  import { showSuccess, showError } from '$lib/services/notifications';

  // Props
  interface Props {
    isOpen: boolean;
    onClose: () => void;
    onSuccess?: () => void;
  }

  let { isOpen = false, onClose, onSuccess }: Props = $props();

  // Form state
  let formData = $state({
    nameOrId: '',
    tokenRevoke: false,
    feeOffer: '',
    sourceOfFunds: ''
  });

  // Available addresses
  let sourceAddresses = $state<string[]>([]);
  let isLoadingAddresses = $state(false);
  let hasLoadedAddresses = $state(false);
  let lastLoadedModalOpen = $state<boolean | null>(null);
  let isSubmitting = $state(false);
  let error = $state<string | null>(null);
  let addressLoadingError = $state<string | null>(null);
  let successTxId = $state<string | null>(null);
  
  // Connection state for chain parameter
  let connectionState = $state<any>();
  connectionStore.subscribe(value => { connectionState = value; });

  // Reset when modal closes
  $effect(() => {
    if (!isOpen && lastLoadedModalOpen !== false) {
      console.log('🔄 Modal closed - resetting form');
      resetForm();
      lastLoadedModalOpen = false;
    }
  });

  // Load addresses when modal opens
  $effect(() => {
    if (isOpen && lastLoadedModalOpen !== true) {
      console.log('🚀 Modal opened - loading addresses');
      loadAddresses();
      lastLoadedModalOpen = true;
    }
  });

  function resetForm() {
    formData = {
      nameOrId: '',
      tokenRevoke: false,
      feeOffer: '',
      sourceOfFunds: ''
    };
    error = null;
    addressLoadingError = null;
    successTxId = null;
    hasLoadedAddresses = false;
    sourceAddresses = [];
  }

  async function loadAddresses() {
    if (isLoadingAddresses || hasLoadedAddresses) return;
    
    console.log('🚀 Starting loadAddresses');
    isLoadingAddresses = true;
    addressLoadingError = null;
    
    try {
      const allAddresses = [];
      
      // Load transparent addresses
      try {
        console.log('📞 Calling list_address_groupings...');
        const chainParam = getChainParam(connectionState?.selectedChain);
        console.log('[RevokeIdentityModal]: Loading transparent addresses for chain:', connectionState?.selectedChain, 'param:', chainParam);
        const transparentResult = await invoke('list_address_groupings', { chain: chainParam });
        
        if (Array.isArray(transparentResult)) {
          for (const group of transparentResult) {
            if (Array.isArray(group)) {
              for (const addressInfo of group) {
                if (addressInfo && typeof addressInfo === 'object' && addressInfo.address) {
                  allAddresses.push(addressInfo.address);
                }
              }
            }
          }
        }
      } catch (err) {
        console.error('Failed to load transparent addresses:', err);
      }

      // Load private addresses
      try {
        console.log('📞 Calling z_list_addresses...');
        const chainParam = getChainParam(connectionState?.selectedChain);
        console.log('[RevokeIdentityModal]: Loading private addresses for chain:', connectionState?.selectedChain, 'param:', chainParam);
        const privateResult = await invoke('z_list_addresses', { chain: chainParam });
        
        if (Array.isArray(privateResult)) {
          allAddresses.push(...privateResult);
        }
      } catch (err) {
        console.error('Failed to load private addresses:', err);
      }

      console.log('✅ Loaded addresses:', allAddresses);
      sourceAddresses = allAddresses;
      hasLoadedAddresses = true;
      
    } catch (err) {
      console.error('Error loading addresses:', err);
      addressLoadingError = 'Failed to load addresses. Please try again.';
    } finally {
      isLoadingAddresses = false;
    }
  }

  async function handleSubmit() {
    error = null;
    isSubmitting = true;

    try {
      console.log('🔐 Revoking identity:', formData);
      
      // Build parameters
      const params: any = {
        nameOrId: formData.nameOrId,
        returnTx: false,
        tokenRevoke: formData.tokenRevoke
      };

      // Add optional parameters
      if (formData.feeOffer) {
        params.feeOffer = parseFloat(formData.feeOffer);
      }
      if (formData.sourceOfFunds) {
        params.sourceOfFunds = formData.sourceOfFunds;
      }

      const chainParam = getChainParam(connectionState?.selectedChain);
      console.log('[RevokeIdentityModal]: Revoking identity for chain:', connectionState?.selectedChain, 'param:', chainParam);
      const result = await invoke('revoke_identity', { ...params, chain: chainParam });
      console.log('✅ Identity revoked:', result);
      
      // Extract transaction ID from result
      if (typeof result === 'string') {
        successTxId = result;
      } else if (result && typeof result === 'object' && 'txid' in result) {
        successTxId = result.txid;
      }

      // Show success toast
      showSuccess('Success: identity revoked', { txid: successTxId });

      // Call success callback if provided
      if (onSuccess) {
        onSuccess();
      }

      // Show success for a moment before closing
      setTimeout(() => {
        onClose();
      }, 2000);

    } catch (err) {
      console.error('Failed to revoke identity:', err);
      error = typeof err === 'string' ? err : 'Failed to revoke identity';
      showError(error);
    } finally {
      isSubmitting = false;
    }
  }

  // Form validation
  let isFormValid = $derived(
    formData.nameOrId.trim() !== '' &&
    !isSubmitting
  );
</script>

<Modal 
  isOpen={isOpen}
  onclose={onClose}
  title="Revoke Identity"
>
  <form onsubmit={(e) => { e.preventDefault(); if (isFormValid) handleSubmit(); }}>
    <div class="space-y-4">
      {#if error}
        <div class="p-3 bg-red-100 dark:bg-red-900/20 border border-red-300 dark:border-red-700 rounded-lg">
          <p class="text-sm text-red-700 dark:text-red-300">{error}</p>
        </div>
      {/if}

      {#if successTxId}
        <div class="p-3 bg-green-100 dark:bg-green-900/20 border border-green-300 dark:border-green-700 rounded-lg">
          <p class="text-sm text-green-700 dark:text-green-300">
            Identity revoked successfully!
            {#if successTxId !== 'true'}
              <br>Transaction ID: {successTxId}
            {/if}
          </p>
        </div>
      {/if}

      <!-- Identity Name/ID -->
      <div>
        <label for="nameOrId" class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-1">
          Identity Name or ID <span class="text-red-500">*</span>
        </label>
        <input
          id="nameOrId"
          type="text"
          bind:value={formData.nameOrId}
          placeholder="e.g., myidentity@ or iAddress"
          class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
          disabled={isSubmitting}
        />
        <p class="mt-1 text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
          Enter the identity name (with @) or identity address to revoke
        </p>
      </div>

      <!-- Token Revoke -->
      <div>
        <label class="flex items-center space-x-2 cursor-pointer">
          <input
            type="checkbox"
            bind:checked={formData.tokenRevoke}
            class="w-4 h-4 text-verusidx-turquoise-deep rounded focus:ring-verusidx-turquoise-light"
            disabled={isSubmitting}
          />
          <span class="text-sm font-medium text-verusidx-stone-dark dark:text-white">
            Use Token Revoke
          </span>
        </label>
        <p class="mt-1 ml-6 text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
          Use the tokenized ID control token to revoke (if one exists)
        </p>
      </div>

      <!-- Fee Offer -->
      <div>
        <label for="feeOffer" class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-1">
          Fee Offer (Optional)
        </label>
        <input
          id="feeOffer"
          type="number"
          step="0.0001"
          bind:value={formData.feeOffer}
          placeholder="e.g., 0.0001"
          class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
          disabled={isSubmitting}
        />
        <p class="mt-1 text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
          Non-standard fee amount for the transaction
        </p>
      </div>

      <!-- Source of Funds -->
      <div>
        <label for="sourceOfFunds" class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-1">
          Source of Funds (Optional)
        </label>
        {#if addressLoadingError}
          <div class="mb-2 p-2 bg-yellow-100 dark:bg-yellow-800/20 border border-yellow-300 dark:border-yellow-700 rounded">
            <p class="text-xs text-yellow-700 dark:text-yellow-300">{addressLoadingError}</p>
            <button
              type="button"
              onclick={() => { addressLoadingError = null; loadAddresses(); }}
              class="mt-1 text-xs text-yellow-700 dark:text-yellow-300 underline hover:no-underline"
            >
              Retry
            </button>
          </div>
        {/if}
        <select
          id="sourceOfFunds"
          bind:value={formData.sourceOfFunds}
          class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
          disabled={isSubmitting || isLoadingAddresses}
        >
          <option value="">
            {isLoadingAddresses ? 'Loading addresses...' : 'Select address...'}
          </option>
          {#each sourceAddresses as address}
            <option value={address}>
              {address}{address.startsWith('z') ? ' (Private)' : ' (Transparent)'}
            </option>
          {/each}
        </select>
        <p class="mt-1 text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
          Address to source funds for fees (preserves privacy)
        </p>
      </div>
    </div>

    <!-- Actions -->
    <div class="mt-6 flex justify-end space-x-3">
      <button
        type="button"
        onclick={onClose}
        class="px-4 py-2 text-verusidx-mountain-grey dark:text-verusidx-mountain-mist hover:text-verusidx-stone-dark dark:hover:text-white transition-colors"
        disabled={isSubmitting}
      >
        Cancel
      </button>
      <button
        type="submit"
        disabled={!isFormValid}
        class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {isSubmitting ? 'Revoking...' : 'Revoke Identity'}
      </button>
    </div>
  </form>
</Modal>