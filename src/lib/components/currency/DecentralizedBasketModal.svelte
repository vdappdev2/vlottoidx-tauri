<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { connectionStore, getChainParam } from "$lib/stores/connection";
  import { Modal } from '../cards';
  import OptionsCheckboxes from './OptionsCheckboxes.svelte';
  import CurrencyWeightManager from './CurrencyWeightManager.svelte';
  import PreallocationManager from './PreallocationManager.svelte';
  import CommonParameters from './CommonParameters.svelte';

  interface Props {
    isOpen: boolean;
    onClose: () => void;
    onSuccess?: () => void;
  }

  let { isOpen = false, onClose, onSuccess }: Props = $props();

  // Form state for basket currency only
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
    minPreconversion: [] as number[],
    maxPreconversion: [] as number[],
    
    // Basket Currency specific  
    basketCurrencies: [''] as string[],
    weights: [0] as number[],
    initialSupply: 0,
    initialContributions: [] as number[],
    prelaunchCarveout: 0,
    prelaunchDiscount: 0,
    idImportFees: 0, // For reserve currency pricing
    useReservePricing: false,
    
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

  // Calculated options value (always 33 for basket currency)
  let finalOptions = $state(33);

  // Derived percentage displays
  let prelaunchDiscountPercentage = $derived((formData.prelaunchDiscount * 100).toFixed(1));
  let prelaunchCarveoutPercentage = $derived((formData.prelaunchCarveout * 100).toFixed(1));

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
      minPreconversion: [],
      maxPreconversion: [],
      basketCurrencies: [''],
      weights: [0],
      initialSupply: 0,
      initialContributions: [],
      prelaunchCarveout: 0,
      prelaunchDiscount: 0,
      idImportFees: 0,
      useReservePricing: false,
      preallocations: []
    };
    error = null;
    isSubmitting = false;
    finalOptions = 33;
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

  function handleCurrencyWeightUpdate(currencies: string[], weights: number[]) {
    formData.basketCurrencies = currencies;
    formData.weights = weights;
  }

  function handleNameUpdate(value: string) {
    formData.name = value;
  }

  function handleInitialSupplyUpdate(value: number) {
    formData.initialSupply = value;
  }

  function handleReservePricingUpdate(value: boolean) {
    formData.useReservePricing = value;
  }

  function handleIdImportFeesUpdate(value: number) {
    formData.idImportFees = value;
  }

  function handleInitialContributionsUpdate(contributions: number[]) {
    formData.initialContributions = contributions;
  }

  function handlePrelaunchDiscountUpdate(value: number) {
    formData.prelaunchDiscount = Math.min(value, 1.0);
  }

  function handlePrelaunchCarveoutUpdate(value: number) {
    formData.prelaunchCarveout = Math.min(value, 1.0);
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

    if (formData.initialSupply <= 0) {
      error = 'Initial supply is required for basket currencies';
      return false;
    }
    
    if (formData.basketCurrencies.some(c => !c.trim())) {
      error = 'All reserve currencies must be specified';
      return false;
    }

    const weightSum = formData.weights.reduce((sum, w) => sum + w, 0);
    if (Math.abs(weightSum - 1.0) > 0.001) {
      error = 'Reserve currency weights must sum to 1.0';
      return false;
    }

    if (formData.weights.some(w => w <= 0)) {
      error = 'All reserve currencies must have non-zero weights';
      return false;
    }

    // Validate prelaunch values don't exceed 1.0 (100%)
    if (formData.prelaunchDiscount > 1.0) {
      error = 'Prelaunch discount cannot exceed 100% (1.0)';
      return false;
    }
    if (formData.prelaunchCarveout > 1.0) {
      error = 'Prelaunch carveout cannot exceed 100% (1.0)';
      return false;
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
        options: finalOptions, // Basket currency = 33 + modifiers
        proofprotocol: 1 // Decentralized
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

      // Basket currency parameters
      currencyDefinition.currencies = formData.basketCurrencies.filter(c => c.trim());
      currencyDefinition.weights = formData.weights;
      
      if (formData.initialSupply > 0) {
        currencyDefinition.initialsupply = formData.initialSupply;
      }
      
      if (formData.initialContributions.length > 0) {
        currencyDefinition.initialcontributions = formData.initialContributions;
      }
      if (formData.prelaunchCarveout > 0) {
        currencyDefinition.prelaunchcarveout = formData.prelaunchCarveout;
      }
      if (formData.prelaunchDiscount > 0) {
        currencyDefinition.prelaunchdiscount = formData.prelaunchDiscount;
      }
      if (formData.useReservePricing && formData.idImportFees >= 0) {
        currencyDefinition.idimportfees = formData.idImportFees / 100000000;
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
      console.log('DecentralizedBasketModal: Defining currency on chain:', connectionState?.selectedChain, 'param:', chainParam);
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

      console.log('Basket currency created successfully:', sendResult);
      
      if (onSuccess) {
        onSuccess();
      }
      onClose();

    } catch (err) {
      console.error('Basket currency creation failed:', err);
      error = typeof err === 'string' ? err : 'Currency creation failed. Please try again.';
    } finally {
      isSubmitting = false;
    }
  }
</script>

<Modal {isOpen} onclose={onClose} title="Create Decentralized Basket Currency" size="xl">
  <div class="p-6">
    <!-- Basket Currency Info -->
    <div class="mb-6 p-4 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
      <p class="text-blue-700 dark:text-blue-300 text-sm">
        🧺 <strong>Basket Currency:</strong> AMM currency backed by reserve currencies. 
        Automatic market-making with weight-based reserves.
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
        oninput={(e) => handleNameUpdate((e.target as HTMLInputElement).value)}
        placeholder="MyBasket"
        required
        class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
      />
      <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
        Must match an identity you control (e.g., if you own "mybasket@", enter "mybasket")
      </p>
    </div>

    <!-- Initial Supply -->
    <div class="border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium pt-6 mt-6 mb-6">
      <h4 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-4">
        Basket Configuration
      </h4>
      
      <div class="mb-6">
        <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
          Initial Supply *
        </label>
        <input 
          type="number" 
          value={formData.initialSupply}
          oninput={(e) => handleInitialSupplyUpdate(parseFloat((e.target as HTMLInputElement).value) || 0)}
          min="0"
          step="0.00000001"
          required
          class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
        />
        <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
          Initial supply distributed among preconverters
        </p>
      </div>

      <!-- Reserve Currencies & Weights -->
      <div class="mb-6">
        <CurrencyWeightManager
          currencies={formData.basketCurrencies}
          weights={formData.weights}
          onUpdate={handleCurrencyWeightUpdate}
        />
      </div>

      <!-- Reserve Pricing Option -->
      <div class="mb-6">
        <label class="flex items-center space-x-2 mb-2">
          <input 
            type="checkbox" 
            checked={formData.useReservePricing}
            onchange={(e) => handleReservePricingUpdate((e.target as HTMLInputElement).checked)}
            class="text-verusidx-forest-deep focus:ring-verusidx-forest-deep"
          />
          <span class="text-sm font-medium text-verusidx-stone-dark dark:text-white">
            Price sub-identities in reserve currency
          </span>
        </label>
        
        {#if formData.useReservePricing}
          <div class="mt-2">
            <label class="block text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-1">
              Reserve currency index for pricing:
            </label>
            <select 
              value={formData.idImportFees}
              onchange={(e) => handleIdImportFeesUpdate(parseInt((e.target as HTMLSelectElement).value) || 0)}
              class="w-full p-2 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white text-sm"
            >
              {#each formData.basketCurrencies as curr, index}
                {#if curr.trim()}
                  <option value={index}>
                    {curr} (index {index})
                  </option>
                {/if}
              {/each}
            </select>
          </div>
        {/if}
      </div>
    </div>

    <!-- Basket Allocation Parameters -->
    <div class="border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium pt-6 mt-6">
      <h4 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-4">
        Basket Allocation Parameters (Optional)
      </h4>
      
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
        <!-- Initial Contributions -->
        <div>
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
            Initial Contributions
          </label>
          <div class="space-y-2">
            {#each formData.basketCurrencies as currency, index}
              {#if currency.trim()}
                <div>
                  <label class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-1 block">
                    {currency}
                  </label>
                  <input 
                    type="number" 
                    value={formData.initialContributions[index] || 0}
                    oninput={(e) => {
                      const newContributions = [...formData.initialContributions];
                      newContributions[index] = parseFloat((e.target as HTMLInputElement).value) || 0;
                      handleInitialContributionsUpdate(newContributions);
                    }}
                    min="0"
                    step="0.00000001"
                    placeholder="0"
                    class="w-full p-2 text-sm border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
                  />
                </div>
              {/if}
            {/each}
          </div>
          <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
            Initial reserve contributions per currency
          </p>
        </div>

        <!-- Prelaunch Discount -->
        <div>
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
            Prelaunch Discount
          </label>
          <input 
            type="number" 
            value={formData.prelaunchDiscount}
            oninput={(e) => handlePrelaunchDiscountUpdate(parseFloat((e.target as HTMLInputElement).value) || 0)}
            min="0"
            max="1"
            step="0.001"
            placeholder="0"
            class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
          />
          <p class="text-xs text-verusidx-turquoise-deep dark:text-verusidx-turquoise-bright font-medium mt-1">
            {prelaunchDiscountPercentage}%
          </p>
          <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
            Discount for early preconverters
          </p>
        </div>

        <!-- Prelaunch Carveout -->
        <div>
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
            Prelaunch Carveout
          </label>
          <input 
            type="number" 
            value={formData.prelaunchCarveout}
            oninput={(e) => handlePrelaunchCarveoutUpdate(parseFloat((e.target as HTMLInputElement).value) || 0)}
            min="0"
            max="1"
            step="0.001"
            placeholder="0"
            class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
          />
          <p class="text-xs text-verusidx-turquoise-deep dark:text-verusidx-turquoise-bright font-medium mt-1">
            {prelaunchCarveoutPercentage}%
          </p>
          <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
            Amount reserved for currency creator
          </p>
        </div>
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
        baseOptions={33}
        selectedModifiers={formData.optionModifiers}
        onUpdate={handleOptionsUpdate}
        onModifiersUpdate={handleModifiersUpdate}
      />
    </div>

    <!-- Common Parameters -->
    <CommonParameters
      formData={formData}
      onUpdate={handleCommonParameterUpdate}
      currencies={formData.basketCurrencies}
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
        class="flex-1 px-6 py-3 bg-verusidx-forest-deep hover:bg-verusidx-turquoise-deep text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {isSubmitting ? 'Creating Basket Currency...' : 'Create Basket Currency'}
      </button>
    </div>
  </div>
</Modal>