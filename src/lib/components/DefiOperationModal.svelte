<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Modal } from './cards';
  import { connectionStore, getChainParam } from "$lib/stores/connection";
  import { showSuccess, showError } from "$lib/services/notifications";

  // Props
  type OperationType = 
    | 'send-transparent'
    | 'send-private' 
    | 'convert-reserve'
    | 'convert-reserve-to-basket'
    | 'convert-basket-to-reserve'
    | 'preconvert'
    | 'export-definition'
    | 'send-crosschain'
    | 'convert-crosschain'
    | 'export-identity'
    | 'advanced'
    | 'estimate-conversion';

  interface Props {
    isOpen: boolean;
    operationType: OperationType | null;
    onClose: () => void;
  }

  let { isOpen = false, operationType = null, onClose }: Props = $props();

  // Operation configurations
  const operationConfigs = {
    'send-transparent': {
      title: 'Send from Transparent Address',
      description: 'Send currency from an R-address',
      icon: '💸'
    },
    'send-private': {
      title: 'Send from Private Address', 
      description: 'Send currency from a z-address with optional memo',
      icon: '🔒'
    },
    'convert-reserve': {
      title: 'Convert Reserve to Reserve',
      description: 'Convert between reserve currencies using basket via',
      icon: '🔄'
    },
    'convert-reserve-to-basket': {
      title: 'Convert Reserve to Basket',
      description: 'Convert reserve currency to basket currency',
      icon: '🔄'
    },
    'convert-basket-to-reserve': {
      title: 'Convert Basket to Reserve',
      description: 'Convert basket currency to reserve currency',
      icon: '🔄'
    },
    'preconvert': {
      title: 'Preconvert to Launch',
      description: 'Convert during currency launch period',
      icon: '🚀'
    },
    'export-definition': {
      title: 'Export Currency Definition',
      description: 'Export currency definition to another chain',
      icon: '📋'
    },
    'send-crosschain': {
      title: 'Send Cross-Chain',
      description: 'Send currency to another chain',
      icon: '🌉'
    },
    'convert-crosschain': {
      title: 'Convert Cross-Chain',
      description: 'Convert and export to another chain',
      icon: '🌉'
    },
    'export-identity': {
      title: 'Export Identity',
      description: 'Export identity to another chain',
      icon: '👤'
    },
    'advanced': {
      title: 'Advanced Mode',
      description: 'Full sendcurrency interface with all parameters',
      icon: '🔧'
    },
    'estimate-conversion': {
      title: 'Estimate Conversion',
      description: 'Calculate expected conversion output',
      icon: '🧮'
    }
  };

  // Get current operation config
  let currentConfig = $derived(operationType ? operationConfigs[operationType] : null);

  // Form state
  let formData = $state({
    fromAddress: '',
    toAddress: '',
    currency: '',
    amount: '',
    memo: '',
    convertTo: '',
    via: '',
    exportTo: '',
    exportCurrency: false,
    exportId: false,
    preconvert: false,
    minConfs: '',
    feeAmount: '',
    feeCurrency: '',
    addConversionFees: false,
    burn: false,
    mintNew: false,
    refundTo: ''
  });

  // Available addresses (will be populated from RPC)
  let availableAddresses = $state<string[]>([]);
  let currencies = $state<{name: string, balance: number}[]>([]);
  let viaCurrencies = $state<string[]>([]);
  let convertToCurrencies = $state<(string | {id: string, name: string, display: string})[]>([]);
  let chains = $state<string[]>(['vrsc', 'vdex', 'varrr', 'chips']);

  // Loading states
  let isSubmitting = $state(false);
  let isLoadingAddresses = $state(false);
  let isLoadingCurrencies = $state(false);
  let isLoadingVia = $state(false);
  let isLoadingConvertTo = $state(false);
  let hasLoadedAddresses = $state(false);
  let hasLoadedCurrencies = $state(false);
  let hasLoadedVia = $state(false);
  let hasLoadedConvertTo = $state(false);
  let lastLoadedOperation = $state<string | null>(null);
  let lastLoadedAddress = $state<string | null>(null);
  let lastLoadedCurrency = $state<string | null>(null);
  let lastLoadedConvertToKey = $state<string | null>(null);
  let error = $state<string | null>(null);
  let estimateResult = $state<any | null>(null);
  let isEstimating = $state(false);
  
  // Connection state for chain parameters
  let connectionState = $state<any>(null);
  
  connectionStore.subscribe(state => {
    connectionState = state;
  });


  // Reset form when modal closes or operation changes
  $effect(() => {
    console.log('🔍 Reset effect triggered - isOpen:', isOpen, 'operationType:', operationType);
    if (!isOpen || !operationType) {
      console.log('🚨 Triggering form reset due to:', !isOpen ? 'modal closed' : 'no operation type');
      resetForm();
    }
  });

  // Load addresses when modal opens or operation type changes
  $effect(() => {
    if (isOpen && operationType && (lastLoadedOperation !== operationType || !hasLoadedAddresses)) {
      console.log('🔄 Loading addresses for operation:', operationType);
      loadAddresses();
    }
  });

  // Load currencies when fromAddress changes
  $effect(() => {
    if (formData.fromAddress && formData.fromAddress !== '' && 
        (lastLoadedAddress !== formData.fromAddress || !hasLoadedCurrencies)) {
      console.log('💰 Loading currencies for address:', formData.fromAddress);
      loadCurrencies();
    } else if (!formData.fromAddress || formData.fromAddress === '') {
      console.log('💰 Clearing currencies - no fromAddress');
      currencies = [];
      hasLoadedCurrencies = false;
      lastLoadedAddress = null;
    }
  });

  // Load via currencies when currency changes (for reserve→reserve)
  $effect(() => {
    if (formData.currency && operationType === 'convert-reserve' && 
        (lastLoadedCurrency !== formData.currency || !hasLoadedVia)) {
      console.log('🔄 Loading via currencies for:', formData.currency);
      loadViaCurrencies();
    } else if (!formData.currency || operationType !== 'convert-reserve') {
      console.log('🔄 Clearing via currencies');
      viaCurrencies = [];
      hasLoadedVia = false;
      lastLoadedCurrency = null;
    }
  });

  // Load convertTo currencies when dependencies change
  $effect(() => {
    // Create current key to check if we need to reload
    let currentKey = '';
    if (operationType === 'convert-reserve' && formData.via) {
      currentKey = `reserve-${formData.via}`;
    } else if (operationType === 'convert-reserve-to-basket' && formData.currency) {
      currentKey = `basket-${formData.currency}`;
    } else if (operationType === 'convert-basket-to-reserve' && formData.currency) {
      currentKey = `basket-to-reserve-${formData.currency}`;
    } else if (operationType === 'preconvert') {
      currentKey = 'preconvert';
    }
    
    if (currentKey && (lastLoadedConvertToKey !== currentKey || !hasLoadedConvertTo)) {
      console.log('🎯 Loading convertTo currencies for key:', currentKey);
      loadConvertToCurrencies();
    } else if (!currentKey) {
      console.log('🎯 Clearing convertTo currencies');
      convertToCurrencies = [];
      hasLoadedConvertTo = false;
      lastLoadedConvertToKey = null;
    }
  });

  // Auto-populate toAddress for export-identity operations
  $effect(() => {
    if (operationType === 'export-identity' && formData.fromAddress) {
      formData.toAddress = formData.fromAddress;
    }
  });

  function getSuccessMessage(type: OperationType): string {
    switch(type) {
      case 'send-transparent':
      case 'send-private':
        return 'Success: currency sent';
      case 'send-crosschain':
        return 'Success: cross-chain transfer initiated';
      case 'convert-reserve-to-basket':
      case 'convert-basket-to-reserve':
      case 'convert-reserve':
        return 'Success: currency converted';
      case 'convert-crosschain':
        return 'Success: cross-chain conversion initiated';
      case 'preconvert':
        return 'Success: preconversion submitted';
      case 'export-identity':
        return 'Success: identity exported';
      case 'export-definition':
        return 'Success: currency definition exported';
      case 'advanced':
        return 'Success: operation submitted';
      default:
        return 'Success: operation completed';
    }
  }

  function resetForm() {
    console.log('🔄 Resetting form');
    console.trace('📍 Reset form called from:');
    formData = {
      fromAddress: '',
      toAddress: '',
      currency: '',
      amount: '',
      memo: '',
      convertTo: '',
      via: '',
      exportTo: '',
      exportCurrency: false,
      exportId: false,
      preconvert: false,
      minConfs: '',
      feeAmount: '',
      feeCurrency: '',
      addConversionFees: false,
      burn: false,
      mintNew: false,
      refundTo: ''
    };
    error = null;
    // Reset loading states when form resets
    hasLoadedAddresses = false;
    hasLoadedCurrencies = false;
    hasLoadedVia = false;
    hasLoadedConvertTo = false;
    lastLoadedOperation = null;
    lastLoadedAddress = null;
    lastLoadedCurrency = null;
    lastLoadedConvertToKey = null;
    availableAddresses = [];
    currencies = [];
    viaCurrencies = [];
    convertToCurrencies = [];
    estimateResult = null;
  }

  function fillSelfAddress(field: 'toAddress' | 'refundTo') {
    if (formData.fromAddress) {
      formData[field] = formData.fromAddress;
    }
  }

  async function loadAddresses() {
    if (isLoadingAddresses || !operationType) {
      console.log('⏸️ Skipping loadAddresses - already loading or no operation type');
      return;
    }
    
    console.log('🚀 Starting loadAddresses for:', operationType);
    isLoadingAddresses = true;
    hasLoadedAddresses = false;
    
    try {
      const allAddresses = [];

      // Determine which addresses to load based on operation type
      const needsTransparent = ['send-transparent', 'convert-reserve', 'convert-reserve-to-basket', 'convert-basket-to-reserve', 'preconvert', 'send-crosschain', 'convert-crosschain', 'export-definition'].includes(operationType);
      const needsPrivate = ['send-private', 'convert-reserve', 'convert-reserve-to-basket', 'convert-basket-to-reserve', 'preconvert', 'send-crosschain', 'convert-crosschain', 'export-definition'].includes(operationType);

      console.log('📋 Address loading plan:', { needsTransparent, needsPrivate });

      if (needsTransparent) {
        try {
          console.log('📞 Calling list_address_groupings...');
          const transparentResult = await invoke('list_address_groupings');
          console.log('📝 Transparent result structure:', JSON.stringify(transparentResult, null, 2));
          
          // listaddressgroupings returns: [[{address, amount, account}, ...], ...]
          // We want to extract addresses from these object structures
          if (Array.isArray(transparentResult)) {
            const addresses = [];
            
            // Extract addresses from nested groups of objects
            for (const group of transparentResult) {
              if (Array.isArray(group)) {
                for (const addressInfo of group) {
                  if (addressInfo && typeof addressInfo === 'object' && addressInfo.address && typeof addressInfo.address === 'string') {
                    addresses.push(addressInfo.address); // Extract the address property
                    console.log('📍 Found address:', addressInfo.address, 'with amount:', addressInfo.amount);
                  }
                }
              }
            }
            
            console.log('✅ Parsed transparent addresses:', addresses);
            allAddresses.push(...addresses);
          }
        } catch (err) {
          console.error('❌ Failed to load transparent addresses:', err);
        }
      }

      if (needsPrivate) {
        try {
          console.log('📞 Calling z_list_addresses...');
          const privateResult = await invoke('z_list_addresses');
          console.log('📝 Private result:', privateResult);
          
          if (Array.isArray(privateResult)) {
            console.log('✅ Parsed private addresses:', privateResult);
            allAddresses.push(...privateResult);
          }
        } catch (err) {
          console.error('❌ Failed to load private addresses:', err);
        }
      }

      // Update state only once at the end
      console.log('🎯 Final addresses to set:', allAddresses);
      availableAddresses = allAddresses;
      hasLoadedAddresses = true;
      lastLoadedOperation = operationType;
      console.log('✅ Address loading completed successfully');

    } catch (err) {
      console.error('💥 Critical error in loadAddresses:', err);
      hasLoadedAddresses = false;
    } finally {
      isLoadingAddresses = false;
      console.log('🏁 loadAddresses finally block - isLoadingAddresses set to false');
    }
  }

  async function loadCurrencies() {
    if (isLoadingCurrencies || !formData.fromAddress || !operationType) {
      console.log('⏸️ Skipping loadCurrencies - already loading or missing requirements');
      return;
    }
    
    // For advanced mode or operations that need manual input, don't auto-load
    if (operationType === 'advanced' || operationType === 'export-definition') {
      console.log('⏸️ Skipping loadCurrencies for advanced/export-definition');
      return;
    }
    
    console.log('🚀 Starting loadCurrencies for address:', formData.fromAddress);
    isLoadingCurrencies = true;
    hasLoadedCurrencies = false;
    
    try {
      const result = await invoke('get_currency_balance', { 
        address: formData.fromAddress
      });
      console.log('💰 Currency balance result:', result);
      if (result && typeof result === 'object') {
        const currencyList = Object.entries(result).map(([name, balance]) => ({
          name,
          balance: balance as number
        }));
        console.log('💰 Parsed currencies:', currencyList);
        currencies = currencyList;
        hasLoadedCurrencies = true;
        lastLoadedAddress = formData.fromAddress;
        console.log('✅ Currency loading completed successfully');
      }
    } catch (err) {
      console.error('❌ Failed to load currencies:', err);
      currencies = [];
      hasLoadedCurrencies = false;
    } finally {
      isLoadingCurrencies = false;
      console.log('🏁 loadCurrencies finally block - isLoadingCurrencies set to false');
    }
  }

  async function loadViaCurrencies() {
    if (isLoadingVia || !formData.currency || operationType !== 'convert-reserve') {
      console.log('⏸️ Skipping loadViaCurrencies - loading:', isLoadingVia, 'currency:', formData.currency, 'op:', operationType);
      return;
    }
    
    console.log('🚀 Starting loadViaCurrencies for currency:', formData.currency);
    isLoadingVia = true;
    hasLoadedVia = false;
    
    try {
      console.log('📞 Calling get_currency_converters with:', [formData.currency]);
      const result = await invoke('get_currency_converters', { 
        currencies: [formData.currency] 
      });
      console.log('🔄 Via currencies result:', result);
      
      if (Array.isArray(result)) {
        console.log('✅ Raw via currencies result:', result);
        // Extract currency names from objects
        const currencyNames = result.map(currency => {
          if (typeof currency === 'string') {
            return currency;
          } else if (typeof currency === 'object' && currency.fullyqualifiedname) {
            return currency.fullyqualifiedname;
          } else if (typeof currency === 'object' && currency.name) {
            return currency.name;
          } else {
            console.warn('Unknown currency object structure:', currency);
            return String(currency);
          }
        });
        console.log('✅ Extracted via currency names:', currencyNames);
        viaCurrencies = currencyNames;
        hasLoadedVia = true;
        lastLoadedCurrency = formData.currency;
        console.log('✅ Via currency loading completed successfully');
      } else {
        console.error('❌ Unexpected via currencies response format:', typeof result);
        viaCurrencies = [];
      }
    } catch (err) {
      console.error('❌ Failed to load via currencies:', err);
      viaCurrencies = [];
      hasLoadedVia = false;
    } finally {
      isLoadingVia = false;
      console.log('🏁 loadViaCurrencies finally block - isLoadingVia set to false');
    }
  }

  async function loadConvertToCurrencies() {
    if (isLoadingConvertTo || !operationType) {
      console.log('⏸️ Skipping loadConvertToCurrencies - loading:', isLoadingConvertTo, 'op:', operationType);
      return;
    }
    
    // Create a key to track what we're loading for
    let currentKey = '';
    if (operationType === 'convert-reserve' && formData.via) {
      currentKey = `reserve-${formData.via}`;
    } else if (operationType === 'convert-reserve-to-basket' && formData.currency) {
      currentKey = `basket-${formData.currency}`;
    } else if (operationType === 'convert-basket-to-reserve' && formData.currency) {
      currentKey = `basket-to-reserve-${formData.currency}`;
    } else if (operationType === 'preconvert') {
      currentKey = 'preconvert';
    } else {
      console.log('⏸️ No valid convertTo loading scenario');
      return;
    }
    
    console.log('🚀 Starting loadConvertToCurrencies for key:', currentKey);
    isLoadingConvertTo = true;
    hasLoadedConvertTo = false;
    
    try {
      let result;
      
      if ((operationType === 'convert-reserve' && formData.via) || (operationType === 'convert-basket-to-reserve' && formData.currency)) {
        // For reserve→reserve or basket→reserve, get currency names from the basket currency
        const currencyName = operationType === 'convert-reserve' ? formData.via : formData.currency;
        console.log('📞 Calling get_currency for basket currency:', currencyName);
        result = await invoke('get_currency', { 
          currencyName: currencyName,
          height: null
        });
        console.log('🎯 Basket currency result:', result);
        
        if (result && typeof result === 'object' && 'currencynames' in result) {
          const currencyNamesObj = (result as any).currencynames;
          console.log('✅ Raw currencynames object:', currencyNamesObj);
          
          // Create objects with both i-address and human-readable name
          const currencyOptions = Object.entries(currencyNamesObj).map(([iAddress, friendlyName]) => ({
            id: iAddress,
            name: friendlyName as string,
            display: `${iAddress} (${friendlyName})`
          }));
          
          console.log('✅ Parsed currency options:', currencyOptions);
          convertToCurrencies = currencyOptions;
          hasLoadedConvertTo = true;
        } else {
          console.error('❌ Unexpected currency response format:', result);
          convertToCurrencies = [];
        }
      } else if (operationType === 'convert-reserve-to-basket') {
        // For basket conversions, use getcurrencyconverters
        const currencyParam = formData.currency || 'VRSC';
        console.log('📞 Calling get_currency_converters for basket:', currencyParam);
        result = await invoke('get_currency_converters', { 
          currencies: [currencyParam] 
        });
        console.log('🎯 Basket conversion result:', result);
        
        if (Array.isArray(result)) {
          console.log('✅ Raw basket currencies result:', result);
          // Extract currency names from objects
          const currencyNames = result.map(currency => {
            if (typeof currency === 'string') {
              return currency;
            } else if (typeof currency === 'object' && currency.fullyqualifiedname) {
              return currency.fullyqualifiedname;
            } else if (typeof currency === 'object' && currency.name) {
              return currency.name;
            } else {
              console.warn('Unknown currency object structure:', currency);
              return String(currency);
            }
          });
          console.log('✅ Extracted basket currency names:', currencyNames);
          convertToCurrencies = currencyNames;
          hasLoadedConvertTo = true;
        } else {
          console.error('❌ Unexpected basket response format:', typeof result);
          convertToCurrencies = [];
        }
      } else if (operationType === 'preconvert') {
        // For preconvert, get currencies in prelaunch state
        console.log('📞 Calling list_currencies for preconvert');
        result = await invoke('list_currencies', { 
          query: null,
          verbose: true,
          launchstate: 'prelaunch'
        });
        console.log('🎯 Preconvert currencies result:', result);
        
        if (Array.isArray(result)) {
          console.log('✅ Raw preconvert currencies result:', result);
          // Extract currency names from the nested currencydefinition structure
          const currencyNames = result.map(currency => {
            // Handle the nested structure from listcurrencies with verbose=true
            if (typeof currency === 'string') {
              return currency;
            } else if (typeof currency === 'object' && currency.currencydefinition) {
              // Extract from nested currencydefinition
              const def = currency.currencydefinition;
              return def.name || def.fullyqualifiedname || def.currencyid || '[Unknown]';
            } else if (typeof currency === 'object' && currency.fullyqualifiedname) {
              return currency.fullyqualifiedname;
            } else if (typeof currency === 'object' && currency.name) {
              return currency.name;
            } else {
              console.warn('Unknown currency object structure:', currency);
              return String(currency);
            }
          });
          console.log('✅ Extracted preconvert currency names:', currencyNames);
          convertToCurrencies = currencyNames;
          hasLoadedConvertTo = true;
        } else {
          console.error('❌ Unexpected preconvert response format:', typeof result);
          convertToCurrencies = [];
        }
      }
      
      if (hasLoadedConvertTo) {
        lastLoadedConvertToKey = currentKey;
        console.log('✅ ConvertTo loading completed successfully for key:', currentKey);
      }
    } catch (err) {
      console.error('❌ Failed to load convertTo currencies:', err);
      convertToCurrencies = [];
      // Mark as loaded even on error to prevent infinite retry loop
      hasLoadedConvertTo = true;
      lastLoadedConvertToKey = currentKey;
      // Set error message for user feedback
      error = `Failed to load currencies: ${err}`;
    } finally {
      isLoadingConvertTo = false;
      console.log('🏁 loadConvertToCurrencies finally block - isLoadingConvertTo set to false');
    }
  }

  async function handleEstimate(event: SubmitEvent) {
    event.preventDefault();
    console.log('Estimating conversion:', operationType, formData);
    
    // Basic validation for estimate
    if (!formData.currency || !formData.amount || !formData.convertTo) {
      error = 'Please fill in currency, amount, and convert to fields';
      return;
    }

    // Amount validation
    if (parseFloat(formData.amount) <= 0) {
      error = 'Please enter a valid amount';
      return;
    }

    isEstimating = true;
    error = null;
    estimateResult = null;

    try {
      console.log('Calling estimate_conversion with:', {
        currency: formData.currency,
        amount: parseFloat(formData.amount),
        convertto: formData.convertTo,
        via: formData.via || null
      });
      
      const chainParam = getChainParam(connectionState?.selectedChain);
      const result = await invoke('estimate_conversion', {
        currency: formData.currency,
        amount: parseFloat(formData.amount),
        convertto: formData.convertTo,
        via: formData.via || null,
        chain: chainParam
      });
      
      console.log('Estimate conversion result:', result);
      estimateResult = result;
      
    } catch (err) {
      error = typeof err === 'string' ? err : 'Estimation failed';
      console.error('Estimation error:', err);
    } finally {
      isEstimating = false;
    }
  }

  async function handleSubmit(event: SubmitEvent) {
    event.preventDefault();
    console.log('Submitting operation:', operationType, formData);
    
    // Basic validation
    if (!formData.fromAddress || !formData.toAddress) {
      error = 'Please fill in required fields';
      return;
    }

    // Amount validation for operations that need it
    if (showField('amount') && (!formData.amount || parseFloat(formData.amount) <= 0)) {
      error = 'Please enter a valid amount';
      return;
    }

    isSubmitting = true;
    error = null;

    try {
      // Build sendcurrency parameters based on operation type
      const outputs = buildOutputs();
      
      // Call sendcurrency RPC
      const chainParam = getChainParam(connectionState?.selectedChain);
      const result = await invoke('send_currency', { 
        fromAddress: formData.fromAddress,
        outputs: outputs,
        minConf: formData.minConfs ? parseInt(formData.minConfs) : undefined,
        feeAmount: formData.feeAmount ? parseFloat(formData.feeAmount) : undefined,
        subtractFeeFromAmount: formData.addConversionFees || undefined,
        chain: chainParam
      });
      
      // Show success message with operation ID
      const successMessage = getSuccessMessage(operationType);
      showSuccess(successMessage, { operationId: result });
      onClose();
      
    } catch (err) {
      error = typeof err === 'string' ? err : 'Operation failed';
      console.error('Operation error:', err);
      showError(error);
    } finally {
      isSubmitting = false;
    }
  }

  function buildOutputs() {
    const output: any = {
      currency: formData.currency,
      address: formData.toAddress
    };

    // Set amount based on operation type
    if (operationType === 'export-definition' || operationType === 'export-identity') {
      output.amount = 0; // Hidden requirement for exports
    } else {
      output.amount = parseFloat(formData.amount);
    }

    // Add operation-specific parameters
    if (formData.convertTo) output.convertto = formData.convertTo;
    if (formData.via) output.via = formData.via;
    if (formData.exportTo) output.exportto = formData.exportTo;
    if (formData.memo) output.memo = formData.memo;
    if (formData.refundTo) output.refundto = formData.refundTo;
    if (formData.feeCurrency) output.feecurrency = formData.feeCurrency;

    // Hidden RPC parameters set automatically based on operation type
    if (operationType === 'export-definition') {
      output.exportcurrency = "true";
    } else if (operationType === 'export-identity') {
      output.exportid = "true";
    } else if (operationType === 'preconvert') {
      output.preconvert = "true";
    }

    // Handle advanced mode manual settings
    if (operationType === 'advanced') {
      if (formData.exportCurrency) output.exportcurrency = "true";
      if (formData.exportId) output.exportid = "true";
      if (formData.preconvert) output.preconvert = "true";
    }

    return [output];
  }

  // Get field visibility based on operation type
  function showField(field: string): boolean {
    if (!operationType) return false;
    
    const fieldMap: Record<OperationType, string[]> = {
      'send-transparent': ['fromAddress', 'toAddress', 'currency', 'amount'],
      'send-private': ['fromAddress', 'toAddress', 'currency', 'amount', 'memo'],
      'convert-reserve': ['fromAddress', 'toAddress', 'currency', 'amount', 'convertTo', 'via'],
      'convert-reserve-to-basket': ['fromAddress', 'toAddress', 'currency', 'amount', 'convertTo'],
      'convert-basket-to-reserve': ['fromAddress', 'toAddress', 'currency', 'amount', 'convertTo'],
      'preconvert': ['fromAddress', 'toAddress', 'currency', 'amount', 'convertTo', 'refundTo'],
      'export-definition': ['fromAddress', 'toAddress', 'currency', 'exportTo'],
      'send-crosschain': ['fromAddress', 'toAddress', 'currency', 'amount', 'exportTo'],
      'convert-crosschain': ['fromAddress', 'toAddress', 'currency', 'amount', 'convertTo', 'via', 'exportTo'],
      'export-identity': ['fromAddress', 'toAddress', 'exportTo'],
      'advanced': ['fromAddress', 'toAddress', 'currency', 'amount', 'convertTo', 'via', 'exportTo', 'exportCurrency', 'exportId', 'preconvert', 'memo', 'refundTo', 'minConfs', 'feeAmount', 'feeCurrency', 'addConversionFees', 'burn', 'mintNew'],
      'estimate-conversion': ['currency', 'amount', 'convertTo', 'via']
    };

    return fieldMap[operationType]?.includes(field) || false;
  }
</script>

<Modal {isOpen} onclose={onClose} title={currentConfig ? currentConfig.title : 'DeFi Operation'} size="lg">
  {#if currentConfig}
    <div class="p-6">
      <!-- Operation Info -->
      <div class="flex items-center space-x-3 mb-6">
        <span class="text-3xl">{currentConfig.icon}</span>
        <div>
          <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
            {currentConfig.description}
          </p>
        </div>
      </div>

      <!-- Error Display -->
      {#if error}
        <div class="mb-4 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
          <p class="text-red-700 dark:text-red-300">{error}</p>
        </div>
      {/if}

      <!-- Estimate Result Display -->
      {#if operationType === 'estimate-conversion' && estimateResult}
        <div class="mb-6 p-4 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg">
          <div class="flex items-center space-x-2 mb-2">
            <span class="text-green-600 dark:text-green-400">✅</span>
            <h3 class="text-lg font-semibold text-green-800 dark:text-green-200">Conversion Estimate</h3>
          </div>
          {#if estimateResult.estimatedcurrencyout}
            <div class="space-y-1">
              <p class="text-sm text-green-700 dark:text-green-300">Expected to receive:</p>
              <p class="text-xl font-mono font-bold text-green-800 dark:text-green-100">
                {Number(estimateResult.estimatedcurrencyout).toFixed(8)} {formData.convertTo}
              </p>
            </div>
          {:else if estimateResult.conversions && estimateResult.conversions[0] && estimateResult.conversions[0].estimatedcurrencyout}
            <div class="space-y-1">
              <p class="text-sm text-green-700 dark:text-green-300">Expected to receive:</p>
              <p class="text-xl font-mono font-bold text-green-800 dark:text-green-100">
                {Number(estimateResult.conversions[0].estimatedcurrencyout).toFixed(8)} {formData.convertTo}
              </p>
            </div>
          {:else}
            <p class="text-green-700 dark:text-green-300">
              Estimate result: {JSON.stringify(estimateResult)}
            </p>
          {/if}
        </div>
      {/if}

      <!-- Form -->
      <form onsubmit={operationType === 'estimate-conversion' ? handleEstimate : handleSubmit} class="space-y-4">
        <!-- From Address -->
        {#if showField('fromAddress')}
          <div>
            <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
              From Address *
            </label>
{#if operationType === 'export-identity'}
              <input 
                type="text" 
                bind:value={formData.fromAddress} 
                required
                placeholder="Enter identity (e.g. myid@)"
                class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
              />
            {:else if operationType === 'advanced'}
              <input 
                type="text" 
                bind:value={formData.fromAddress} 
                required
                placeholder="Enter from address"
                class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
              />
            {:else}
              <select 
                bind:value={formData.fromAddress} 
                required
                disabled={isLoadingAddresses}
                class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
              >
                <option value="">{isLoadingAddresses ? 'Loading addresses...' : 'Select address...'}</option>
                {#each availableAddresses as address}
                  <option value={address}>
                    {address}{address.startsWith('z') ? ' (Private)' : ' (Transparent)'}
                  </option>
                {/each}
              </select>
            {/if}
          </div>
        {/if}

        <!-- Currency -->
        {#if showField('currency')}
          <div>
            <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
              Currency *
            </label>
            {#if operationType === 'advanced' || operationType === 'export-definition' || operationType === 'estimate-conversion'}
              <input 
                type="text" 
                bind:value={formData.currency} 
                required
                placeholder="Enter currency name (e.g. VRSC, DAI.vETH)"
                class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
              />
            {:else}
              <select 
                bind:value={formData.currency} 
                required
                disabled={isLoadingCurrencies}
                class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
              >
                <option value="">{isLoadingCurrencies ? 'Loading currencies...' : 'Select currency...'}</option>
                {#each currencies as currency}
                  <option value={currency.name}>{currency.name} ({currency.balance.toFixed(8)})</option>
                {/each}
              </select>
            {/if}
          </div>
        {/if}

        <!-- Amount -->
        {#if showField('amount')}
          <div>
            <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
              Amount *
            </label>
            <input 
              type="number" 
              step="any"
              bind:value={formData.amount} 
              required
              placeholder="0.00"
              class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
            />
          </div>
        {/if}

        <!-- Via (for reserve to reserve) - comes first -->
        {#if showField('via')}
          <div>
            <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
              Via Basket *
            </label>
            {#if operationType === 'convert-crosschain' || operationType === 'advanced' || operationType === 'estimate-conversion'}
              <input 
                type="text" 
                bind:value={formData.via} 
                placeholder={operationType === 'advanced' ? 'Enter via currency (optional)' : operationType === 'estimate-conversion' ? 'Enter via currency (optional)' : 'Enter via currency (optional for cross-chain)'}
                class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
              />
            {:else}
              <select 
                bind:value={formData.via} 
                required
                disabled={isLoadingVia}
                class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
              >
                <option value="">{isLoadingVia ? 'Loading via currencies...' : 'Select basket currency...'}</option>
                {#each viaCurrencies as currency}
                  <option value={currency}>{currency}</option>
                {/each}
              </select>
            {/if}
          </div>
        {/if}

        <!-- Convert To -->
        {#if showField('convertTo')}
          <div>
            <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
              Convert To *
            </label>
            {#if operationType === 'convert-crosschain' || operationType === 'advanced' || operationType === 'estimate-conversion'}
              <input 
                type="text" 
                bind:value={formData.convertTo} 
                required
                placeholder={operationType === 'advanced' ? 'Enter target currency' : operationType === 'estimate-conversion' ? 'Enter target currency' : 'Enter target currency for cross-chain'}
                class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
              />
            {:else}
              <select 
                bind:value={formData.convertTo} 
                required
                disabled={isLoadingConvertTo}
                class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
              >
                <option value="">{isLoadingConvertTo ? 'Loading currencies...' : 'Select target currency...'}</option>
                {#each convertToCurrencies as currency}
                  {#if typeof currency === 'string'}
                    <option value={currency}>{currency}</option>
                  {:else}
                    <option value={currency.name}>{currency.display}</option>
                  {/if}
                {/each}
              </select>
            {/if}
          </div>
        {/if}

        <!-- Export To -->
        {#if showField('exportTo')}
          <div>
            <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
              Export To Chain *
            </label>
{#if operationType === 'advanced' || operationType === 'export-definition'}
              <input 
                type="text" 
                bind:value={formData.exportTo} 
                required
                placeholder="Enter network name (eg. vETH, vARRR)"
                class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
              />
            {:else}
              <select 
                bind:value={formData.exportTo} 
                required
                class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
              >
                <option value="">Select destination chain...</option>
                {#each chains as chain}
                  <option value={chain}>{chain.toUpperCase()}</option>
                {/each}
              </select>
            {/if}
          </div>
        {/if}

        <!-- To Address -->
        {#if showField('toAddress')}
          <div>
            <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
              To Address *
            </label>
            <div class="flex space-x-2">
              <input 
                type="text" 
                bind:value={formData.toAddress} 
                required
                placeholder="Enter destination address"
                class="flex-1 p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
              />
              {#if formData.fromAddress}
                <button
                  type="button"
                  onclick={() => fillSelfAddress('toAddress')}
                  class="px-3 py-2 text-sm bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-stone-dark dark:text-white rounded-lg hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-dark transition-colors"
                >
                  Self
                </button>
              {/if}
            </div>
          </div>
        {/if}

        <!-- Memo (for private sends) -->
        {#if showField('memo')}
          <div>
            <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
              Memo (Optional)
            </label>
            <textarea 
              bind:value={formData.memo} 
              placeholder="Optional message for private send"
              rows="3"
              class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white resize-none"
            ></textarea>
          </div>
        {/if}

        <!-- Refund To (for preconvert) -->
        {#if showField('refundTo')}
          <div>
            <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
              Refund To (Optional)
            </label>
            <div class="flex space-x-2">
              <input 
                type="text" 
                bind:value={formData.refundTo} 
                placeholder="Refund address (defaults to from address)"
                class="flex-1 p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
              />
              {#if formData.fromAddress}
                <button
                  type="button"
                  onclick={() => fillSelfAddress('refundTo')}
                  class="px-3 py-2 text-sm bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-stone-dark dark:text-white rounded-lg hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-dark transition-colors"
                >
                  Self
                </button>
              {/if}
            </div>
          </div>
        {/if}

        <!-- Advanced parameters (only in advanced mode) -->
        {#if operationType === 'advanced'}
          <div class="border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium pt-4">
            <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-4">Advanced Parameters</h3>
            
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <!-- Min Confirmations -->
              <div>
                <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
                  Min Confirmations
                </label>
                <input 
                  type="number" 
                  bind:value={formData.minConfs} 
                  placeholder="1"
                  class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
                />
              </div>

              <!-- Fee Amount -->
              <div>
                <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
                  Custom Fee Amount
                </label>
                <input 
                  type="number" 
                  step="any"
                  bind:value={formData.feeAmount} 
                  placeholder="Default fee"
                  class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
                />
              </div>

              <!-- Fee Currency -->
              <div>
                <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
                  Fee Currency
                </label>
                <input 
                  type="text" 
                  bind:value={formData.feeCurrency} 
                  placeholder="Default currency (VRSC)"
                  class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
                />
              </div>
            </div>

            <!-- Boolean options -->
            <div class="mt-4 space-y-3">
              <label class="flex items-center space-x-3">
                <input type="checkbox" bind:checked={formData.exportCurrency} class="rounded" />
                <span class="text-verusidx-stone-dark dark:text-white">Export Currency Definition</span>
              </label>
              
              <label class="flex items-center space-x-3">
                <input type="checkbox" bind:checked={formData.exportId} class="rounded" />
                <span class="text-verusidx-stone-dark dark:text-white">Export Identity</span>
              </label>
              
              <label class="flex items-center space-x-3">
                <input type="checkbox" bind:checked={formData.preconvert} class="rounded" />
                <span class="text-verusidx-stone-dark dark:text-white">Preconvert</span>
              </label>
              
              <label class="flex items-center space-x-3">
                <input type="checkbox" bind:checked={formData.addConversionFees} class="rounded" />
                <span class="text-verusidx-stone-dark dark:text-white">Add Conversion Fees</span>
              </label>
              
              <label class="flex items-center space-x-3">
                <input type="checkbox" bind:checked={formData.burn} class="rounded" />
                <span class="text-verusidx-stone-dark dark:text-white">Burn Currency</span>
              </label>
              
              <label class="flex items-center space-x-3">
                <input type="checkbox" bind:checked={formData.mintNew} class="rounded" />
                <span class="text-verusidx-stone-dark dark:text-white">Mint New</span>
              </label>
            </div>
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
            disabled={operationType === 'estimate-conversion' ? isEstimating : isSubmitting}
            class="flex-1 px-6 py-3 bg-verusidx-mountain-blue hover:bg-verusidx-lake-blue text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {#if operationType === 'estimate-conversion'}
              {isEstimating ? 'Estimating...' : 'Estimate Conversion'}
            {:else}
              {isSubmitting ? 'Submitting...' : 'Submit Operation'}
            {/if}
          </button>
        </div>
      </form>
    </div>
  {/if}
</Modal>