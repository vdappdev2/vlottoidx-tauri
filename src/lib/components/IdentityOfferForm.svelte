<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  // Props
  interface Props {
    identityData: {
      name: string;
      parent: string;
      primaryAddresses: string[];
      minimumSignatures: number;
      revocationAuthority: string;
      recoveryAuthority: string;
      privateAddress: string;
    };
    onIdentityChange: (identityData: any) => void;
    mode?: 'offer' | 'accept';
    disabled?: boolean;
  }

  let {
    identityData = {
      name: '',
      parent: '',
      primaryAddresses: [''],
      minimumSignatures: 1,
      revocationAuthority: '',
      recoveryAuthority: '',
      privateAddress: ''
    },
    onIdentityChange,
    mode = 'offer',
    disabled = false
  }: Props = $props();

  // Create local reactive copy to avoid prop mutation issues
  let localIdentityData = $state({
    name: identityData.name || '',
    parent: identityData.parent || '',
    primaryAddresses: [...(identityData.primaryAddresses || [''])],
    minimumSignatures: identityData.minimumSignatures || 1,
    revocationAuthority: identityData.revocationAuthority || '',
    recoveryAuthority: identityData.recoveryAuthority || '',
    privateAddress: identityData.privateAddress || ''
  });

  // Note: Removed automatic prop syncing to prevent resetting user changes

  // Ensure primaryAddresses always has at least one element
  $effect(() => {
    if (!localIdentityData.primaryAddresses || localIdentityData.primaryAddresses.length === 0) {
      localIdentityData.primaryAddresses = [''];
      notifyChange();
    }
  });

  // Available data - reuse patterns from identity creation wizard
  let sourceAddresses = $state<string[]>([]);
  let privateAddresses = $state<string[]>([]);
  let isLoadingSourceAddresses = $state(false);
  let isLoadingPrivateAddresses = $state(false);
  let hasLoadedSourceAddresses = $state(false);
  let hasLoadedPrivateAddresses = $state(false);
  let addressLoadingError = $state<string | null>(null);

  // Load addresses when component mounts
  $effect(() => {
    if (!hasLoadedSourceAddresses && !isLoadingSourceAddresses) {
      loadAddresses();
    }
  });

  async function loadSourceAddresses() {
    if (isLoadingSourceAddresses || hasLoadedSourceAddresses) return;
    
    isLoadingSourceAddresses = true;
    hasLoadedSourceAddresses = false;
    
    try {
      const allAddresses = [];
      
      // Get addresses by account (default account "")
      try {
        const addresses = await invoke('get_addresses_by_account', { account: "" });
        if (Array.isArray(addresses)) {
          allAddresses.push(...addresses);
          console.log('✅ Loaded addresses from default account:', addresses.length);
        }
      } catch (err) {
        console.error('Failed to load addresses by account:', err);
      }
      
      // Also get private addresses from z_list_addresses
      try {
        const privateAddressList = await invoke('z_list_addresses');
        if (Array.isArray(privateAddressList)) {
          allAddresses.push(...privateAddressList);
          console.log('✅ Loaded private addresses:', privateAddressList.length);
        }
      } catch (err) {
        console.error('Failed to load private addresses:', err);
      }
      
      sourceAddresses = allAddresses;
      hasLoadedSourceAddresses = true;
      console.log('✅ IdentityOfferForm: Loaded', allAddresses.length, 'source addresses');
      
      // Do not auto-populate - let user explicitly select an address
      
    } catch (err) {
      console.error('Critical error in loadSourceAddresses:', err);
      addressLoadingError = `Failed to load source addresses: ${err}`;
      hasLoadedSourceAddresses = false;
    } finally {
      isLoadingSourceAddresses = false;
    }
  }

  async function loadPrivateAddresses() {
    if (isLoadingPrivateAddresses || hasLoadedPrivateAddresses) return;
    
    isLoadingPrivateAddresses = true;
    hasLoadedPrivateAddresses = false;
    
    try {
      const zAddresses = await invoke('z_list_addresses');
      if (Array.isArray(zAddresses) && zAddresses.length > 0) {
        privateAddresses = zAddresses;
        hasLoadedPrivateAddresses = true;
      } else {
        privateAddresses = [];
        hasLoadedPrivateAddresses = true;
      }
    } catch (err) {
      console.error('Failed to load private addresses:', err);
      addressLoadingError = `Failed to load private addresses: ${err}`;
      privateAddresses = [];
      hasLoadedPrivateAddresses = false;
    } finally {
      isLoadingPrivateAddresses = false;
    }
  }

  async function loadAddresses() {
    addressLoadingError = null;
    await Promise.all([
      loadSourceAddresses(),
      loadPrivateAddresses()
    ]);
  }

  function addPrimaryAddress() {
    if (localIdentityData.primaryAddresses.length < 25) {
      localIdentityData.primaryAddresses = [...localIdentityData.primaryAddresses, ''];
      notifyChange();
    }
  }

  function removePrimaryAddress(index: number) {
    if (localIdentityData.primaryAddresses.length > 1) {
      localIdentityData.primaryAddresses = localIdentityData.primaryAddresses.filter((_, i) => i !== index);
      notifyChange();
    }
  }

  function notifyChange() {
    // Pass camelCase for internal component communication (consistency with other forms)
    // The conversion to lowercase for RPC happens in the parent component
    const identity = {
      name: localIdentityData.name,
      parent: localIdentityData.parent || '',
      primaryAddresses: localIdentityData.primaryAddresses, // Keep camelCase for internal state
      minimumSignatures: localIdentityData.minimumSignatures, // Keep camelCase for internal state
      revocationAuthority: localIdentityData.revocationAuthority || '',
      recoveryAuthority: localIdentityData.recoveryAuthority || '',
      privateAddress: localIdentityData.privateAddress || ''
    };

    onIdentityChange(identity);
  }

  // Call notifyChange whenever form data changes
  $effect(() => {
    notifyChange();
  });
</script>

<div class="space-y-4">
  <div class="flex items-center space-x-2 mb-4">
    <span class="text-lg">👤</span>
    <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">
      Identity Definition
    </h3>
    <span class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
      ({mode === 'offer' ? 'What you\'re offering' : 'What you\'ll accept'})
    </span>
  </div>

  <!-- Address Loading Error Display -->
  {#if addressLoadingError}
    <div class="mb-4 p-4 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg">
      <div class="flex items-center space-x-2 mb-2">
        <span class="text-yellow-600 dark:text-yellow-400">⚠️</span>
        <h3 class="text-sm font-medium text-yellow-800 dark:text-yellow-200">Address Loading Issue</h3>
      </div>
      <p class="text-yellow-700 dark:text-yellow-300 text-sm">{addressLoadingError}</p>
      <button
        type="button"
        onclick={() => { addressLoadingError = null; loadAddresses(); }}
        class="mt-2 px-3 py-1 text-xs bg-yellow-100 dark:bg-yellow-800 text-yellow-800 dark:text-yellow-100 rounded hover:bg-yellow-200 dark:hover:bg-yellow-700 transition-colors"
      >
        Retry Loading Addresses
      </button>
    </div>
  {/if}

  <!-- Identity Name -->
  <div>
    <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
      Identity Name *
    </label>
    <input 
      type="text" 
      bind:value={localIdentityData.name} 
      required
      disabled={disabled}
      placeholder="Enter identity name (without @)"
      class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
    />
    <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
      Identity will be: {localIdentityData.name}@
    </p>
  </div>

  <!-- Parent -->
  <div>
    <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
      Parent Currency *
    </label>
    <input 
      type="text" 
      bind:value={localIdentityData.parent} 
      required
      disabled={disabled}
      placeholder="e.g. VRSC, vDEX, vETH, Pure"
      class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
    />
    <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
      The parent namespace of this identity
    </p>
  </div>

  <!-- Primary Addresses -->
  <div>
    <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
      Primary Addresses *
    </label>
    {#if localIdentityData.primaryAddresses && localIdentityData.primaryAddresses.length > 0}
      {#each localIdentityData.primaryAddresses as _, index}
        <div class="flex space-x-2 mb-2">
          {#if index === 0}
            <!-- First address: dropdown from loaded addresses -->
            <select 
              bind:value={localIdentityData.primaryAddresses[index]}
              required
              disabled={disabled || isLoadingSourceAddresses}
              class="flex-1 p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
            >
              <option value="">
                {#if isLoadingSourceAddresses}
                  Loading addresses...
                {:else if hasLoadedSourceAddresses && sourceAddresses.length === 0}
                  No addresses available
                {:else if !hasLoadedSourceAddresses}
                  Failed to load addresses
                {:else}
                  Select address
                {/if}
              </option>
              {#each sourceAddresses as addr}
                <option value={addr}>{addr}{addr.startsWith('z') ? ' (Private)' : ' (Transparent)'}</option>
              {/each}
            </select>
          {:else}
            <!-- Additional addresses: manual text input -->
            <input 
              type="text"
              bind:value={localIdentityData.primaryAddresses[index]}
              required
              disabled={disabled}
              placeholder="Enter address manually"
              class="flex-1 p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
            />
          {/if}
          {#if localIdentityData.primaryAddresses.length > 1}
            <button
              type="button"
              onclick={() => removePrimaryAddress(index)}
              disabled={disabled}
              class="px-3 py-2 text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors disabled:opacity-50"
            >
              Remove
            </button>
          {/if}
        </div>
      {/each}
    {:else}
      <!-- Fallback: Show at least one field if array is empty -->
      <div class="flex space-x-2 mb-2">
        <select 
          disabled={disabled || isLoadingSourceAddresses}
          class="flex-1 p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
        >
          <option value="">
            {#if isLoadingSourceAddresses}
              Loading addresses...
            {:else}
              No addresses loaded - check array initialization
            {/if}
          </option>
        </select>
      </div>
    {/if}
    <button
      type="button"
      onclick={addPrimaryAddress}
      disabled={disabled || localIdentityData.primaryAddresses.length >= 25}
      class="text-sm text-verusidx-turquoise-deep hover:text-verusidx-turquoise-bright disabled:opacity-50"
    >
      + Add another address {localIdentityData.primaryAddresses.length >= 25 ? '(Maximum 25 reached)' : `(${localIdentityData.primaryAddresses.length}/25)`}
    </button>
  </div>

  <!-- Minimum Signatures -->
  <div>
    <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
      Minimum Signatures *
    </label>
    <input 
      type="number" 
      bind:value={localIdentityData.minimumSignatures} 
      min="1"
      max={localIdentityData.primaryAddresses.filter(addr => addr.trim()).length || 1}
      required
      disabled={disabled}
      class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
    />
    <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
      How many signatures are required to control this identity
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
          bind:value={localIdentityData.revocationAuthority} 
          disabled={disabled}
          placeholder="Address that can revoke this identity"
          class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
        />
      </div>

      <!-- Recovery Authority -->
      <div>
        <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
          Recovery Authority
        </label>
        <input 
          type="text" 
          bind:value={localIdentityData.recoveryAuthority} 
          disabled={disabled}
          placeholder="Address that can recover this identity"
          class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
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
            bind:value={localIdentityData.privateAddress}
            disabled={disabled}
            class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
          >
            <option value="">Select a private address (optional)</option>
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

  <!-- Preview -->
  <div class="mt-4 p-4 bg-verusidx-snow-ice dark:bg-verusidx-stone-medium rounded-lg">
    <h4 class="font-medium text-verusidx-stone-dark dark:text-white mb-2">Identity Preview</h4>
    <div class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist space-y-1">
      <p><span class="font-medium">Full Name:</span> {localIdentityData.name}{localIdentityData.parent ? `.${localIdentityData.parent}` : ''}@</p>
      <p><span class="font-medium">Primary Addresses:</span> {localIdentityData.primaryAddresses.filter(addr => addr.trim()).length}</p>
      <p><span class="font-medium">Required Signatures:</span> {localIdentityData.minimumSignatures}</p>
      {#if localIdentityData.revocationAuthority}
        <p><span class="font-medium">Revocation Authority:</span> {localIdentityData.revocationAuthority}</p>
      {/if}
      {#if localIdentityData.recoveryAuthority}
        <p><span class="font-medium">Recovery Authority:</span> {localIdentityData.recoveryAuthority}</p>
      {/if}
      {#if localIdentityData.privateAddress}
        <p><span class="font-medium">Private Address:</span> {localIdentityData.privateAddress}</p>
      {/if}
    </div>
  </div>
</div>