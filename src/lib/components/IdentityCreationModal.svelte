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

  // Wizard step state
  let currentStep = $state<'name-reservation' | 'identity-registration' | 'waiting-confirmation' | 'success'>('name-reservation');
  let commitmentResult = $state<any>(null);
  let registrationTxId = $state<string | null>(null);
  let commitmentBlockHeight = $state<number | null>(null);
  let currentBlockHeight = $state<number | null>(null);
  let copiedCommitment = $state(false);
  let showCloseConfirmation = $state(false);

  // Form state
  let formData = $state({
    // Step 1: Name Reservation
    identityName: '',
    controlAddress: '',
    referral: '',
    parentNameOrId: '',
    sourceOfFunds: '',
    
    // Step 2: Identity Registration  
    primaryAddresses: [''],
    minimumSignatures: 1,
    revocationAuthority: '',
    recoveryAuthority: '',
    privateAddress: ''
  });

  // Available data - separate for different purposes
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
  let isWaitingBlocks = $state(false);
  let error = $state<string | null>(null);
  let addressLoadingError = $state<string | null>(null);
  let blockCheckInterval = $state<NodeJS.Timeout | null>(null);
  
  // Connection state for chain parameter
  let connectionState = $state<any>();
  connectionStore.subscribe(value => { connectionState = value; });
  
  // Fee calculation state (only used during registeridentity step)
  let isCalculatingFee = $state(false);
  let feeCalculationError = $state<string | null>(null);

  // Reset when modal closes
  $effect(() => {
    if (!isOpen && lastLoadedModalOpen !== false) {
      console.log('🔄 Modal closed - resetting wizard');
      resetWizard();
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

  // Fee calculation moved to registerIdentity function - no longer needed here

  function resetWizard() {
    console.log('🔄 Resetting wizard');
    currentStep = 'name-reservation';
    commitmentResult = null;
    formData = {
      identityName: '',
      controlAddress: '',
      referral: '',
      parentNameOrId: '',
      sourceOfFunds: '',
      primaryAddresses: [''],
      minimumSignatures: 1,
      revocationAuthority: '',
      recoveryAuthority: '',
      privateAddress: ''
    };
    error = null;
    addressLoadingError = null;
    isSubmitting = false;
    isWaitingBlocks = false;
    // Reset loading states
    hasLoadedControlAddresses = false;
    hasLoadedSourceAddresses = false;
    hasLoadedPrivateAddresses = false;
    lastLoadedModalOpen = null;
    controlAddresses = [];
    sourceAddresses = [];
    privateAddresses = [];
    // Reset fee calculation state
    isCalculatingFee = false;
    feeCalculationError = null;
    // Reset success state
    registrationTxId = null;
    // Reset block tracking state
    commitmentBlockHeight = null;
    currentBlockHeight = null;
    copiedCommitment = false;
    showCloseConfirmation = false;
    if (blockCheckInterval) {
      clearInterval(blockCheckInterval);
      blockCheckInterval = null;
    }
  }

  async function loadControlAddresses() {
    if (isLoadingControlAddresses || hasLoadedControlAddresses) {
      console.log('⏸️ Skipping loadControlAddresses - loading:', isLoadingControlAddresses, 'hasLoaded:', hasLoadedControlAddresses);
      return;
    }
    
    console.log('🚀 Starting loadControlAddresses');
    isLoadingControlAddresses = true;
    hasLoadedControlAddresses = false;
    
    try {
      // Use getaddressesbyaccount for control addresses (wallet managed)
      console.log('📞 Calling get_addresses_by_account...');
      const chainParam = getChainParam(connectionState?.selectedChain);
      console.log('[IdentityCreationModal]: Loading control addresses for chain:', connectionState?.selectedChain, 'param:', chainParam);
      const addresses = await invoke('get_addresses_by_account', { account: '', chain: chainParam });
      console.log('📝 Control addresses result:', addresses);
      
      if (Array.isArray(addresses) && addresses.length > 0) {
        controlAddresses = addresses;
        hasLoadedControlAddresses = true;
        
        // Don't auto-select - let user choose from dropdown
        console.log('✅ Control address loading completed successfully');
      } else {
        console.log('⚠️ No addresses returned from get_addresses_by_account, trying fallback');
        throw new Error('No addresses returned from get_addresses_by_account');
      }
      
    } catch (err) {
      console.error('❌ Primary control address loading failed:', err);
      // Fallback to address groupings if getaddressesbyaccount fails
      try {
        console.log('📞 Fallback: Calling list_address_groupings...');
        const chainParam = getChainParam(connectionState?.selectedChain);
        console.log('[IdentityCreationModal]: Fallback loading control addresses for chain:', connectionState?.selectedChain, 'param:', chainParam);
        const transparentResult = await invoke('list_address_groupings', { chain: chainParam });
        console.log('📝 Fallback result:', transparentResult);
        const fallbackAddresses = [];
        
        if (Array.isArray(transparentResult)) {
          for (const group of transparentResult) {
            if (Array.isArray(group)) {
              for (const addressInfo of group) {
                if (addressInfo && typeof addressInfo === 'object' && addressInfo.address) {
                  fallbackAddresses.push(addressInfo.address);
                }
              }
            }
          }
        }
        
        if (fallbackAddresses.length > 0) {
          controlAddresses = fallbackAddresses;
          hasLoadedControlAddresses = true;
          // Don't auto-select - let user choose from dropdown
          console.log('✅ Fallback control address loading completed');
        } else {
          throw new Error('No addresses found in fallback method');
        }
      } catch (fallbackErr) {
        console.error('❌ Fallback control address loading failed:', fallbackErr);
        addressLoadingError = `Failed to load control addresses: ${fallbackErr}`;
        hasLoadedControlAddresses = false;
      }
    } finally {
      isLoadingControlAddresses = false;
      console.log('🏁 loadControlAddresses finally block - isLoading set to false');
    }
  }

  async function loadSourceAddresses() {
    if (isLoadingSourceAddresses || hasLoadedSourceAddresses) {
      console.log('⏸️ Skipping loadSourceAddresses - loading:', isLoadingSourceAddresses, 'hasLoaded:', hasLoadedSourceAddresses);
      return;
    }
    
    console.log('🚀 Starting loadSourceAddresses (for source of funds)');
    isLoadingSourceAddresses = true;
    hasLoadedSourceAddresses = false;
    
    try {
      const allAddresses = [];
      
      // Source of funds should only use list_address_groupings + z_list_addresses
      // (NOT get_addresses_by_account - that's only for primary/control addresses)
      
      // Load transparent addresses via list_address_groupings (for R and i addresses)
      try {
        console.log('📞 Calling list_address_groupings for source addresses...');
        const chainParam = getChainParam(connectionState?.selectedChain);
        console.log('[IdentityCreationModal]: Loading source addresses (transparent) for chain:', connectionState?.selectedChain, 'param:', chainParam);
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
        console.error('❌ Failed to load transparent addresses via list_address_groupings:', err);
      }
      
      // Get private addresses from z_list_addresses (for source of funds)
      try {
        console.log('📞 Calling z_list_addresses for source addresses...');
        const chainParam = getChainParam(connectionState?.selectedChain);
        console.log('[IdentityCreationModal]: Loading source addresses (private) for chain:', connectionState?.selectedChain, 'param:', chainParam);
        const privateAddresses = await invoke('z_list_addresses', { chain: chainParam });
        console.log('📝 Private source result:', privateAddresses);
        if (Array.isArray(privateAddresses)) {
          allAddresses.push(...privateAddresses);
          console.log('📍 Added private source addresses:', privateAddresses);
        }
      } catch (err) {
        console.error('❌ Failed to load private source addresses:', err);
      }
      
      console.log('🎯 Final source addresses to set:', allAddresses);
      sourceAddresses = allAddresses;
      hasLoadedSourceAddresses = true;
      
      // Don't auto-populate - let user choose from dropdown
      
      console.log('✅ Source address loading completed successfully');
      
    } catch (err) {
      console.error('❌ Critical error in loadSourceAddresses:', err);
      addressLoadingError = `Failed to load source addresses: ${err}`;
      hasLoadedSourceAddresses = false;
    } finally {
      isLoadingSourceAddresses = false;
      console.log('🏁 loadSourceAddresses finally block - isLoading set to false');
    }
  }

  async function loadPrivateAddresses() {
    if (isLoadingPrivateAddresses || hasLoadedPrivateAddresses) {
      console.log('⏸️ Skipping loadPrivateAddresses - loading:', isLoadingPrivateAddresses, 'hasLoaded:', hasLoadedPrivateAddresses);
      return;
    }
    
    console.log('🚀 Starting loadPrivateAddresses');
    isLoadingPrivateAddresses = true;
    hasLoadedPrivateAddresses = false;
    
    try {
      // Get private addresses from z_list_addresses
      console.log('📞 Calling z_list_addresses...');
      const chainParam = getChainParam(connectionState?.selectedChain);
      console.log('[IdentityCreationModal]: Loading private addresses for chain:', connectionState?.selectedChain, 'param:', chainParam);
      const zAddresses = await invoke('z_list_addresses', { chain: chainParam });
      console.log('📝 Private addresses result:', zAddresses);
      
      if (Array.isArray(zAddresses) && zAddresses.length > 0) {
        privateAddresses = zAddresses;
        hasLoadedPrivateAddresses = true;
        console.log('✅ Private address loading completed successfully');
      } else {
        console.log('⚠️ No private addresses returned');
        privateAddresses = [];
        hasLoadedPrivateAddresses = true;
      }
      
    } catch (err) {
      console.error('❌ Failed to load private addresses:', err);
      addressLoadingError = `Failed to load private addresses: ${err}`;
      privateAddresses = [];
      hasLoadedPrivateAddresses = false;
    } finally {
      isLoadingPrivateAddresses = false;
      console.log('🏁 loadPrivateAddresses finally block - isLoading set to false');
    }
  }

  async function loadAddresses() {
    console.log('🚀 Starting parallel address loading');
    addressLoadingError = null;
    
    // Load all types of addresses in parallel
    await Promise.all([
      loadControlAddresses(),
      loadSourceAddresses(),
      loadPrivateAddresses()
    ]);
    
    console.log('🎯 Address loading completed. Control:', hasLoadedControlAddresses, 'Source:', hasLoadedSourceAddresses, 'Private:', hasLoadedPrivateAddresses);
  }

  // Fee calculation function moved inline to handleIdentityRegistration

  function addPrimaryAddress() {
    if (formData.primaryAddresses.length < 25) {
      formData.primaryAddresses = [...formData.primaryAddresses, ''];
      console.log('➕ Added primary address field, now:', formData.primaryAddresses.length);
    }
  }

  function removePrimaryAddress(index: number) {
    if (formData.primaryAddresses.length > 1) {
      formData.primaryAddresses = formData.primaryAddresses.filter((_, i) => i !== index);
    }
  }

  async function handleNameReservation(event: SubmitEvent) {
    event.preventDefault();
    
    if (!formData.identityName || !formData.controlAddress) {
      error = 'Please fill in required fields';
      return;
    }

    isSubmitting = true;
    error = null;

    try {
      const chainParam = getChainParam(connectionState?.selectedChain);
      console.log('[IdentityCreationModal]: Name reservation for chain:', connectionState?.selectedChain, 'param:', chainParam);
      const result = await invoke('register_name_commitment', {
        name: formData.identityName,
        controlAddress: formData.controlAddress,
        referral: formData.referral || null,
        parentNameOrId: formData.parentNameOrId || null,
        sourceOfFunds: formData.sourceOfFunds || null,
        chain: chainParam
      });

      console.log('Name commitment result:', result);

      if (result && typeof result === 'object') {
        // Store the complete commitment result for registeridentity
        commitmentResult = result;

        if ((result as any).txid) {
          // Get current block height to track confirmation
          try {
            const chainParam = getChainParam(connectionState?.selectedChain);
            const blockCount = await invoke('get_block_count', { chain: chainParam }) as number;
            commitmentBlockHeight = blockCount;
            currentBlockHeight = blockCount;
            console.log('📊 Commitment made at block height:', commitmentBlockHeight);
          } catch (blockErr) {
            console.error('Failed to get block height:', blockErr);
            // Continue anyway - we'll use time-based fallback
          }

          currentStep = 'waiting-confirmation';
          startBlockConfirmationCheck();
        } else {
          error = 'Failed to get transaction ID from commitment';
        }
      } else {
        error = 'Invalid response from name commitment';
      }
      
    } catch (err) {
      console.error('Name reservation failed:', err);
      error = typeof err === 'string' ? err : 'Name reservation failed';
      showError(error);
    } finally {
      isSubmitting = false;
    }
  }

  function startBlockConfirmationCheck() {
    isWaitingBlocks = true;
    console.log('⏳ Starting block confirmation polling...');

    // Check every 10 seconds for new block
    blockCheckInterval = setInterval(async () => {
      try {
        const chainParam = getChainParam(connectionState?.selectedChain);
        const blockCount = await invoke('get_block_count', { chain: chainParam }) as number;
        currentBlockHeight = blockCount;

        console.log('📊 Block check - Commitment:', commitmentBlockHeight, 'Current:', blockCount);

        // Check if a new block has been mined (need at least one confirmation)
        if (commitmentBlockHeight !== null && blockCount > commitmentBlockHeight) {
          console.log('✅ New block confirmed! Advancing to identity registration');
          isWaitingBlocks = false;
          currentStep = 'identity-registration';

          if (blockCheckInterval) {
            clearInterval(blockCheckInterval);
            blockCheckInterval = null;
          }
        }
      } catch (err) {
        console.error('❌ Block check failed:', err);
      }
    }, 10000); // Check every 10 seconds

    // Safety timeout after 15 minutes (in case block times are very long)
    setTimeout(() => {
      if (currentStep === 'waiting-confirmation' && blockCheckInterval) {
        console.log('⚠️ Block confirmation timeout (15 minutes) - proceeding anyway');
        isWaitingBlocks = false;
        currentStep = 'identity-registration';
        clearInterval(blockCheckInterval);
        blockCheckInterval = null;
      }
    }, 900000); // 15 minutes
  }

  async function handleIdentityRegistration(event: SubmitEvent) {
    event.preventDefault();
    
    // Get chain parameter for all RPC calls in this function
    const chainParam = getChainParam(connectionState?.selectedChain);
    console.log('[IdentityCreationModal]: Identity registration for chain:', connectionState?.selectedChain, 'param:', chainParam);
    
    // Validate required fields
    if (!formData.identityName || formData.primaryAddresses.some(addr => !addr.trim()) || !commitmentResult) {
      error = 'Please fill in all required fields and ensure name commitment is complete';
      return;
    }

    isSubmitting = true;
    isCalculatingFee = true;
    error = null;
    feeCalculationError = null;

    try {
      // Build identity object as per Verus RPC spec
      const identity = {
        name: formData.identityName,
        parent: formData.parentNameOrId || undefined,
        primaryaddresses: formData.primaryAddresses.filter(addr => addr.trim()),
        minimumsignatures: formData.minimumSignatures,
        revocationauthority: formData.revocationAuthority || undefined,
        recoveryauthority: formData.recoveryAuthority || undefined,
        privateaddress: formData.privateAddress || undefined
      };

      // Remove undefined values to clean up the object
      Object.keys(identity).forEach(key => {
        if ((identity as any)[key] === undefined) {
          delete (identity as any)[key];
        }
      });

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

        // If first attempt succeeds (shouldn't happen but handle it)
        console.log('✅ Registration succeeded on first attempt:', result);
        registrationTxId = typeof result === 'string' ? result : (result as any).txid || String(result);
        showSuccess('Success: identity created', { txid: registrationTxId });
        currentStep = 'success';
        if (onSuccess) onSuccess();

      } catch (firstErr) {
        // Parse the error message to extract required fee
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

          console.log('✅ Identity registration succeeded:', result);

          // Store transaction ID and transition to success step
          registrationTxId = typeof result === 'string' ? result : (result as any).txid || String(result);

          // Show success toast
          showSuccess('Success: identity created', { txid: registrationTxId });

          currentStep = 'success';

          // Notify parent component if needed
          if (onSuccess) {
            onSuccess();
          }

        } else {
          // Error didn't match expected pattern, show original error to user
          error = errorStr;
          showError(errorStr);
          return; // Don't throw, just show error and stop
        }
      }

    } catch (err) {
      console.error('❌ Identity registration failed:', err);
      error = typeof err === 'string' ? err : 'Identity registration failed';
      showError(error);
    } finally {
      isSubmitting = false;
      isCalculatingFee = false;
    }
  }

  function goBackToNameReservation() {
    currentStep = 'name-reservation';
    isWaitingBlocks = false;
    if (blockCheckInterval) {
      clearInterval(blockCheckInterval);
      blockCheckInterval = null;
    }
  }

  function handleCloseWithConfirmation() {
    if (currentStep === 'waiting-confirmation' && !showCloseConfirmation) {
      showCloseConfirmation = true;
    } else {
      onClose();
      showCloseConfirmation = false;
    }
  }

  function cancelCloseConfirmation() {
    showCloseConfirmation = false;
  }

  async function copyCommitmentToClipboard() {
    if (!commitmentResult) return;

    try {
      // Reorder the JSON to show txid first and namereservation fields in specific order
      const orderedResult = {
        txid: commitmentResult.txid,
        namereservation: {
          version: commitmentResult.namereservation?.version,
          name: commitmentResult.namereservation?.name,
          parent: commitmentResult.namereservation?.parent,
          salt: commitmentResult.namereservation?.salt,
          referral: commitmentResult.namereservation?.referral,
          nameid: commitmentResult.namereservation?.nameid
        }
      };

      const json = JSON.stringify(orderedResult, null, 2);
      await navigator.clipboard.writeText(json);
      copiedCommitment = true;
      setTimeout(() => {
        copiedCommitment = false;
      }, 2000);
    } catch (err) {
      console.error('Failed to copy to clipboard:', err);
    }
  }

  async function copyToClipboard(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      // Could add a toast notification here
      console.log("Copied to clipboard:", text);
    } catch (err) {
      console.error("Failed to copy:", err);
    }
  }

  // Get step title and description
  let stepInfo = $derived(() => {
    switch (currentStep) {
      case 'name-reservation':
        return {
          title: 'Step 1: Name Reservation',
          description: 'Reserve your identity name to prevent others from claiming it',
          icon: '📝'
        };
      case 'waiting-confirmation':
        return {
          title: 'Step 2: Waiting for Confirmation',
          description: 'Waiting for your name commitment to be confirmed on the blockchain',
          icon: '⏳'
        };
      case 'identity-registration':
        return {
          title: 'Step 3: Identity Registration',
          description: 'Complete your identity registration with addresses and settings',
          icon: '👤'
        };
      case 'success':
        return {
          title: 'Success!',
          description: 'Your identity has been created successfully',
          icon: '✅'
        };
      default:
        return { title: '', description: '', icon: '' };
    }
  });
</script>

<Modal {isOpen} onclose={onClose} title="Create New Identity" size="xl" preventBackdropClose={true}>
  <div class="p-6">
    <!-- Step indicator -->
    <div class="flex items-center space-x-3 mb-6">
      <span class="text-3xl">{stepInfo().icon}</span>
      <div>
        <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">{stepInfo().title}</h3>
        <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">{stepInfo().description}</p>
      </div>
    </div>

    <!-- Progress bar -->
    <div class="mb-6">
      <div class="flex items-center">
        <div class="flex-1 h-2 bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium rounded-full overflow-hidden">
          <div 
            class="h-full bg-verusidx-turquoise-deep transition-all duration-300"
            style="width: {currentStep === 'name-reservation' ? '33%' : currentStep === 'waiting-confirmation' ? '66%' : '100%'}"
          ></div>
        </div>
        <span class="ml-3 text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
          {currentStep === 'name-reservation' ? '1' : currentStep === 'waiting-confirmation' ? '2' : '3'} of 3
        </span>
      </div>
    </div>

    <!-- Error Display -->
    {#if error}
      <div class="mb-4 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
        <p class="text-red-700 dark:text-red-300">{error}</p>
      </div>
    {/if}

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

    <!-- Step 1: Name Reservation -->
    {#if currentStep === 'name-reservation'}
      <form onsubmit={handleNameReservation} class="space-y-4">
        <!-- Identity Name -->
        <div>
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
            Identity Name *
          </label>
          <input 
            type="text" 
            bind:value={formData.identityName} 
            required
            placeholder="Enter your identity name (without @)"
            class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
          />
          <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
            Your identity will be: {formData.identityName}@
          </p>
        </div>

        <!-- Control Address -->
        <div>
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
            Control Address *
          </label>
          <select 
            bind:value={formData.controlAddress} 
            required
            disabled={isLoadingControlAddresses}
            class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
          >
            <option value="">
              {#if isLoadingControlAddresses}
                Loading control addresses...
              {:else if hasLoadedControlAddresses && controlAddresses.length === 0}
                No control addresses available
              {:else if !hasLoadedControlAddresses}
                Failed to load addresses
              {:else}
                Select control address...
              {/if}
            </option>
            {#each controlAddresses as address}
              <option value={address}>{address}</option>
            {/each}
          </select>
          <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
            Address from your wallet that will control this commitment
          </p>
        </div>

        <!-- Referral (Optional) -->
        <div>
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
            Referral (Optional)
          </label>
          <input 
            type="text" 
            bind:value={formData.referral} 
            placeholder="e.g. referrer@ or iJhCezBExJHvtyH3fGhNnt2NhU4Ztkf2yq"
            class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
          />
          <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
            Referral identity for discounted registration fees
          </p>
        </div>

        <!-- Parent Name (for Sub-IDs) -->
        <div>
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
            Parent Name (Optional)
          </label>
          <input
            type="text"
            bind:value={formData.parentNameOrId}
            placeholder="e.g. SomeCurrency or currencyname.pbaaschain"
            class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
          />
          <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
            required for subID & PBAAS chains, optional for Root IDs (name.vrsc@)
          </p>
        </div>

        <!-- Source of Funds (Optional) -->
        <div>
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
            Source of Funds (Optional)
          </label>
          <select
            bind:value={formData.sourceOfFunds}
            disabled={isLoadingSourceAddresses}
            class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
          >
            <option value="">
              {#if isLoadingSourceAddresses}
                Loading source addresses...
              {:else if hasLoadedSourceAddresses && sourceAddresses.length === 0}
                Select address (optional)
              {:else if !hasLoadedSourceAddresses}
                Select address (optional)
              {:else}
                Select address (optional)
              {/if}
            </option>
            {#each sourceAddresses as address}
              <option value={address}>{address}{address.startsWith('z') ? ' (Private)' : ' (Transparent)'}</option>
            {/each}
          </select>
          <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
            Address to source funds for fees
          </p>
        </div>

        <!-- Fee information moved to identity registration step -->

        <!-- Buttons -->
        <div class="flex space-x-4 pt-6">
          <button
            type="button"
            onclick={onClose}
            class="flex-1 px-6 py-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium text-verusidx-stone-dark dark:text-white rounded-lg hover:bg-verusidx-mountain-mist dark:hover:bg-verusidx-stone-medium transition-colors"
          >
            Cancel
          </button>
          <button
            type="submit"
            disabled={isSubmitting}
            class="flex-1 px-6 py-3 bg-verusidx-mountain-blue hover:bg-verusidx-lake-blue text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isSubmitting ? 'Reserving...' : 'Reserve Name'}
          </button>
        </div>
      </form>

    <!-- Step 2: Waiting for Confirmation -->
    {:else if currentStep === 'waiting-confirmation'}
      <div class="text-center py-8">
        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-verusidx-turquoise-deep mx-auto mb-4"></div>
        <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-2">
          Waiting for Block Confirmation
        </h3>
        <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-4">
          Your name commitment transaction is being confirmed on the blockchain.
        </p>
        
        {#if commitmentResult}
          <div class="bg-verusidx-snow-ice dark:bg-verusidx-stone-medium rounded-lg p-4 mb-4">
            <div class="flex justify-between items-center mb-2">
              <p class="text-sm font-medium text-verusidx-stone-dark dark:text-white">Name Commitment Data</p>
              <button
                onclick={copyCommitmentToClipboard}
                class="px-3 py-1 text-xs bg-verusidx-mountain-blue hover:bg-verusidx-lake-blue dark:bg-verusidx-turquoise-deep dark:hover:bg-verusidx-turquoise-bright text-white rounded transition-colors"
              >
                {copiedCommitment ? 'Copied!' : 'Copy'}
              </button>
            </div>
            <pre class="text-xs font-mono text-verusidx-mountain-grey dark:text-verusidx-mountain-mist bg-white dark:bg-verusidx-stone-dark p-3 rounded overflow-x-auto max-h-64">{JSON.stringify({
  txid: commitmentResult.txid,
  namereservation: {
    version: commitmentResult.namereservation?.version,
    name: commitmentResult.namereservation?.name,
    parent: commitmentResult.namereservation?.parent,
    salt: commitmentResult.namereservation?.salt,
    referral: commitmentResult.namereservation?.referral,
    nameid: commitmentResult.namereservation?.nameid
  }
}, null, 2)}</pre>
            <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-2">
              Save this data for your records. You'll need it if you need to recover this registration.
            </p>
          </div>
        {/if}

        {#if commitmentBlockHeight !== null && currentBlockHeight !== null}
          <div class="bg-verusidx-lake-deep/10 dark:bg-verusidx-turquoise-deep/20 border border-verusidx-turquoise-light dark:border-verusidx-turquoise-deep rounded-lg p-4 mb-4">
            <p class="text-sm text-verusidx-stone-dark dark:text-white">
              <span class="font-medium">Block Progress:</span>
              {currentBlockHeight} / {commitmentBlockHeight + 1}
            </p>
            <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
              Waiting for block {commitmentBlockHeight + 1} (current: {currentBlockHeight})
            </p>
          </div>
        {/if}

        {#if showCloseConfirmation}
          <div class="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4 mb-4">
            <div class="flex items-start space-x-2 mb-3">
              <span class="text-yellow-600 dark:text-yellow-400 text-xl">⚠️</span>
              <div>
                <h3 class="text-sm font-medium text-yellow-800 dark:text-yellow-200 mb-1">Warning: Close Modal?</h3>
                <p class="text-yellow-700 dark:text-yellow-300 text-sm">
                  Make sure you have copied your name commitment data above before closing. If you lose this data, you may not be able to complete the identity registration.
                </p>
              </div>
            </div>
            <div class="flex space-x-3">
              <button
                onclick={cancelCloseConfirmation}
                class="px-4 py-2 bg-white dark:bg-verusidx-stone-dark text-yellow-800 dark:text-yellow-200 border border-yellow-200 dark:border-yellow-800 rounded-lg hover:bg-yellow-50 dark:hover:bg-verusidx-stone-medium transition-colors text-sm"
              >
                Cancel
              </button>
              <button
                onclick={onClose}
                class="px-4 py-2 bg-yellow-600 hover:bg-yellow-700 dark:bg-yellow-700 dark:hover:bg-yellow-800 text-white rounded-lg transition-colors text-sm"
              >
                Yes, Close Anyway
              </button>
            </div>
          </div>
        {/if}

        <div class="flex justify-center">
          <button
            onclick={handleCloseWithConfirmation}
            class="px-6 py-3 bg-verusidx-mountain-grey hover:bg-verusidx-stone-dark text-white rounded-lg transition-colors"
          >
            Close
          </button>
        </div>
      </div>

    <!-- Step 3: Identity Registration -->
    {:else if currentStep === 'identity-registration'}
      <form onsubmit={handleIdentityRegistration} class="space-y-4">
        <!-- Identity Name (Read-only) -->
        <div>
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
            Identity Name
          </label>
          <input 
            type="text" 
            value={formData.identityName}
            readonly
            class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-stone-dark dark:text-white"
          />
        </div>

        <!-- Commitment Details (Read-only) -->
        {#if commitmentResult}
          <div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium rounded-lg p-4">
            <h4 class="font-medium text-verusidx-stone-dark dark:text-white mb-2">Name Commitment Details</h4>
            <div class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist space-y-1">
              <p><span class="font-medium">Transaction ID:</span> {commitmentResult.txid}</p>
              {#if commitmentResult.namereservation}
                <p><span class="font-medium">Identity Address:</span> {commitmentResult.namereservation.nameid}</p>
                {#if commitmentResult.namereservation.parent}
                  <p><span class="font-medium">Parent:</span> {commitmentResult.namereservation.parent}</p>
                {/if}
                {#if commitmentResult.namereservation.referral}
                  <p><span class="font-medium">Referral:</span> {commitmentResult.namereservation.referral}</p>
                {/if}
              {/if}
            </div>
          </div>
        {/if}

        <!-- Primary Addresses -->
        <div>
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
            Primary Addresses *
          </label>
          {#each formData.primaryAddresses as _, index}
            <div class="flex space-x-2 mb-2">
              {#if index === 0}
                <!-- First address: dropdown from loaded addresses -->
                <select 
                  bind:value={formData.primaryAddresses[index]}
                  required
                  disabled={isLoadingControlAddresses}
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
                  {#each controlAddresses as addr}
                    <option value={addr}>{addr}{addr.startsWith('z') ? ' (Private)' : ' (Transparent)'}</option>
                  {/each}
                </select>
              {:else}
                <!-- Additional addresses: manual text input -->
                <input 
                  type="text"
                  bind:value={formData.primaryAddresses[index]}
                  required
                  placeholder="Enter address manually"
                  class="flex-1 p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
                />
              {/if}
              {#if formData.primaryAddresses.length > 1}
                <button
                  type="button"
                  onclick={() => removePrimaryAddress(index)}
                  class="px-3 py-2 text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
                >
                  Remove
                </button>
              {/if}
            </div>
          {/each}
          <button
            type="button"
            onclick={addPrimaryAddress}
            disabled={formData.primaryAddresses.length >= 25}
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

        <!-- Source of Funds (Optional) -->
        <div>
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
            Source of Funds (Optional)
          </label>
          <select 
            bind:value={formData.sourceOfFunds}
            disabled={isLoadingSourceAddresses}
            class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
          >
            <option value="">
              {#if isLoadingSourceAddresses}
                Loading source addresses...
              {:else if hasLoadedSourceAddresses && sourceAddresses.length === 0}
                Default (transparent wildcard "*") - No addresses found
              {:else if !hasLoadedSourceAddresses}
                Default (transparent wildcard "*") - Failed to load
              {:else}
                Default (transparent wildcard "*")
              {/if}
            </option>
            {#each sourceAddresses as address}
              <option value={address}>
                {address}{address.startsWith('z') ? ' (Private)' : ' (Transparent)'}
              </option>
            {/each}
          </select>
          <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
            Address to source funds for registration fees. Leave empty to use transparent wildcard "*"
          </p>
        </div>


        <!-- Buttons -->
        <div class="flex justify-end pt-6">
          <button
            type="submit"
            disabled={isSubmitting}
            class="px-6 py-3 bg-verusidx-mountain-blue hover:bg-verusidx-lake-blue text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isSubmitting ? 'Creating Identity...' : 'Create Identity'}
          </button>
        </div>
      </form>
    {/if}

    <!-- Success Step -->
    {#if currentStep === 'success'}
      <div class="space-y-6">
        <!-- Success Icon and Message -->
        <div class="text-center">
          <div class="text-6xl mb-4">🎉</div>
          <h3 class="text-2xl font-bold text-verusidx-stone-dark dark:text-white mb-2">
            Identity Created Successfully!
          </h3>
          <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
            Your identity <span class="font-medium text-verusidx-turquoise-deep">{formData.identityName}{formData.parentNameOrId ? `.${formData.parentNameOrId}` : ''}@</span> has been created
          </p>
        </div>

        <!-- Transaction Details -->
        <div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium rounded-lg p-6">
          <h4 class="font-medium text-verusidx-stone-dark dark:text-white mb-3">Transaction Details</h4>
          
          {#if registrationTxId}
            <div class="space-y-3">
              <div>
                <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-1">Transaction ID:</p>
                <div class="flex items-center space-x-2">
                  <code class="flex-1 text-xs font-mono bg-white dark:bg-verusidx-stone-dark p-2 rounded border border-verusidx-mountain-mist dark:border-verusidx-stone-light break-all">
                    {registrationTxId}
                  </code>
                  <button
                    onclick={() => copyToClipboard(registrationTxId || '')}
                    class="px-3 py-2 bg-verusidx-mountain-mist dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-verusidx-mountain-mist text-sm rounded hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-medium transition-colors"
                    title="Copy transaction ID"
                  >
                    Copy
                  </button>
                </div>
              </div>
              
              <div class="pt-3 border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
                <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
                  <span class="text-yellow-600 dark:text-yellow-400">⚠️ Important:</span> Your identity will become active after blockchain confirmation. 
                  This typically takes a few minutes but may vary based on network conditions.
                </p>
              </div>
            </div>
          {/if}
        </div>

        <!-- Next Steps -->
        <div class="bg-verusidx-lake-deep/10 dark:bg-verusidx-turquoise-deep/20 border border-verusidx-turquoise-light dark:border-verusidx-turquoise-deep rounded-lg p-4">
          <h4 class="font-medium text-verusidx-stone-dark dark:text-white mb-2">What's Next?</h4>
          <ul class="text-sm text-verusidx-stone-dark dark:text-white space-y-1">
            <li>• Your identity will appear in the Identity Management section once confirmed</li>
            <li>• You can now use this identity for marketplace offers and transactions</li>
            <li>• Consider setting up multisig or timelocks for additional security</li>
          </ul>
        </div>

        <!-- Close Button -->
        <div class="pt-4">
          <button
            onclick={onClose}
            class="w-full px-6 py-3 bg-verusidx-mountain-blue hover:bg-verusidx-lake-blue text-white rounded-lg transition-colors"
          >
            Close
          </button>
        </div>
      </div>
    {/if}
  </div>
</Modal>