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
    identity: '',
    timelockType: 'absolute', // 'absolute' or 'delay'
    blockValue: '',
    feeOffer: '',
    sourceOfFunds: ''
  });

  // Available addresses for source of funds
  let sourceFundsAddresses = $state<string[]>([]);
  let isLoadingSourceFunds = $state(false);
  let hasLoadedSourceFunds = $state(false);
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
      loadSourceFundsAddresses();
      lastLoadedModalOpen = true;
    }
  });

  function resetForm() {
    formData = {
      identity: '',
      timelockType: 'absolute',
      blockValue: '',
      feeOffer: '',
      sourceOfFunds: ''
    };
    error = null;
    addressLoadingError = null;
    successTxId = null;
    hasLoadedSourceFunds = false;
    sourceFundsAddresses = [];
  }

  async function loadSourceFundsAddresses() {
    if (isLoadingSourceFunds || hasLoadedSourceFunds) return;
    
    console.log('🚀 Starting loadSourceFundsAddresses');
    isLoadingSourceFunds = true;
    
    try {
      const allAddresses = [];
      
      // Load transparent addresses (R and i addresses)
      try {
        console.log('📞 Calling list_address_groupings for source funds...');
        const chainParam = getChainParam(connectionState?.selectedChain);
        console.log('[TimelockIdentityModal]: Loading transparent addresses for source funds for chain:', connectionState?.selectedChain, 'param:', chainParam);
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
        console.error('Failed to load transparent addresses for source funds:', err);
      }

      // Load private addresses (z-addresses)
      try {
        console.log('📞 Calling z_list_addresses for source funds...');
        const chainParam = getChainParam(connectionState?.selectedChain);
        console.log('[TimelockIdentityModal]: Loading private addresses for source funds for chain:', connectionState?.selectedChain, 'param:', chainParam);
        const privateResult = await invoke('z_list_addresses', { chain: chainParam });
        
        if (Array.isArray(privateResult)) {
          allAddresses.push(...privateResult);
        }
      } catch (err) {
        console.error('Failed to load private addresses for source funds:', err);
      }

      console.log('✅ Loaded source funds addresses:', allAddresses);
      sourceFundsAddresses = allAddresses;
      hasLoadedSourceFunds = true;
      
    } catch (err) {
      console.error('Error loading source funds addresses:', err);
      addressLoadingError = 'Failed to load source funds addresses. Please try again.';
    } finally {
      isLoadingSourceFunds = false;
    }
  }

  async function handleSubmit() {
    error = null;
    isSubmitting = true;

    try {
      console.log('🔒 Setting identity timelock:', formData);
      
      // Build timelock parameters object
      const timelockParams: any = {};
      if (formData.timelockType === 'absolute') {
        timelockParams.unlockatblock = parseInt(formData.blockValue);
      } else {
        timelockParams.setunlockdelay = parseInt(formData.blockValue);
      }

      // Build parameters
      const params: any = {
        identity: formData.identity,
        timelockParams: timelockParams,
        returnTx: false
      };

      // Add optional parameters
      if (formData.feeOffer) {
        params.feeOffer = parseFloat(formData.feeOffer);
      }
      if (formData.sourceOfFunds) {
        params.sourceOfFunds = formData.sourceOfFunds;
      }

      const chainParam = getChainParam(connectionState?.selectedChain);
      console.log('[TimelockIdentityModal]: Setting identity timelock for chain:', connectionState?.selectedChain, 'param:', chainParam);
      const result = await invoke('set_identity_timelock', { ...params, chain: chainParam });
      console.log('✅ Identity timelock set:', result);
      
      // Extract transaction ID from result
      if (typeof result === 'string') {
        successTxId = result;
      } else if (result && typeof result === 'object' && 'txid' in result) {
        successTxId = (result as any).txid;
      }

      // Show success toast
      showSuccess('Success: identity timelock set', { txid: successTxId });

      // Call success callback if provided
      if (onSuccess) {
        onSuccess();
      }

      // Show success for a moment before closing
      setTimeout(() => {
        onClose();
      }, 2000);

    } catch (err) {
      console.error('Failed to set identity timelock:', err);
      error = typeof err === 'string' ? err : 'Failed to set identity timelock';
      showError(error);
    } finally {
      isSubmitting = false;
    }
  }

  // Form validation
  let isFormValid = $derived(() => {
    const identityValid = formData.identity.trim() !== '';
    const blockValueValid = formData.blockValue.trim() !== '';
    const blockValueNumeric = !isNaN(parseInt(formData.blockValue));
    // For absolute timelock, allow 0 or higher. For delay, require > 0
    const blockValuePositive = formData.timelockType === 'absolute'
      ? parseInt(formData.blockValue) >= 0
      : parseInt(formData.blockValue) > 0;
    const notSubmitting = !isSubmitting;

    console.log('Form validation:', {
      identityValid,
      blockValueValid,
      blockValueNumeric,
      blockValuePositive,
      notSubmitting,
      identity: formData.identity,
      blockValue: formData.blockValue,
      timelockType: formData.timelockType
    });

    return identityValid && blockValueValid && blockValueNumeric && blockValuePositive && notSubmitting;
  });
</script>

<Modal 
  isOpen={isOpen}
  onclose={onClose}
  title="Timelock Identity"
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
            Identity timelock set successfully!
            {#if successTxId !== 'true'}
              <br>Transaction ID: {successTxId}
            {/if}
          </p>
        </div>
      {/if}

      <!-- Identity Name -->
      <div>
        <label for="identity" class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-1">
          Identity Name <span class="text-red-500">*</span>
        </label>
        <input
          id="identity"
          type="text"
          bind:value={formData.identity}
          placeholder="e.g., myidentity@"
          class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
          disabled={isSubmitting}
        />
        <p class="mt-1 text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
          The identity to apply timelock protection to (include @ suffix)
        </p>
      </div>

      <!-- Timelock Type Selection -->
      <div>
        <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
          Timelock Type <span class="text-red-500">*</span>
        </label>
        <div class="space-y-3">
          <label class="flex items-start space-x-3 cursor-pointer">
            <input
              type="radio"
              name="timelockType"
              bind:group={formData.timelockType}
              value="absolute"
              class="mt-1 w-4 h-4 text-purple-900 border-verusidx-mountain-mist focus:ring-purple-900"
              disabled={isSubmitting}
            />
            <div>
              <span class="text-sm font-medium text-verusidx-stone-dark dark:text-white">
                🔒 Absolute Timelock (unlockatblock)
              </span>
              <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
                Unlock at a specific block height. Countdown starts immediately when the transaction is mined.
              </p>
            </div>
          </label>
          
          <label class="flex items-start space-x-3 cursor-pointer">
            <input
              type="radio"
              name="timelockType"
              bind:group={formData.timelockType}
              value="delay"
              class="mt-1 w-4 h-4 text-purple-900 border-verusidx-mountain-mist focus:ring-purple-900"
              disabled={isSubmitting}
            />
            <div>
              <span class="text-sm font-medium text-verusidx-stone-dark dark:text-white">
                ⏰ Unlock Delay (setunlockdelay)
              </span>
              <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
                Set a delay that only starts counting after an unlock request is made. More flexible for ongoing protection.
              </p>
            </div>
          </label>
        </div>
      </div>

      <!-- Block Value Input -->
      <div>
        <label for="blockValue" class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-1">
          {formData.timelockType === 'absolute' ? 'Unlock at Block Height' : 'Delay After Unlock Request (Blocks)'} <span class="text-red-500">*</span>
        </label>
        <input
          id="blockValue"
          type="number"
          min={formData.timelockType === 'absolute' ? '0' : '1'}
          bind:value={formData.blockValue}
          placeholder={formData.timelockType === 'absolute' ? 'e.g., 2500000' : 'e.g., 1440'}
          class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
          disabled={isSubmitting}
        />
        <p class="mt-1 text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
          {formData.timelockType === 'absolute' 
            ? 'The specific block height when the identity will unlock (see current block height in header above)'
            : 'Number of blocks to wait after unlock request (1440 blocks ≈ 1 day)'
          }
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
              onclick={() => { addressLoadingError = null; loadSourceFundsAddresses(); }}
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
          disabled={isSubmitting || isLoadingSourceFunds}
        >
          <option value="">
            {isLoadingSourceFunds ? 'Loading addresses...' : 'Select address...'}
          </option>
          {#each sourceFundsAddresses as address}
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
        class="px-4 py-2 bg-purple-900 text-white rounded-lg hover:bg-purple-800 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {isSubmitting ? 'Setting Timelock...' : 'Set Timelock'}
      </button>
    </div>
  </form>
</Modal>