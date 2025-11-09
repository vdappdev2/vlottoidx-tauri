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
    name: '',
    parent: '',
    primaryAddresses: [''],
    minimumSignatures: 1,
    revocationAuthority: '',
    recoveryAuthority: '',
    privateAddress: '',
    tokenRecover: false,
    feeOffer: '',
    sourceOfFunds: ''
  });

  // Available addresses - separate for different purposes
  let sourceAddresses = $state<string[]>([]);
  let sourceFundsAddresses = $state<string[]>([]);
  let privateAddresses = $state<string[]>([]);
  let isLoadingAddresses = $state(false);
  let isLoadingSourceFunds = $state(false);
  let isLoadingPrivateAddresses = $state(false);
  let hasLoadedAddresses = $state(false);
  let hasLoadedSourceFunds = $state(false);
  let hasLoadedPrivateAddresses = $state(false);
  let lastLoadedModalOpen = $state<boolean | null>(null);
  let isSubmitting = $state(false);
  let error = $state<string | null>(null);
  let addressLoadingError = $state<string | null>(null);
  let successTxId = $state<string | null>(null);
  let showManualPrimaryInput = $state(false);

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
      loadSourceFundsAddresses();
      loadPrivateAddresses();
      lastLoadedModalOpen = true;
    }
  });

  function resetForm() {
    formData = {
      name: '',
      parent: '',
      primaryAddresses: [''],
      minimumSignatures: 1,
      revocationAuthority: '',
      recoveryAuthority: '',
      privateAddress: '',
      tokenRecover: false,
      feeOffer: '',
      sourceOfFunds: ''
    };
    error = null;
    addressLoadingError = null;
    successTxId = null;
    hasLoadedAddresses = false;
    hasLoadedSourceFunds = false;
    hasLoadedPrivateAddresses = false;
    sourceAddresses = [];
    sourceFundsAddresses = [];
    privateAddresses = [];
    showManualPrimaryInput = false;
  }

  async function loadAddresses() {
    if (isLoadingAddresses || hasLoadedAddresses) return;
    
    console.log('🚀 Starting loadAddresses');
    isLoadingAddresses = true;
    addressLoadingError = null;
    
    try {
      // Get addresses by account (default account "") - only transparent addresses for primary addresses
      try {
        console.log('📞 Calling get_addresses_by_account for primary addresses...');
        const chainParam = getChainParam(connectionState?.selectedChain);
        console.log('[RecoverIdentityModal]: Loading primary addresses for chain:', connectionState?.selectedChain, 'param:', chainParam);
        const addresses = await invoke('get_addresses_by_account', { account: "", chain: chainParam });
        console.log('📝 Addresses by account result:', addresses);
        if (Array.isArray(addresses)) {
          sourceAddresses = addresses;
          console.log('📍 Found addresses from default account:', addresses.length);
        }
      } catch (err) {
        console.error('❌ Failed to load addresses by account:', err);
        addressLoadingError = `Failed to load addresses: ${err}`;
      }
      
      hasLoadedAddresses = true;
      console.log('✅ Primary address loading completed successfully');
      
    } catch (err) {
      console.error('❌ Critical error in loadAddresses:', err);
      addressLoadingError = `Failed to load source addresses: ${err}`;
      hasLoadedAddresses = false;
    } finally {
      isLoadingAddresses = false;
    }
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
        console.log('[RecoverIdentityModal]: Loading transparent addresses for source funds for chain:', connectionState?.selectedChain, 'param:', chainParam);
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
        console.log('[RecoverIdentityModal]: Loading private addresses for source funds for chain:', connectionState?.selectedChain, 'param:', chainParam);
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

  async function loadPrivateAddresses() {
    if (isLoadingPrivateAddresses || hasLoadedPrivateAddresses) return;
    
    console.log('🚀 Starting loadPrivateAddresses');
    isLoadingPrivateAddresses = true;
    
    try {
      // Get private addresses from z_list_addresses
      console.log('📞 Calling z_list_addresses...');
      const chainParam = getChainParam(connectionState?.selectedChain);
      console.log('[RecoverIdentityModal]: Loading private addresses for chain:', connectionState?.selectedChain, 'param:', chainParam);
      const zAddresses = await invoke('z_list_addresses', { chain: chainParam });
      console.log('📝 Private addresses result:', zAddresses);
      
      if (Array.isArray(zAddresses)) {
        privateAddresses = zAddresses;
        hasLoadedPrivateAddresses = true;
        console.log('✅ Private address loading completed successfully');
      } else {
        console.log('⚠️ No private addresses returned');
        privateAddresses = [];
        hasLoadedPrivateAddresses = true;
      }
      
    } catch (err) {
      console.error('❌ Private address loading failed:', err);
      privateAddresses = [];
      hasLoadedPrivateAddresses = true;
    } finally {
      isLoadingPrivateAddresses = false;
    }
  }

  function addPrimaryAddress() {
    formData.primaryAddresses = [...formData.primaryAddresses, ''];
  }

  function removePrimaryAddress(index: number) {
    if (formData.primaryAddresses.length > 1) {
      formData.primaryAddresses = formData.primaryAddresses.filter((_, i) => i !== index);
    }
  }

  function handlePrimaryAddressDropdownChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    if (target.value === '__MANUAL__') {
      showManualPrimaryInput = true;
      formData.primaryAddresses[0] = '';
    } else {
      showManualPrimaryInput = false;
      formData.primaryAddresses[0] = target.value;
    }
  }

  function switchToDropdown() {
    showManualPrimaryInput = false;
    formData.primaryAddresses[0] = '';
  }

  async function handleSubmit() {
    error = null;
    isSubmitting = true;

    try {
      console.log('🔓 Recovering identity:', formData);
      
      // Build JSON identity object
      const jsonIdentity: any = {
        name: formData.name,
        primaryaddresses: formData.primaryAddresses.filter(addr => addr.trim() !== ''),
        minimumsignatures: formData.minimumSignatures
      };

      // Add parent if provided (required for sub-IDs and PBaaS chains)
      if (formData.parent) {
        jsonIdentity.parent = formData.parent;
      }

      // Add optional parameters to identity
      if (formData.revocationAuthority) {
        jsonIdentity.revocationauthority = formData.revocationAuthority;
      }
      if (formData.recoveryAuthority) {
        jsonIdentity.recoveryauthority = formData.recoveryAuthority;
      }
      if (formData.privateAddress) {
        if (formData.privateAddress === "null") {
          jsonIdentity.privateaddress = null;
        } else {
          jsonIdentity.privateaddress = formData.privateAddress;
        }
      }

      // Build parameters
      const params: any = {
        jsonIdentity,
        returnTx: false,
        tokenRecover: formData.tokenRecover
      };

      // Add optional parameters
      if (formData.feeOffer) {
        params.feeOffer = parseFloat(formData.feeOffer);
      }
      if (formData.sourceOfFunds) {
        params.sourceOfFunds = formData.sourceOfFunds;
      }

      const chainParam = getChainParam(connectionState?.selectedChain);
      console.log('[RecoverIdentityModal]: Recovering identity for chain:', connectionState?.selectedChain, 'param:', chainParam);
      const result = await invoke('recover_identity', { ...params, chain: chainParam });
      console.log('✅ Identity recovered:', result);
      
      // Extract transaction ID from result
      if (typeof result === 'string') {
        successTxId = result;
      } else if (result && typeof result === 'object' && 'txid' in result) {
        successTxId = (result as any).txid;
      }

      // Show success toast
      showSuccess('Success: identity recovered', { txid: successTxId });

      // Call success callback if provided
      if (onSuccess) {
        onSuccess();
      }

      // Show success for a moment before closing
      setTimeout(() => {
        onClose();
      }, 2000);

    } catch (err) {
      console.error('Failed to recover identity:', err);
      error = typeof err === 'string' ? err : 'Failed to recover identity';
      showError(error);
    } finally {
      isSubmitting = false;
    }
  }

  // Form validation
  let isFormValid = $derived(
    formData.name.trim() !== '' &&
    formData.primaryAddresses.some(addr => addr.trim() !== '') &&
    formData.minimumSignatures > 0 &&
    formData.minimumSignatures <= formData.primaryAddresses.filter(addr => addr.trim() !== '').length &&
    !isSubmitting
  );
</script>

<Modal 
  isOpen={isOpen}
  onclose={onClose}
  title="Recover Identity"
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
            Identity recovery initiated successfully!
            {#if successTxId !== 'true'}
              <br>Transaction ID: {successTxId}
            {/if}
          </p>
        </div>
      {/if}

      <!-- Identity Name -->
      <div>
        <label for="name" class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-1">
          Identity Name <span class="text-red-500">*</span>
        </label>
        <input
          id="name"
          type="text"
          bind:value={formData.name}
          placeholder="e.g., myidentity"
          class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
          disabled={isSubmitting}
        />
        <p class="mt-1 text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
          The name of the identity to recover (without @ suffix)
        </p>
      </div>

      <!-- Parent Name (for Sub-IDs) -->
      <div>
        <label for="parent" class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-1">
          Parent Name (Optional)
        </label>
        <input
          id="parent"
          type="text"
          bind:value={formData.parent}
          placeholder="e.g., SomeCurrency or currencyname.pbaaschain"
          class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
          disabled={isSubmitting}
        />
        <p class="mt-1 text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
          required for sub-IDs & PBaaS chains, optional for Root IDs (name.vrsc@)
        </p>
      </div>

      <!-- Primary Addresses -->
      <div>
        <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-1">
          Primary Addresses <span class="text-red-500">*</span>
        </label>
        <div class="space-y-2">
          {#each formData.primaryAddresses as _, index}
            <div class="flex space-x-2">
              {#if index === 0}
                <!-- First address: dropdown or manual input -->
                {#if !showManualPrimaryInput}
                  <select
                    onchange={handlePrimaryAddressDropdownChange}
                    value={formData.primaryAddresses[index]}
                    required
                    disabled={isLoadingAddresses || isSubmitting}
                    class="flex-1 p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
                  >
                    <option value="">
                      {#if isLoadingAddresses}
                        Loading addresses...
                      {:else if hasLoadedAddresses && sourceAddresses.length === 0}
                        No addresses available
                      {:else if !hasLoadedAddresses}
                        Failed to load addresses
                      {:else}
                        Select primary address...
                      {/if}
                    </option>
                    <option value="__MANUAL__">Input address manually</option>
                    {#each sourceAddresses as address}
                      <option value={address}>{address}</option>
                    {/each}
                  </select>
                {:else}
                  <div class="flex-1">
                    <input
                      type="text"
                      bind:value={formData.primaryAddresses[index]}
                      required
                      placeholder="Enter recovery address (R-address)"
                      class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
                      disabled={isSubmitting}
                    />
                    <button
                      type="button"
                      onclick={switchToDropdown}
                      class="mt-1 text-xs text-verusidx-turquoise-deep hover:text-verusidx-turquoise-bright"
                    >
                      ← Use address dropdown
                    </button>
                  </div>
                {/if}
              {:else}
                <!-- Additional addresses: manual text input -->
                <input
                  type="text"
                  bind:value={formData.primaryAddresses[index]}
                  placeholder="Enter recovery address"
                  class="flex-1 p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
                  disabled={isSubmitting}
                />
              {/if}
              {#if formData.primaryAddresses.length > 1}
                <button
                  type="button"
                  onclick={() => removePrimaryAddress(index)}
                  class="px-3 py-2 text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
                  disabled={isSubmitting}
                >
                  Remove
                </button>
              {/if}
            </div>
          {/each}
        </div>
        <button
          type="button"
          onclick={addPrimaryAddress}
          class="mt-2 text-sm text-verusidx-turquoise-deep hover:text-verusidx-turquoise-bright"
          disabled={isSubmitting}
        >
          + Add another address
        </button>
      </div>

      <!-- Minimum Signatures -->
      <div>
        <label for="minimumSignatures" class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-1">
          Minimum Signatures <span class="text-red-500">*</span>
        </label>
        <input
          id="minimumSignatures"
          type="number"
          min="1"
          max={formData.primaryAddresses.filter(addr => addr.trim() !== '').length || 1}
          bind:value={formData.minimumSignatures}
          class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
          disabled={isSubmitting}
        />
        <p class="mt-1 text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
          Number of signatures required (max: {formData.primaryAddresses.filter(addr => addr.trim() !== '').length || 1})
        </p>
      </div>

      <!-- Optional Parameters (Collapsible) -->
      <details class="border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg">
        <summary class="p-3 cursor-pointer text-verusidx-stone-dark dark:text-white font-medium">
          Optional Parameters
        </summary>
        <div class="p-3 space-y-4 border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
          <!-- Revocation Authority -->
          <div>
            <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
              Revocation Authority
            </label>
            <input 
              type="text" 
              bind:value={formData.revocationAuthority} 
              placeholder="Address that can revoke this identity"
              class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
              disabled={isSubmitting}
            />
          </div>

          <!-- Recovery Authority -->
          <div>
            <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
              Recovery Authority
            </label>
            <input 
              type="text" 
              bind:value={formData.recoveryAuthority} 
              placeholder="Address that can recover this identity"
              class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
              disabled={isSubmitting}
            />
          </div>

          <!-- Private Address -->
          <div>
            <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
              Private Address
            </label>
            {#if isLoadingPrivateAddresses}
              <select 
                disabled
                class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white opacity-50"
              >
                <option>Loading private addresses...</option>
              </select>
            {:else}
              <select
                bind:value={formData.privateAddress}
                class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
                disabled={isSubmitting}
              >
                <option value="">Select a private address (optional)</option>
                <option value="null">Remove z-address</option>
                {#each privateAddresses as addr}
                  <option value={addr}>{addr} (Private)</option>
                {/each}
              </select>
            {/if}
            <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
              Private Z-address associated with this identity
            </p>
          </div>
        </div>
      </details>

      <!-- Token Recover -->
      <div>
        <label class="flex items-center space-x-2 cursor-pointer">
          <input
            type="checkbox"
            bind:checked={formData.tokenRecover}
            class="w-4 h-4 text-verusidx-turquoise-deep rounded focus:ring-verusidx-turquoise-light"
            disabled={isSubmitting}
          />
          <span class="text-sm font-medium text-verusidx-stone-dark dark:text-white">
            Use Token Recover
          </span>
        </label>
        <p class="mt-1 ml-6 text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
          Use the tokenized ID control token to recover (if one exists)
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
        class="px-4 py-2 bg-verusidx-turquoise-deep text-white rounded-lg hover:bg-verusidx-turquoise-bright transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {isSubmitting ? 'Recovering...' : 'Recover Identity'}
      </button>
    </div>
  </form>
</Modal>