<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { connectionStore, getChainParam } from "$lib/stores/connection";
  import { Modal } from '../cards';
  import OptionsCheckboxes from './OptionsCheckboxes.svelte';
  import CurrencyConversionManager from './CurrencyConversionManager.svelte';
  import PreallocationManager from './PreallocationManager.svelte';
  import CommonParameters from './CommonParameters.svelte';

  interface Props {
    isOpen: boolean;
    onClose: () => void;
    onSuccess?: () => void;
  }

  let { isOpen = false, onClose, onSuccess }: Props = $props();

  // Form state for centralized simple token
  let formData = $state({
    name: '',
    
    // Options modifiers
    optionModifiers: {
      onlyIdCanCreateSubIds: false,
      enableReferrals: false,
      referralsRequired: false
    },
    
    // Common parameters (optional)
    idRegistrationFees: undefined,
    idReferralLevels: undefined,
    startBlock: '',
    endBlock: '', // Centralized specific
    minPreconversion: [] as number[],
    maxPreconversion: [] as number[],
    
    // Simple Token specific
    currencies: [] as string[],
    conversions: [] as number[],
    
    // Preallocations
    preallocations: [] as {address: string, amount: number}[]
  });

  // State management
  let isSubmitting = $state(false);
  let error = $state<string | null>(null);
  let connectionState = $state<any>(null);

  connectionStore.subscribe(state => {
    connectionState = state;
  });

  // Calculated options value (always 32 for simple token)
  let finalOptions = $state(32);

  // Reset when modal opens/closes
  $effect(() => {
    if (!isOpen) {
      resetForm();
    }
  });

  function resetForm() {
    formData = {
      name: '',
      optionModifiers: {
        onlyIdCanCreateSubIds: false,
        enableReferrals: false,
        referralsRequired: false
      },
      idRegistrationFees: undefined,
      idReferralLevels: undefined,
      startBlock: '',
      endBlock: '',
      minPreconversion: [],
      maxPreconversion: [],
      currencies: [],
      conversions: [],
      preallocations: []
    };
    error = null;
    isSubmitting = false;
    finalOptions = 32;
  }

  function handleOptionsUpdate(newOptions: number) {
    finalOptions = newOptions;
  }

  function handleCommonParameterUpdate(field: string, value: any) {
    (formData as any)[field] = value;
  }

  function handlePreallocationsUpdate(preallocations: {address: string, amount: number}[]) {
    formData.preallocations = preallocations;
  }

  function handleCurrencyConversionUpdate(currencies: string[], conversions: number[]) {
    formData.currencies = currencies;
    formData.conversions = conversions;
  }

  function handleNameUpdate(value: string) {
    formData.name = value;
  }

  function handleModifiersUpdate(newModifiers: { onlyIdCanCreateSubIds: boolean; enableReferrals: boolean; referralsRequired: boolean; }) {
    formData.optionModifiers = newModifiers;
  }

  // Validation
  function validateForm(): boolean {
    if (!formData.name.trim()) {
      error = 'Currency name is required';
      return false;
    }

    // Simple token validation
    if (formData.currencies.length > 0) {
      if (formData.currencies.length !== formData.conversions.length) {
        error = 'Number of currencies and conversions must match';
        return false;
      }
    } else {
      // If no currencies/conversions, must have preallocations
      if (formData.preallocations.length === 0) {
        error = 'Simple tokens require either currencies/conversions or preallocations';
        return false;
      }
    }

    // Validate endBlock if specified
    if (formData.endBlock && formData.startBlock) {
      const start = parseInt(formData.startBlock);
      const end = parseInt(formData.endBlock);
      if (end <= start) {
        error = 'End block must be greater than start block';
        return false;
      }
    }

    return true;
  }

  async function handleSubmit() {
    if (!validateForm()) return;

    isSubmitting = true;
    error = null;

    try {
      // Build currency definition
      const currencyDefinition: any = {
        name: formData.name,
        options: finalOptions, // Simple token = 32 + modifiers
        proofprotocol: 2 // Centralized
      };

      // Add optional parameters only if specified
      if (formData.idRegistrationFees !== undefined) {
        currencyDefinition.idregistrationfees = formData.idRegistrationFees;
      }
      if (formData.idReferralLevels !== undefined) {
        currencyDefinition.idreferrallevels = formData.idReferralLevels;
      }

      // Add conditional parameters
      if (formData.startBlock) {
        currencyDefinition.startblock = parseInt(formData.startBlock);
      }
      if (formData.endBlock) {
        currencyDefinition.endblock = parseInt(formData.endBlock);
      }

      // Simple token parameters
      if (formData.currencies.length > 0) {
        currencyDefinition.currencies = formData.currencies.filter(c => c.trim());
        currencyDefinition.conversions = formData.conversions;
      }

      // Add preconversion limits if specified
      if (formData.minPreconversion.some(v => v > 0)) {
        currencyDefinition.minpreconversion = formData.minPreconversion;
      }
      if (formData.maxPreconversion.some(v => v > 0)) {
        currencyDefinition.maxpreconversion = formData.maxPreconversion;
      }

      // Add preallocations if specified
      if (formData.preallocations.length > 0) {
        const preallocations = formData.preallocations
          .filter(p => p.address.trim() && p.amount > 0)
          .reduce((acc, p) => {
            acc[p.address] = p.amount;
            return acc;
          }, {} as Record<string, number>);
        
        if (Object.keys(preallocations).length > 0) {
          currencyDefinition.preallocations = [preallocations];
        }
      }

      // Step 1: Define currency
      const chainParam = getChainParam(connectionState?.selectedChain);
      console.log('CentralizedSimpleTokenModal: Defining currency on chain:', connectionState?.selectedChain, 'param:', chainParam);
      const defineResult = await invoke('define_currency', {
        currencyDefinition,
        fractionalGateway: null,
        reserves: null,
        chain: chainParam
      });

      if (!defineResult || !(defineResult as any).hex) {
        throw new Error('Failed to get transaction hex from define currency');
      }

      // Step 2: Send raw transaction
      const sendResult = await invoke('send_raw_transaction', {
        hexData: (defineResult as any).hex,
        chain: chainParam
      });

      console.log('Centralized simple token created successfully:', sendResult);
      
      if (onSuccess) {
        onSuccess();
      }
      onClose();

    } catch (err) {
      console.error('Centralized simple token creation failed:', err);
      error = typeof err === 'string' ? err : 'Currency creation failed. Please try again.';
    } finally {
      isSubmitting = false;
    }
  }
</script>

<Modal {isOpen} onclose={onClose} title="Create Centralized Simple Token" size="xl">
  <div class="p-6">
    <!-- Centralized Simple Token Info -->
    <div class="mb-6 p-4 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
      <p class="text-blue-700 dark:text-blue-300 text-sm">
        🏦 <strong>Centralized Simple Token:</strong> Currency you can mint and burn even after launch. 
        Optionally set an end block to automatically become decentralized.
      </p>
    </div>

    <!-- Currency Name -->
    <div class="mb-6">
      <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
        Currency Name *
      </label>
      <input 
        type="text" 
        value={formData.name}
        oninput={(e) => handleNameUpdate(e.target.value)}
        placeholder="MyToken"
        required
        class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
      />
      <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
        Must match an identity you control (e.g., if you own "mytoken@", enter "mytoken")
      </p>
    </div>

    <!-- Preconversion Currencies (Optional) -->
    <div class="border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium pt-6 mt-6">
      <h4 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-4">
        Preconversion Setup (Optional)
      </h4>
      <div class="mb-6">
        <CurrencyConversionManager
          currencies={formData.currencies}
          conversions={formData.conversions}
          onUpdate={handleCurrencyConversionUpdate}
        />
      </div>
    </div>

    <!-- Preallocations -->
    <div class="border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium pt-6 mt-6">
      <PreallocationManager
        preallocations={formData.preallocations}
        onUpdate={handlePreallocationsUpdate}
      />
    </div>

    <!-- Sub-Identity Control Options -->
    <div class="border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium pt-6 mt-6">
      <OptionsCheckboxes 
        baseOptions={32}
        selectedModifiers={formData.optionModifiers}
        onUpdate={handleOptionsUpdate}
        onModifiersUpdate={handleModifiersUpdate}
      />
    </div>

    <!-- Common Parameters (includes endBlock for centralized) -->
    <CommonParameters
      formData={formData}
      onUpdate={handleCommonParameterUpdate}
      showEndBlock={true}
      currencies={formData.currencies}
    />

    <!-- Error Display -->
    {#if error}
      <div class="mt-4 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
        <p class="text-red-700 dark:text-red-300">{error}</p>
      </div>
    {/if}

    <!-- Buttons -->
    <div class="flex space-x-4 pt-6 border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium mt-6">
      <button
        type="button"
        onclick={onClose}
        class="flex-1 px-6 py-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium text-verusidx-stone-dark dark:text-white rounded-lg hover:bg-verusidx-mountain-mist dark:hover:bg-verusidx-stone-medium transition-colors"
      >
        Cancel
      </button>
      <button
        type="button"
        onclick={handleSubmit}
        disabled={isSubmitting}
        class="flex-1 px-6 py-3 bg-verusidx-turquoise-deep hover:bg-verusidx-turquoise-bright text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {isSubmitting ? 'Creating Centralized Token...' : 'Create Centralized Token'}
      </button>
    </div>
  </div>
</Modal>