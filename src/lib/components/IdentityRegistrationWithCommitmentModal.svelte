<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Modal } from './cards';
  import { connectionStore, getChainParam } from '$lib/stores/connection';
  import { showSuccess, showError } from '$lib/services/notifications';

  interface Props {
    isOpen: boolean;
    onClose: () => void;
    onSuccess?: () => void;
  }

  let { isOpen = false, onClose, onSuccess }: Props = $props();

  // Form state
  let commitmentJson = $state('');
  let commitmentResult = $state<any>(null);
  let jsonError = $state<string | null>(null);

  let formData = $state({
    primaryAddresses: [''],
    minimumSignatures: 1,
    revocationAuthority: '',
    recoveryAuthority: '',
    privateAddress: '',
    sourceOfFunds: ''
  });

  // Available addresses
  let controlAddresses = $state<string[]>([]);
  let sourceAddresses = $state<string[]>([]);
  let privateAddresses = $state<string[]>([]);
  let isLoadingControlAddresses = $state(false);
  let isLoadingSourceAddresses = $state(false);
  let isLoadingPrivateAddresses = $state(false);
  let hasLoadedControlAddresses = $state(false);
  let hasLoadedSourceAddresses = $state(false);
  let hasLoadedPrivateAddresses = $state(false);
  let lastLoadedModalOpen = $state<boolean | null>(null);
  let isSubmitting = $state(false);
  let isCalculatingFee = $state(false);
  let error = $state<string | null>(null);
  let addressLoadingError = $state<string | null>(null);
  let showManualPrimaryInput = $state(false);

  // Connection state
  let connectionState = $state<any>();
  connectionStore.subscribe(value => { connectionState = value; });

  // Example JSON for placeholder
  const exampleJson = `{
  "txid": "abc123...",
  "namereservation": {
    "version": 1,
    "name": "myidentity",
    "parent": "vrsc",
    "salt": "...",
    "referral": "...",
    "nameid": "i..."
  }
}`;

  // Reset when modal closes
  $effect(() => {
    if (!isOpen && lastLoadedModalOpen !== false) {
      resetForm();
      lastLoadedModalOpen = false;
    }
  });

  // Load addresses when modal opens
  $effect(() => {
    if (isOpen && lastLoadedModalOpen !== true) {
      loadAddresses();
      lastLoadedModalOpen = true;
    }
  });

  function resetForm() {
    commitmentJson = '';
    commitmentResult = null;
    jsonError = null;
    formData = {
      primaryAddresses: [''],
      minimumSignatures: 1,
      revocationAuthority: '',
      recoveryAuthority: '',
      privateAddress: '',
      sourceOfFunds: ''
    };
    error = null;
    addressLoadingError = null;
    isSubmitting = false;
    isCalculatingFee = false;
    hasLoadedControlAddresses = false;
    hasLoadedSourceAddresses = false;
    hasLoadedPrivateAddresses = false;
    lastLoadedModalOpen = null;
    controlAddresses = [];
    sourceAddresses = [];
    privateAddresses = [];
    showManualPrimaryInput = false;
  }

  async function loadAddresses() {
    await Promise.all([
      loadControlAddresses(),
      loadSourceAddresses(),
      loadPrivateAddresses()
    ]);
  }

  async function loadControlAddresses() {
    if (isLoadingControlAddresses || hasLoadedControlAddresses) return;

    isLoadingControlAddresses = true;

    try {
      const chainParam = getChainParam(connectionState?.selectedChain);
      const addresses = await invoke('get_addresses_by_account', { account: '', chain: chainParam });

      if (Array.isArray(addresses) && addresses.length > 0) {
        controlAddresses = addresses;
        hasLoadedControlAddresses = true;
      }
    } catch (err) {
      console.error('Failed to load control addresses:', err);
      addressLoadingError = `Failed to load addresses: ${err}`;
    } finally {
      isLoadingControlAddresses = false;
    }
  }

  async function loadSourceAddresses() {
    if (isLoadingSourceAddresses || hasLoadedSourceAddresses) return;

    isLoadingSourceAddresses = true;

    try {
      const allAddresses = [];
      const chainParam = getChainParam(connectionState?.selectedChain);

      // Get transparent addresses
      try {
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

      // Get private addresses
      try {
        const privateAddresses = await invoke('z_list_addresses', { chain: chainParam });
        if (Array.isArray(privateAddresses)) {
          allAddresses.push(...privateAddresses);
        }
      } catch (err) {
        console.error('Failed to load private source addresses:', err);
      }

      sourceAddresses = allAddresses;
      hasLoadedSourceAddresses = true;
    } catch (err) {
      console.error('Critical error in loadSourceAddresses:', err);
      addressLoadingError = `Failed to load source addresses: ${err}`;
    } finally {
      isLoadingSourceAddresses = false;
    }
  }

  async function loadPrivateAddresses() {
    if (isLoadingPrivateAddresses || hasLoadedPrivateAddresses) return;

    isLoadingPrivateAddresses = true;

    try {
      const chainParam = getChainParam(connectionState?.selectedChain);
      const addresses = await invoke('z_list_addresses', { chain: chainParam });

      if (Array.isArray(addresses)) {
        privateAddresses = addresses;
      }

      hasLoadedPrivateAddresses = true;
    } catch (err) {
      console.error('Private address loading failed:', err);
      privateAddresses = [];
      hasLoadedPrivateAddresses = true;
    } finally {
      isLoadingPrivateAddresses = false;
    }
  }

  function validateAndParseCommitment() {
    jsonError = null;
    commitmentResult = null;

    if (!commitmentJson.trim()) {
      jsonError = 'Please paste your name commitment JSON';
      return;
    }

    try {
      const parsed = JSON.parse(commitmentJson);

      // Validate required fields
      if (!parsed.txid || typeof parsed.txid !== 'string') {
        jsonError = 'Missing or invalid "txid" field';
        return;
      }

      if (!parsed.namereservation || typeof parsed.namereservation !== 'object') {
        jsonError = 'Missing or invalid "namereservation" field';
        return;
      }

      const nr = parsed.namereservation;

      if (!nr.name || typeof nr.name !== 'string') {
        jsonError = 'Missing or invalid "namereservation.name" field';
        return;
      }

      if (!nr.nameid || typeof nr.nameid !== 'string') {
        jsonError = 'Missing or invalid "namereservation.nameid" field';
        return;
      }

      if (nr.version === undefined || typeof nr.version !== 'number') {
        jsonError = 'Missing or invalid "namereservation.version" field';
        return;
      }

      // Success - store the parsed result
      commitmentResult = parsed;

    } catch (err) {
      jsonError = `Invalid JSON format: ${err}`;
    }
  }

  function addPrimaryAddress() {
    if (formData.primaryAddresses.length < 25) {
      formData.primaryAddresses = [...formData.primaryAddresses, ''];
    }
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

  async function handleSubmit(event: SubmitEvent) {
    event.preventDefault();

    if (!commitmentResult) {
      error = 'Please parse your name commitment first';
      return;
    }

    if (!formData.primaryAddresses[0]) {
      error = 'Please provide at least one primary address';
      return;
    }

    isSubmitting = true;
    isCalculatingFee = true;
    error = null;

    try {
      const chainParam = getChainParam(connectionState?.selectedChain);

      // Build identity object
      const identity: any = {
        name: commitmentResult.namereservation.name,
        parent: commitmentResult.namereservation.parent,
        primaryaddresses: formData.primaryAddresses.filter(addr => addr.trim() !== ''),
        minimumsignatures: formData.minimumSignatures
      };

      if (formData.revocationAuthority) {
        identity.revocationauthority = formData.revocationAuthority;
      }

      if (formData.recoveryAuthority) {
        identity.recoveryauthority = formData.recoveryAuthority;
      }

      if (formData.privateAddress) {
        identity.privateaddress = formData.privateAddress;
      }

      // First attempt: Call with null feeOffer to get daemon's expected fee
      try {
        const result = await invoke('register_identity', {
          txid: commitmentResult.txid,
          namereservation: commitmentResult.namereservation,
          identity: identity,
          returnTx: false,
          feeOffer: null,
          sourceOfFunds: formData.sourceOfFunds || null,
          chain: chainParam
        });

        console.log('Identity registration succeeded:', result);
        const registrationTxId = typeof result === 'string' ? result : (result as any).txid || String(result);

        showSuccess('Success: identity created', { txid: registrationTxId });

        if (onSuccess) {
          onSuccess();
        }
        onClose();

      } catch (firstErr: any) {
        // Fee detection: Check if error message contains required fee
        const errorStr = typeof firstErr === 'string' ? firstErr : String(firstErr);
        const feeMatch = errorStr.match(/Fee offer must be at least ([\d.]+)/);

        if (feeMatch && feeMatch[1]) {
          const requiredFee = parseFloat(feeMatch[1]);
          isCalculatingFee = false;

          // Second attempt: Retry with the required fee
          const result = await invoke('register_identity', {
            txid: commitmentResult.txid,
            namereservation: commitmentResult.namereservation,
            identity: identity,
            returnTx: false,
            feeOffer: requiredFee,
            sourceOfFunds: formData.sourceOfFunds || null,
            chain: chainParam
          });

          console.log('Identity registration succeeded:', result);
          const registrationTxId = typeof result === 'string' ? result : (result as any).txid || String(result);

          showSuccess('Success: identity created', { txid: registrationTxId });

          if (onSuccess) {
            onSuccess();
          }
          onClose();

        } else {
          // Error didn't match expected pattern, show original error to user
          error = errorStr;
          showError(errorStr);
          return;
        }
      }

    } catch (err) {
      console.error('Identity registration failed:', err);
      error = typeof err === 'string' ? err : 'Identity registration failed';
      showError(error);
    } finally {
      isSubmitting = false;
      isCalculatingFee = false;
    }
  }
</script>

<Modal {isOpen} onclose={onClose} title="Register Identity with Commitment" size="xl" preventBackdropClose={true}>
  <form onsubmit={handleSubmit} class="p-6">
    <!-- Step 1: Paste Commitment JSON -->
    <div class="mb-6">
      <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
        Step 1: Name Commitment JSON *
      </label>
      <textarea
        bind:value={commitmentJson}
        onblur={validateAndParseCommitment}
        placeholder={exampleJson}
        rows="10"
        class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white font-mono text-sm"
        disabled={isSubmitting}
      ></textarea>
      <div class="flex justify-between items-center mt-2">
        <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
          Paste the complete commitment JSON from your name reservation
        </p>
        <button
          type="button"
          onclick={validateAndParseCommitment}
          class="text-xs text-verusidx-turquoise-deep hover:text-verusidx-turquoise-bright cursor-pointer"
        >
          Validate JSON
        </button>
      </div>
    </div>

      {#if jsonError}
        <div class="mt-2 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
          <p class="text-red-700 dark:text-red-300 text-sm">{jsonError}</p>
        </div>
      {/if}

      {#if commitmentResult}
        <div class="mt-4 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium rounded-lg p-4">
          <h4 class="font-medium text-verusidx-stone-dark dark:text-white mb-2">✓ Parsed Commitment</h4>
          <div class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist space-y-1">
            <p><span class="font-medium">Transaction ID:</span> {commitmentResult.txid}</p>
            <p><span class="font-medium">Identity Name:</span> {commitmentResult.namereservation.name}</p>
            <p><span class="font-medium">Identity Address:</span> {commitmentResult.namereservation.nameid}</p>
            {#if commitmentResult.namereservation.parent}
              <p><span class="font-medium">Parent:</span> {commitmentResult.namereservation.parent}</p>
            {/if}
            {#if commitmentResult.namereservation.referral}
              <p><span class="font-medium">Referral:</span> {commitmentResult.namereservation.referral}</p>
            {/if}
          </div>
        </div>
      {/if}

    {#if commitmentResult}
      <!-- Step 2: Registration Form -->
      <div class="space-y-4 pt-4 border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
        <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">Step 2: Identity Registration</h3>

        <!-- Primary Addresses -->
        <div>
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
            Primary Addresses *
          </label>
          {#each formData.primaryAddresses as _, index}
            <div class="flex space-x-2 mb-2">
              {#if index === 0}
                <!-- First address: dropdown or manual input -->
                {#if !showManualPrimaryInput}
                  <select
                    onchange={handlePrimaryAddressDropdownChange}
                    value={formData.primaryAddresses[index]}
                    required
                    disabled={isLoadingControlAddresses || isSubmitting}
                    class="flex-1 p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
                  >
                    <option value="">
                      {#if isLoadingControlAddresses}
                        Loading addresses...
                      {:else if hasLoadedControlAddresses && controlAddresses.length === 0}
                        No addresses available
                      {:else if !hasLoadedControlAddresses}
                        Failed to load addresses
                      {:else}
                        Select address...
                      {/if}
                    </option>
                    <option value="__MANUAL__">Input address manually</option>
                    {#each controlAddresses as addr}
                      <option value={addr}>{addr}{addr.startsWith('z') ? ' (Private)' : ' (Transparent)'}</option>
                    {/each}
                  </select>
                {:else}
                  <div class="flex-1">
                    <input
                      type="text"
                      bind:value={formData.primaryAddresses[index]}
                      required
                      placeholder="Enter primary address (R-address)"
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
                  required
                  placeholder="Enter address manually"
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
          <button
            type="button"
            onclick={addPrimaryAddress}
            disabled={formData.primaryAddresses.length >= 25 || isSubmitting}
            class="text-sm text-verusidx-turquoise-deep hover:text-verusidx-turquoise-bright disabled:opacity-50"
          >
            + Add another address {formData.primaryAddresses.length >= 25 ? '(Maximum 25 reached)' : `(${formData.primaryAddresses.length}/25)`}
          </button>
        </div>

        <!-- Minimum Signatures -->
        <div>
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
            Minimum Signatures *
          </label>
          <input
            type="number"
            bind:value={formData.minimumSignatures}
            min="1"
            max={formData.primaryAddresses.length}
            required
            class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
            disabled={isSubmitting}
          />
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
                Private Address (z-address)
              </label>
              <select
                bind:value={formData.privateAddress}
                class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
                disabled={isLoadingPrivateAddresses || isSubmitting}
              >
                <option value="">None</option>
                {#each privateAddresses as addr}
                  <option value={addr}>{addr}</option>
                {/each}
              </select>
            </div>
          </div>
        </details>

        <!-- Source of Funds -->
        <div>
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
            Source of Funds
          </label>
          <select
            bind:value={formData.sourceOfFunds}
            class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
            disabled={isLoadingSourceAddresses || isSubmitting}
          >
            <option value="">Select address</option>
            {#each sourceAddresses as addr}
              <option value={addr}>{addr}</option>
            {/each}
          </select>
        </div>
      </div>
    {/if}

    <!-- Error Display -->
    {#if error}
      <div class="mt-4 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
        <p class="text-red-700 dark:text-red-300">{error}</p>
      </div>
    {/if}

    <!-- Address Loading Error -->
    {#if addressLoadingError}
      <div class="mt-4 p-4 bg-orange-50 dark:bg-orange-900/20 border border-orange-200 dark:border-orange-800 rounded-lg">
        <p class="text-orange-700 dark:text-orange-300">{addressLoadingError}</p>
      </div>
    {/if}

    <!-- Buttons -->
    <div class="flex space-x-4 pt-6 mt-6 border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
      <button
        type="button"
        onclick={onClose}
        class="flex-1 px-6 py-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium text-verusidx-stone-dark dark:text-white rounded-lg hover:bg-verusidx-mountain-mist dark:hover:bg-verusidx-stone-medium transition-colors"
        disabled={isSubmitting}
      >
        Cancel
      </button>
      <button
        type="submit"
        disabled={isSubmitting || !commitmentResult || isCalculatingFee}
        class="flex-1 px-6 py-3 bg-verusidx-turquoise-deep hover:bg-verusidx-turquoise-bright text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {#if isCalculatingFee}
          Calculating Fee...
        {:else if isSubmitting}
          Registering Identity...
        {:else}
          Register Identity
        {/if}
      </button>
    </div>
  </form>
</Modal>
