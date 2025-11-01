<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Modal } from './cards';
  import { showSuccess, showError } from '$lib/services/notifications';

  // Props
  interface Props {
    isOpen: boolean;
    onClose: () => void;
    onSuccess?: () => void;
  }

  let { isOpen = false, onClose, onSuccess }: Props = $props();

  // Wizard step state
  let currentStep = $state<'currency-design' | 'launch-parameters' | 'review-and-confirm' | 'processing' | 'waiting-blocks'>('currency-design');
  let transactionHash = $state<string>('');
  let isAdvancedMode = $state(false);

  // Form state
  let formData = $state({
    // Step 1: Currency Design
    name: '',
    currencyType: 'simple', // 'simple' | 'basket' | 'erc20'
    proofProtocol: 1, // 1 = decentralized, 2 = centralized, 3 = eth bridge
    idRegistrationFees: 100,
    idReferralLevels: 3,
    useReservePricingForSubIDs: false, // Whether to use idimportfees
    idImportFeesReserveIndex: 0, // Which reserve currency to use for pricing
    
    // Step 2: Launch Parameters
    startBlock: '',
    endBlock: '',
    initialSupply: 0,
    currencies: [''], // Reserve currencies for basket type
    weights: [1.0], // Weights for basket currencies
    conversions: [], // Conversion rates for preconversion
    minPreconversion: [], // Minimum preconversion amounts
    maxPreconversion: [], // Maximum preconversion amounts
    initialContributions: [], // Initial contributions from rootID
    prelaunchCarveout: 0,
    prelaunchDiscount: 0,
    preallocations: [] as {identity: string, amount: number}[],
    
    // Advanced parameters (all optional)
    notarizationProtocol: 1,
    notarizationReward: '',
    minNotariesConfirm: '',
    proofProtocolAdvanced: '',
    blockTime: '',
    powAveragingWindow: '',
    notarizationPeriod: '',
    gatewayConverterName: '',
    nativeCurrencyId: '',
    systemId: '',
    parent: '',
    launchSystemId: '',
    expiryHeight: '',
    premine: '',
    nodes: [] as {networkAddress: string, nodeIdentity: string}[],
    eras: [] as {reward: number, decay?: number, halving?: number, eraEnd?: number}[],
    notaries: [] as string[],
    reserves: [] as number[],
    
    // Raw JSON mode
    rawJsonMode: false,
    rawJsonInput: ''
  });

  // Available data
  let availableIdentities = $state<string[]>([]);
  let availableCurrencies = $state<string[]>([]);
  let isLoadingData = $state(false);
  let isSubmitting = $state(false);
  let error = $state<string | null>(null);

  // Reset when modal opens/closes
  $effect(() => {
    if (!isOpen) {
      resetWizard();
    } else {
      loadAvailableData();
    }
  });

  function resetWizard() {
    currentStep = 'currency-design';
    transactionHash = '';
    formData = {
      name: '',
      currencyType: 'simple',
      proofProtocol: 1,
      idRegistrationFees: 100,
      idReferralLevels: 3,
      useReservePricingForSubIDs: false,
      idImportFeesReserveIndex: 0,
      startBlock: '',
      endBlock: '',
      initialSupply: 0,
      currencies: [''],
      weights: [1.0],
      conversions: [],
      minPreconversion: [],
      maxPreconversion: [],
      initialContributions: [],
      prelaunchCarveout: 0,
      prelaunchDiscount: 0,
      preallocations: [],
      notarizationProtocol: 1,
      notarizationReward: '',
      minNotariesConfirm: '',
      proofProtocolAdvanced: '',
      blockTime: '',
      powAveragingWindow: '',
      notarizationPeriod: '',
      gatewayConverterName: '',
      nativeCurrencyId: '',
      systemId: '',
      parent: '',
      launchSystemId: '',
      expiryHeight: '',
      premine: '',
      nodes: [],
      eras: [],
      notaries: [],
      reserves: [],
      rawJsonMode: false,
      rawJsonInput: ''
    };
    error = null;
    isSubmitting = false;
  }

  async function loadAvailableData() {
    if (isLoadingData) return;
    
    isLoadingData = true;
    try {
      // Load available identities
      const identitiesResult = await invoke('list_identities', { 
        can_sign_for: true, 
        only_root: true 
      });
      
      if (Array.isArray(identitiesResult)) {
        availableIdentities = identitiesResult.map((id: any) => id.identity?.name || '').filter(Boolean);
      }

      // Load available currencies for basket reserves
      const currenciesResult = await invoke('list_currencies', {
        query: null,
        verbose: false
      });
      
      if (Array.isArray(currenciesResult)) {
        availableCurrencies = currenciesResult.map((currency: any) => {
          const currencyDef = currency.currencydefinition || currency;
          return currencyDef?.name || currencyDef?.currencyid || '';
        }).filter(Boolean);
      }
      
    } catch (err) {
      console.error('Failed to load available data:', err);
    } finally {
      isLoadingData = false;
    }
  }

  function addReserveCurrency() {
    formData.currencies = [...formData.currencies, ''];
    formData.weights = [...formData.weights, 0.1];
  }

  function removeReserveCurrency(index: number) {
    if (formData.currencies.length > 1) {
      formData.currencies = formData.currencies.filter((_, i) => i !== index);
      formData.weights = formData.weights.filter((_, i) => i !== index);
    }
  }

  function addPreallocation() {
    formData.preallocations = [...formData.preallocations, { identity: '', amount: 0 }];
  }

  function removePreallocation(index: number) {
    formData.preallocations = formData.preallocations.filter((_, i) => i !== index);
  }

  function calculateOptions(): number {
    let options = 0;
    
    if (formData.currencyType === 'basket') {
      options |= 1; // FRACTIONAL
    }
    
    options |= 32; // TOKEN (required for all currencies)
    
    // Add ID referrals if levels > 0
    if (formData.idReferralLevels > 0) {
      options |= 8; // IDREFERRALS
    }
    
    return options;
  }

  async function handleCurrencyDesign(event: SubmitEvent) {
    event.preventDefault();
    
    // In raw JSON mode, skip to final submission
    if (isAdvancedMode && formData.rawJsonMode) {
      if (!formData.rawJsonInput.trim()) {
        error = 'Raw JSON input is required';
        return;
      }
      
      try {
        JSON.parse(formData.rawJsonInput);
      } catch (e) {
        error = 'Invalid JSON format';
        return;
      }
      
      await handleFinalSubmit();
      return;
    }
    
    if (!formData.name) {
      error = 'Currency name is required';
      return;
    }

    // In regular mode, validate the name is available as an identity
    if (!isAdvancedMode && !availableIdentities.includes(formData.name)) {
      error = `Identity "${formData.name}" not found or not controllable. You need a VerusID with this name.`;
      return;
    }

    // In advanced mode, go directly to review if not using guided mode
    if (isAdvancedMode && !formData.rawJsonMode) {
      currentStep = 'review-and-confirm';
    } else {
      currentStep = 'launch-parameters';
    }
    error = null;
  }

  async function handleLaunchParameters(event: SubmitEvent) {
    event.preventDefault();
    
    // Validate basket currency requirements
    if (formData.currencyType === 'basket') {
      if (formData.initialSupply <= 0) {
        error = 'Initial supply is required for basket currencies';
        return;
      }
      
      if (formData.currencies.some(c => !c.trim())) {
        error = 'All reserve currencies must be specified';
        return;
      }

      // Validate weights sum to 1.0
      const weightSum = formData.weights.reduce((sum, w) => sum + w, 0);
      if (Math.abs(weightSum - 1.0) > 0.001) {
        error = 'Reserve currency weights must sum to 1.0';
        return;
      }
    }

    currentStep = 'review-and-confirm';
    error = null;
  }

  async function handleFinalSubmit() {
    isSubmitting = true;
    error = null;
    currentStep = 'processing';

    try {
      // Build currency definition
      let currencyDefinition: any;
      
      if (isAdvancedMode && formData.rawJsonMode) {
        // Use raw JSON input
        currencyDefinition = JSON.parse(formData.rawJsonInput);
      } else {
        // Build from form data
        currencyDefinition = {
          name: formData.name,
          options: calculateOptions(),
          proofprotocol: formData.proofProtocol,
          notarizationprotocol: isAdvancedMode ? formData.notarizationProtocol : 1,
          currencies: formData.currencyType === 'basket' ? formData.currencies.filter(c => c.trim()) : [],
          weights: formData.currencyType === 'basket' ? formData.weights : [],
          conversions: formData.conversions.length > 0 ? formData.conversions : [],
          minpreconversion: formData.minPreconversion.length > 0 ? formData.minPreconversion : undefined,
          maxpreconversion: formData.maxPreconversion.length > 0 ? formData.maxPreconversion : undefined,
          initialcontributions: formData.initialContributions.length > 0 ? formData.initialContributions : undefined,
          initialsupply: formData.currencyType === 'basket' ? formData.initialSupply : 0,
          startblock: formData.startBlock ? parseInt(formData.startBlock) : undefined,
          endblock: formData.endBlock ? parseInt(formData.endBlock) : undefined,
          idregistrationfees: formData.idRegistrationFees,
          idreferrallevels: formData.idReferralLevels,
          // Only include idimportfees if user wants to price subIDs in reserve currency
          idimportfees: (formData.currencyType === 'basket' && formData.useReservePricingForSubIDs) 
            ? formData.idImportFeesReserveIndex / 100000000 
            : undefined
        };

        // Add advanced parameters if in advanced mode
        if (isAdvancedMode) {
          if (formData.notarizationReward) currencyDefinition.notarizationreward = parseFloat(formData.notarizationReward);
          if (formData.minNotariesConfirm) currencyDefinition.minnotariesconfirm = parseInt(formData.minNotariesConfirm);
          if (formData.blockTime) currencyDefinition.blocktime = parseInt(formData.blockTime);
          if (formData.powAveragingWindow) currencyDefinition.powaveragingwindow = parseInt(formData.powAveragingWindow);
          if (formData.notarizationPeriod) currencyDefinition.notarizationperiod = parseInt(formData.notarizationPeriod);
          if (formData.gatewayConverterName) currencyDefinition.gatewayconvertername = formData.gatewayConverterName;
          if (formData.systemId) currencyDefinition.systemid = formData.systemId;
          if (formData.parent) currencyDefinition.parent = formData.parent;
          if (formData.launchSystemId) currencyDefinition.launchsystemid = formData.launchSystemId;
          if (formData.expiryHeight) currencyDefinition.expiryheight = parseInt(formData.expiryHeight);
          if (formData.premine) currencyDefinition.premine = parseInt(formData.premine);
          if (formData.notaries.length > 0) currencyDefinition.notaries = formData.notaries;
          if (formData.reserves.length > 0) currencyDefinition.reserves = formData.reserves;
          if (formData.nodes.length > 0) currencyDefinition.nodes = formData.nodes;
          if (formData.eras.length > 0) currencyDefinition.eras = formData.eras;
        }
      };

      // Add preallocations if specified
      if (formData.preallocations.length > 0) {
        const preallocations = formData.preallocations
          .filter(p => p.identity.trim() && p.amount > 0)
          .reduce((acc, p) => {
            acc[p.identity] = p.amount;
            return acc;
          }, {} as Record<string, number>);
        
        if (Object.keys(preallocations).length > 0) {
          (currencyDefinition as any).preallocations = [preallocations];
        }
      }

      // Step 1: Define currency
      const defineResult = await invoke('define_currency', {
        currencyDefinition,
        fractionalGateway: null,
        reserves: null
      });

      console.log('Define currency result:', defineResult);

      if (!defineResult || !(defineResult as any).hex) {
        throw new Error('Failed to get transaction hex from define currency');
      }

      // Step 2: Send raw transaction
      const sendResult = await invoke('send_raw_transaction', {
        hexData: (defineResult as any).hex
      });

      console.log('Send raw transaction result:', sendResult);
      transactionHash = sendResult as string;
      
      // Show success toast
      showSuccess('Success: currency created', { txid: transactionHash });
      
      currentStep = 'waiting-blocks';
      
      // Auto-complete after demonstrating the process
      setTimeout(() => {
        if (onSuccess) {
          onSuccess();
        }
        onClose();
      }, 5000);

    } catch (err) {
      console.error('Currency creation failed:', err);
      error = typeof err === 'string' ? err : 'Currency creation failed. Please try again.';
      showError(error);
      currentStep = 'review-and-confirm';
    } finally {
      isSubmitting = false;
    }
  }

  function goBack() {
    if (currentStep === 'launch-parameters') {
      currentStep = 'currency-design';
    } else if (currentStep === 'review-and-confirm') {
      currentStep = 'launch-parameters';
    }
    error = null;
  }

  // Get step info
  let stepInfo = $derived(() => {
    switch (currentStep) {
      case 'currency-design':
        return {
          title: 'Step 1: Currency Design',
          description: 'Define the basic properties and type of your currency',
          icon: '🎨'
        };
      case 'launch-parameters':
        return {
          title: 'Step 2: Launch Parameters',
          description: 'Configure launch settings, supply, and distribution',
          icon: '🚀'
        };
      case 'review-and-confirm':
        return {
          title: 'Step 3: Review & Confirm',
          description: 'Review your currency definition before launching',
          icon: '✅'
        };
      case 'processing':
        return {
          title: 'Processing',
          description: 'Creating currency definition and broadcasting to network',
          icon: '⚙️'
        };
      case 'waiting-blocks':
        return {
          title: 'Currency Launched',
          description: 'Your currency has been submitted and will be active in ~20 blocks',
          icon: '🎉'
        };
      default:
        return { title: '', description: '', icon: '' };
    }
  });

  let progressPercentage = $derived(() => {
    switch (currentStep) {
      case 'currency-design': return 25;
      case 'launch-parameters': return 50;
      case 'review-and-confirm': return 75;
      case 'processing': return 85;
      case 'waiting-blocks': return 100;
      default: return 0;
    }
  });
</script>

<Modal {isOpen} onclose={onClose} title="Create New Currency" size="xl">
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
            class="h-full bg-verusidx-forest-deep transition-all duration-300"
            style="width: {progressPercentage}%"
          ></div>
        </div>
        <span class="ml-3 text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
          {progressPercentage}%
        </span>
      </div>
    </div>

    <!-- Error Display -->
    {#if error}
      <div class="mb-4 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
        <p class="text-red-700 dark:text-red-300">{error}</p>
      </div>
    {/if}

    <!-- Step 1: Currency Design -->
    {#if currentStep === 'currency-design'}
      <form onsubmit={handleCurrencyDesign} class="space-y-4">
        <!-- Currency Name -->
        <div>
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
            Currency Name *
          </label>
          <select 
            bind:value={formData.name}
            required
            disabled={isLoadingData}
            class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
          >
            <option value="">{isLoadingData ? 'Loading identities...' : 'Select an identity you control...'}</option>
            {#each availableIdentities as identity}
              <option value={identity}>{identity}@</option>
            {/each}
          </select>
          <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
            Your currency will be named: {formData.name || 'YourCurrency'}
          </p>
        </div>

        <!-- Currency Type -->
        <div>
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
            Currency Type *
          </label>
          <div class="space-y-2">
            <label class="flex items-center space-x-2">
              <input type="radio" bind:group={formData.currencyType} value="simple" class="text-verusidx-forest-deep" />
              <span class="text-verusidx-stone-dark dark:text-white">Simple Token Currency</span>
            </label>
            <label class="flex items-center space-x-2">
              <input type="radio" bind:group={formData.currencyType} value="basket" class="text-verusidx-forest-deep" />
              <span class="text-verusidx-stone-dark dark:text-white">Basket Currency (with reserves)</span>
            </label>
          </div>
        </div>

        <!-- Proof Protocol -->
        <div>
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
            Control Type *
          </label>
          <select 
            bind:value={formData.proofProtocol}
            class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
          >
            <option value={1}>Decentralized (no minting/burning after launch)</option>
            <option value={2}>Centralized (can mint/burn via root identity)</option>
          </select>
        </div>

        <!-- ID Registration Fees -->
        <div>
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
            Sub-Identity Registration Fee
          </label>
          <input 
            type="number" 
            bind:value={formData.idRegistrationFees}
            min="0"
            step="0.00000001"
            class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
          />
          <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
            Cost in your currency to register sub-identities
          </p>
        </div>

        <!-- ID Referral Levels -->
        <div>
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
            Referral Levels (0-5)
          </label>
          <input 
            type="number" 
            bind:value={formData.idReferralLevels}
            min="0"
            max="5"
            class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
          />
          <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
            Number of referral levels for sub-identity registration rewards
          </p>
        </div>

        <!-- Advanced Mode Toggle -->
        <div class="border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium pt-4">
          <label class="flex items-center space-x-2">
            <input 
              type="checkbox" 
              bind:checked={isAdvancedMode}
              class="text-verusidx-forest-deep"
            />
            <span class="text-sm font-medium text-verusidx-stone-dark dark:text-white">
              Advanced Mode (Full Parameter Control)
            </span>
          </label>
          <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
            Expose all definecurrency parameters for manual configuration
          </p>
        </div>

        <!-- Advanced Parameters (only in advanced mode) -->
        {#if isAdvancedMode}
          <div class="border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium pt-4">
            <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-4">Advanced Parameters</h3>
            
            <!-- Raw JSON Mode Toggle -->
            <div class="mb-4">
              <label class="flex items-center space-x-2">
                <input 
                  type="checkbox" 
                  bind:checked={formData.rawJsonMode}
                  class="text-verusidx-forest-deep"
                />
                <span class="text-sm font-medium text-verusidx-stone-dark dark:text-white">
                  Raw JSON Mode
                </span>
              </label>
              <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
                Input the complete currency definition as JSON
              </p>
            </div>

            {#if formData.rawJsonMode}
              <!-- Raw JSON Input -->
              <div class="mb-4">
                <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
                  Currency Definition JSON
                </label>
                <textarea 
                  bind:value={formData.rawJsonInput}
                  rows="20"
                  placeholder={`{
  "name": "MyCurrency",
  "options": 32,
  "proofprotocol": 1,
  "notarizationprotocol": 1,
  "currencies": [],
  "weights": [],
  "conversions": [],
  "initialsupply": 0,
  "idregistrationfees": 100,
  "idreferrallevels": 3
}`}
                  class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white font-mono text-sm"
                ></textarea>
                <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
                  Enter a complete JSON object for the currency definition. This will override all form fields.
                </p>
              </div>
            {:else}
              <!-- Individual Advanced Parameters -->
              <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <!-- Notarization Protocol -->
                <div>
                  <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
                    Notarization Protocol
                  </label>
                  <select 
                    bind:value={formData.notarizationProtocol}
                    class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
                  >
                    <option value={1}>1 - PROOF_PBAASMMR (Verus MMR proof)</option>
                    <option value={2}>2 - PROOF_CHAINID (chain ID notary)</option>
                    <option value={3}>3 - PROOF_ETHNOTARIZATION (Ethereum proof)</option>
                  </select>
                </div>

                <!-- Notarization Reward -->
                <div>
                  <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
                    Notarization Reward
                  </label>
                  <input 
                    type="number" 
                    step="any"
                    bind:value={formData.notarizationReward}
                    placeholder="Default reward"
                    class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
                  />
                </div>

                <!-- Min Notaries Confirm -->
                <div>
                  <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
                    Min Notaries Confirm
                  </label>
                  <input 
                    type="number"
                    bind:value={formData.minNotariesConfirm}
                    placeholder="Minimum notary signatures"
                    class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
                  />
                </div>

                <!-- Block Time -->
                <div>
                  <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
                    Block Time (seconds)
                  </label>
                  <input 
                    type="number"
                    bind:value={formData.blockTime}
                    placeholder="60 (default)"
                    class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
                  />
                </div>

                <!-- POW Averaging Window -->
                <div>
                  <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
                    POW Averaging Window
                  </label>
                  <input 
                    type="number"
                    bind:value={formData.powAveragingWindow}
                    placeholder="45 (default blocks)"
                    class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
                  />
                </div>

                <!-- Notarization Period -->
                <div>
                  <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
                    Notarization Period
                  </label>
                  <input 
                    type="number"
                    bind:value={formData.notarizationPeriod}
                    placeholder="10 (default minutes)"
                    class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
                  />
                </div>

                <!-- Gateway Converter Name -->
                <div>
                  <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
                    Gateway Converter Name
                  </label>
                  <input 
                    type="text"
                    bind:value={formData.gatewayConverterName}
                    placeholder="For PBaaS chains"
                    class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
                  />
                </div>

                <!-- System ID -->
                <div>
                  <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
                    System ID
                  </label>
                  <input 
                    type="text"
                    bind:value={formData.systemId}
                    placeholder="System identifier"
                    class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
                  />
                </div>

                <!-- Parent -->
                <div>
                  <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
                    Parent
                  </label>
                  <input 
                    type="text"
                    bind:value={formData.parent}
                    placeholder="Parent chain identifier"
                    class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
                  />
                </div>

                <!-- Launch System ID -->
                <div>
                  <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
                    Launch System ID
                  </label>
                  <input 
                    type="text"
                    bind:value={formData.launchSystemId}
                    placeholder="Launch system identifier"
                    class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
                  />
                </div>

                <!-- Expiry Height -->
                <div>
                  <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
                    Expiry Height
                  </label>
                  <input 
                    type="number"
                    bind:value={formData.expiryHeight}
                    placeholder="Block height expiry"
                    class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
                  />
                </div>

                <!-- Premine -->
                <div>
                  <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
                    Premine
                  </label>
                  <input 
                    type="number"
                    step="any"
                    bind:value={formData.premine}
                    placeholder="Premined amount"
                    class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
                  />
                </div>
              </div>

              <!-- Warning about advanced mode -->
              <div class="mt-4 p-4 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg">
                <p class="text-yellow-700 dark:text-yellow-300 text-sm">
                  ⚠️ <strong>Advanced Mode:</strong> These parameters directly map to the Verus definecurrency RPC command. 
                  Incorrect values may cause currency creation to fail. Refer to the Verus documentation for parameter details.
                </p>
              </div>
            {/if}
          </div>
        {/if}

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
            class="flex-1 px-6 py-3 bg-verusidx-forest-deep hover:bg-verusidx-turquoise-deep text-white rounded-lg transition-colors"
          >
            {isAdvancedMode && formData.rawJsonMode ? 'Create Currency' : 'Next: Launch Parameters'}
          </button>
        </div>
      </form>

    <!-- Step 2: Launch Parameters -->
    {:else if currentStep === 'launch-parameters'}
      <form onsubmit={handleLaunchParameters} class="space-y-4">
        <!-- Basket Currency Specific -->
        {#if formData.currencyType === 'basket'}
          <!-- Initial Supply -->
          <div>
            <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
              Initial Supply *
            </label>
            <input 
              type="number" 
              bind:value={formData.initialSupply}
              min="0"
              step="0.00000001"
              required
              class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
            />
            <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
              Initial supply distributed among preconverters
            </p>
          </div>

          <!-- Reserve Currencies -->
          <div>
            <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
              Reserve Currencies *
            </label>
            {#each formData.currencies as _, index}
              <div class="flex space-x-2 mb-2">
                <div class="flex-1">
                  <select 
                    bind:value={formData.currencies[index]}
                    required
                    class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
                  >
                    <option value="">Select currency...</option>
                    {#each availableCurrencies as availableCurrency}
                      <option value={availableCurrency}>{availableCurrency}</option>
                    {/each}
                  </select>
                </div>
                <div class="w-24">
                  <input 
                    type="number" 
                    bind:value={formData.weights[index]}
                    min="0.1"
                    max="1"
                    step="0.1"
                    placeholder="Weight"
                    class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
                  />
                </div>
                {#if formData.currencies.length > 1}
                  <button
                    type="button"
                    onclick={() => removeReserveCurrency(index)}
                    class="px-3 py-2 text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
                  >
                    Remove
                  </button>
                {/if}
              </div>
            {/each}
            {#if formData.currencies.length < 10}
              <button
                type="button"
                onclick={addReserveCurrency}
                class="text-sm text-verusidx-turquoise-deep hover:text-verusidx-turquoise-bright"
              >
                + Add another reserve currency
              </button>
            {/if}
            <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
              Weights must sum to 1.0. Current sum: {formData.weights.reduce((a, b) => a + b, 0).toFixed(1)}
            </p>
          </div>

          <!-- Sub-ID Pricing Option -->
          <div>
            <label class="flex items-center space-x-2 mb-2">
              <input 
                type="checkbox" 
                bind:checked={formData.useReservePricingForSubIDs}
                class="text-verusidx-forest-deep"
              />
              <span class="text-sm font-medium text-verusidx-stone-dark dark:text-white">
                Price sub-identities in reserve currency
              </span>
            </label>
            
            {#if formData.useReservePricingForSubIDs}
              <div class="mt-2">
                <label class="block text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-1">
                  Reserve currency for sub-ID pricing:
                </label>
                <select 
                  bind:value={formData.idImportFeesReserveIndex}
                  class="w-full p-2 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white text-sm"
                >
                  {#each formData.currencies as curr, index}
                    {#if curr.trim()}
                      <option value={index}>
                        {curr} (index {index})
                      </option>
                    {/if}
                  {/each}
                </select>
                <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
                  Sub-ID fees will be priced in {formData.currencies[formData.idImportFeesReserveIndex] || 'selected currency'} but paid in the basket currency
                </p>
              </div>
            {/if}
          </div>
        {/if}

        <!-- Launch Timing -->
        <div>
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
            Start Block (Optional)
          </label>
          <input 
            type="number" 
            bind:value={formData.startBlock}
            placeholder="Leave empty for automatic (current + 20 blocks)"
            class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
          />
        </div>

        <!-- Preallocations -->
        <div>
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
            Preallocations (Optional)
          </label>
          {#each formData.preallocations as preallocation, index}
            <div class="flex space-x-2 mb-2">
              <input 
                type="text" 
                bind:value={preallocation.identity}
                placeholder="Identity name (e.g., alice@)"
                class="flex-1 p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
              />
              <input 
                type="number" 
                bind:value={preallocation.amount}
                min="0"
                step="0.00000001"
                placeholder="Amount"
                class="w-32 p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
              />
              <button
                type="button"
                onclick={() => removePreallocation(index)}
                class="px-3 py-2 text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
              >
                Remove
              </button>
            </div>
          {/each}
          <button
            type="button"
            onclick={addPreallocation}
            class="text-sm text-verusidx-turquoise-deep hover:text-verusidx-turquoise-bright"
          >
            + Add preallocation
          </button>
          <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
            Currency amounts allocated to specific identities at launch
          </p>
        </div>

        <!-- Buttons -->
        <div class="flex space-x-4 pt-6">
          <button
            type="button"
            onclick={goBack}
            class="px-6 py-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium text-verusidx-stone-dark dark:text-white rounded-lg hover:bg-verusidx-mountain-mist dark:hover:bg-verusidx-stone-medium transition-colors"
          >
            Back
          </button>
          <button
            type="submit"
            class="flex-1 px-6 py-3 bg-verusidx-forest-deep hover:bg-verusidx-turquoise-deep text-white rounded-lg transition-colors"
          >
            Next: Review & Confirm
          </button>
        </div>
      </form>

    <!-- Step 3: Review & Confirm -->
    {:else if currentStep === 'review-and-confirm'}
      <div class="space-y-6">
        <!-- Currency Overview -->
        <div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium rounded-lg p-4">
          <h4 class="font-medium text-verusidx-stone-dark dark:text-white mb-3">Currency Overview</h4>
          <div class="space-y-2 text-sm">
            <div class="flex justify-between">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Name:</span>
              <span class="text-verusidx-stone-dark dark:text-white">{formData.name}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Type:</span>
              <span class="text-verusidx-stone-dark dark:text-white">{formData.currencyType === 'basket' ? 'Basket Currency' : 'Simple Token'}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Control:</span>
              <span class="text-verusidx-stone-dark dark:text-white">{formData.proofProtocol === 1 ? 'Decentralized' : 'Centralized'}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Options Value:</span>
              <span class="text-verusidx-stone-dark dark:text-white">{calculateOptions()}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Sub-ID Registration Fee:</span>
              <span class="text-verusidx-stone-dark dark:text-white">{formData.idRegistrationFees} {formData.name}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Referral Levels:</span>
              <span class="text-verusidx-stone-dark dark:text-white">{formData.idReferralLevels}</span>
            </div>
            {#if formData.currencyType === 'basket' && formData.useReservePricingForSubIDs}
              <div class="flex justify-between">
                <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Sub-ID Pricing Currency:</span>
                <span class="text-verusidx-stone-dark dark:text-white">{formData.currencies[formData.idImportFeesReserveIndex]} (0.{formData.idImportFeesReserveIndex.toString().padStart(8, '0')})</span>
              </div>
            {/if}
          </div>
        </div>

        {#if formData.currencyType === 'basket'}
          <!-- Reserve Details -->
          <div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium rounded-lg p-4">
            <h4 class="font-medium text-verusidx-stone-dark dark:text-white mb-3">Reserve Currencies</h4>
            <div class="space-y-1 text-sm">
              {#each formData.currencies as curr, index}
                {#if curr.trim()}
                  <div class="flex justify-between">
                    <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">{curr}:</span>
                    <span class="text-verusidx-stone-dark dark:text-white">{(formData.weights[index] * 100).toFixed(1)}%</span>
                  </div>
                {/if}
              {/each}
            </div>
          </div>
        {/if}

        {#if formData.preallocations.length > 0}
          <!-- Preallocations -->
          <div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium rounded-lg p-4">
            <h4 class="font-medium text-verusidx-stone-dark dark:text-white mb-3">Preallocations</h4>
            <div class="space-y-1 text-sm">
              {#each formData.preallocations as preallocation}
                {#if preallocation.identity.trim() && preallocation.amount > 0}
                  <div class="flex justify-between">
                    <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">{preallocation.identity}:</span>
                    <span class="text-verusidx-stone-dark dark:text-white">{preallocation.amount}</span>
                  </div>
                {/if}
              {/each}
            </div>
          </div>
        {/if}

        <!-- Cost Warning -->
        <div class="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
          <p class="text-yellow-700 dark:text-yellow-300 text-sm">
            ⚠️ <strong>Cost:</strong> Ensure your identity "{formData.name}@" has sufficient funds before proceeding.
          </p>
        </div>

        <!-- Buttons -->
        <div class="flex space-x-4 pt-6">
          <button
            type="button"
            onclick={goBack}
            class="px-6 py-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium text-verusidx-stone-dark dark:text-white rounded-lg hover:bg-verusidx-mountain-mist dark:hover:bg-verusidx-stone-medium transition-colors"
          >
            Back
          </button>
          <button
            type="button"
            onclick={handleFinalSubmit}
            disabled={isSubmitting}
            class="flex-1 px-6 py-3 bg-verusidx-forest-deep hover:bg-verusidx-turquoise-deep text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isSubmitting ? 'Creating Currency...' : 'Launch Currency'}
          </button>
        </div>
      </div>

    <!-- Step 4: Processing -->
    {:else if currentStep === 'processing'}
      <div class="text-center py-8">
        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-verusidx-forest-deep mx-auto mb-4"></div>
        <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-2">
          Creating Currency Definition
        </h3>
        <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
          Please wait while we prepare and broadcast your currency to the network...
        </p>
      </div>

    <!-- Step 5: Success -->
    {:else if currentStep === 'waiting-blocks'}
      <div class="text-center py-8">
        <div class="text-green-600 dark:text-green-400 mb-4">
          <svg class="h-16 w-16 mx-auto" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
        </div>
        <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-2">
          Currency Successfully Launched!
        </h3>
        <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-4">
          Your currency "{formData.name}" has been submitted to the network and will be active in approximately 20 blocks.
        </p>
        
        {#if transactionHash}
          <div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium rounded-lg p-4 mb-4">
            <p class="text-sm font-medium text-verusidx-stone-dark dark:text-white">Transaction Hash:</p>
            <p class="text-xs font-mono text-verusidx-mountain-grey dark:text-verusidx-mountain-mist break-all">
              {transactionHash}
            </p>
          </div>
        {/if}

        <button
          onclick={onClose}
          class="px-6 py-3 bg-verusidx-forest-deep hover:bg-verusidx-turquoise-deep text-white rounded-lg transition-colors"
        >
          Done
        </button>
      </div>
    {/if}
  </div>
</Modal>