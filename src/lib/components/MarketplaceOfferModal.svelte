<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Modal } from './cards';
  import { OfferTypeSelector, IdentityOfferForm, OfferPreview } from './';
  import { processOfferForTakeForm, type TakeOfferFormMapping } from '$lib/utils/takeOfferUtils';
  import { showSuccess, showError } from '$lib/services/notifications';

  // Props
  interface Props {
    isOpen: boolean;
    mode: 'make' | 'take';
    existingOffer?: any; // For taking offers
    onClose: () => void;
    onSuccess?: () => void;
  }

  let { isOpen = false, mode = 'make', existingOffer = null, onClose, onSuccess }: Props = $props();

  // Wizard state
  let currentStep = $state<'type-selection' | 'offer-details' | 'for-details' | 'confirmation' | 'success'>('type-selection');
  let selectedOfferType = $state<'id-for-id' | 'id-for-currency' | 'currency-for-id' | 'currency-for-currency' | null>(null);
  let offerTxId = $state<string | null>(null);

  // Form state
  let formData = $state({
    fromAddress: '',
    changeAddress: '',
    expiryBlocks: '',
    // Offering side data
    offerCurrency: '',
    offerAmount: '',
    offerIdentityName: '', // Simple name@ for offer side
    // For side data  
    forCurrency: '',
    forAmount: '',
    forAddress: '',
    forIdentity: {
      name: '',
      parent: '',
      primaryAddresses: [''],
      minimumSignatures: 1,
      revocationAuthority: '',
      recoveryAuthority: '',
      privateAddress: ''
    },
    // Take offer specific fields
    deliverCurrency: '',
    deliverAmount: '',
    deliverIdentity: '',
    deliverIdentityName: '', // Display name for the identity being delivered
    acceptCurrency: '',
    acceptAmount: '',
    acceptAddress: '',
    acceptIdentityName: '' // Identity name user will accept
  });

  // Available data
  let availableAddresses = $state<string[]>([]);
  let isLoadingAddresses = $state(false);
  let hasLoadedAddresses = $state(false);
  let isSubmitting = $state(false);
  let error = $state<string | null>(null);

  // Step progress
  let stepProgress = $derived(() => {
    const steps = ['type-selection', 'offer-details', 'for-details', 'confirmation', 'success'];
    const currentIndex = steps.indexOf(currentStep);
    return ((currentIndex + 1) / steps.length) * 100;
  });

  // Reset form when modal opens/closes or mode changes
  $effect(() => {
    if (!isOpen) {
      resetForm();
    } else if (isOpen && !hasLoadedAddresses) {
      console.log('🔄 Loading addresses for marketplace modal');
      loadAddresses();
      if (mode === 'take' && existingOffer) {
        prefillTakeOffer();
      }
    }
  });

  function resetForm() {
    currentStep = 'type-selection';
    selectedOfferType = null;
    offerTxId = null;
    formData = {
      fromAddress: '',
      changeAddress: '',
      expiryBlocks: '',
      offerCurrency: '',
      offerAmount: '',
      offerIdentityName: '',
      forCurrency: '',
      forAmount: '',
      forAddress: '',
      forIdentity: {
        name: '',
        parent: '',
        primaryAddresses: [''],
        minimumSignatures: 1,
        revocationAuthority: '',
        recoveryAuthority: '',
        privateAddress: ''
      },
      deliverCurrency: '',
      deliverAmount: '',
      deliverIdentity: '',
      deliverIdentityName: '',
      acceptCurrency: '',
      acceptAmount: '',
      acceptAddress: '',
      acceptIdentityName: ''
    };
    // Reset loading states
    hasLoadedAddresses = false;
    availableAddresses = [];
    error = null;
  }

  function prefillTakeOffer() {
    if (!existingOffer) {
      console.warn('❌ No existing offer provided for prefill');
      return;
    }
    
    console.log('🔍 Prefilling take offer with existingOffer:', JSON.stringify(existingOffer, null, 2));
    
    // Use utility function to process offer data
    const mapping = processOfferForTakeForm(existingOffer);
    
    if (!mapping) {
      console.error('❌ Failed to process offer for take form');
      error = 'Invalid offer data. Please try a different offer.';
      return;
    }
    
    console.log('🎯 Successfully processed offer mapping:', mapping);
    
    // Clear any previous error
    error = null;
    
    // Skip type selection for take offers since it's determined by the existing offer
    currentStep = 'offer-details';
    selectedOfferType = mapping.offerType;
    
    // Apply the mapping to form data
    // What user will ACCEPT (receive)
    if (mapping.acceptIdentityName) {
      formData.acceptIdentityName = mapping.acceptIdentityName;
      // Set default values for identity definition user will configure
      formData.forIdentity.name = mapping.acceptIdentityName;
      formData.forIdentity.parent = ''; // Empty by default, user must choose
      formData.forIdentity.primaryaddresses = ['']; // lowercase to match IdentityOfferForm output
      formData.forIdentity.minimumsignatures = 1;
    }
    
    if (mapping.acceptCurrency) {
      formData.acceptCurrency = mapping.acceptCurrency;
    }
    
    if (mapping.acceptAmount) {
      formData.acceptAmount = mapping.acceptAmount;
    }
    
    // What user will DELIVER (provide)
    if (mapping.deliverIdentity) {
      formData.deliverIdentity = mapping.deliverIdentity;
    }
    
    if (mapping.deliverIdentityName) {
      formData.deliverIdentityName = mapping.deliverIdentityName;
    }
    
    if (mapping.deliverCurrency) {
      formData.deliverCurrency = mapping.deliverCurrency;
    }
    
    if (mapping.deliverAmount) {
      formData.deliverAmount = mapping.deliverAmount;
    }
    
    console.log('✅ Form data updated with take offer mapping:', {
      offerType: selectedOfferType,
      accept: {
        identityName: formData.acceptIdentityName,
        currency: formData.acceptCurrency,
        amount: formData.acceptAmount
      },
      deliver: {
        identity: formData.deliverIdentity,
        identityName: formData.deliverIdentityName,
        currency: formData.deliverCurrency,
        amount: formData.deliverAmount
      }
    });
  }

  function handleTypeSelection(type: typeof selectedOfferType) {
    selectedOfferType = type;
    currentStep = 'offer-details';
  }

  function nextStep() {
    if (currentStep === 'type-selection') {
      currentStep = 'offer-details';
    } else if (currentStep === 'offer-details') {
      currentStep = 'for-details';
    } else if (currentStep === 'for-details') {
      currentStep = 'confirmation';
    }
  }

  function previousStep() {
    if (currentStep === 'confirmation') {
      currentStep = 'for-details';
    } else if (currentStep === 'for-details') {
      currentStep = 'offer-details';
    } else if (currentStep === 'offer-details') {
      currentStep = 'type-selection';
    }
  }

  function handleForIdentityChange(identityData: any) {
    console.log('🔄 MarketplaceOfferModal: handleForIdentityChange called with:', identityData);
    // Merge the data to preserve primaryAddresses array
    formData.forIdentity = {
      ...formData.forIdentity,
      ...identityData
    };
  }

  async function copyToClipboard(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      console.log("Copied to clipboard:", text);
    } catch (err) {
      console.error("Failed to copy:", err);
    }
  }

  async function loadAddresses() {
    if (isLoadingAddresses) {
      console.log('⏸️ Skipping loadAddresses - already loading');
      return;
    }
    
    console.log('🚀 Starting loadAddresses for marketplace modal');
    isLoadingAddresses = true;
    hasLoadedAddresses = false;
    
    try {
      const allAddresses = [];
      
      // Get addresses by account (default account "")
      try {
        console.log('📞 Calling get_addresses_by_account...');
        const accountAddresses = await invoke('get_addresses_by_account', { account: "" });
        if (Array.isArray(accountAddresses)) {
          allAddresses.push(...accountAddresses);
          console.log('✅ Loaded addresses from default account:', accountAddresses.length);
        }
      } catch (err) {
        console.error('❌ Failed to load addresses by account:', err);
      }
      
      // Get transparent addresses from list_address_groupings
      try {
        console.log('📞 Calling list_address_groupings...');
        const transparentResult = await invoke('list_address_groupings');
        
        if (Array.isArray(transparentResult)) {
          const addresses = [];
          
          // Extract addresses from nested groups of objects
          for (const group of transparentResult) {
            if (Array.isArray(group)) {
              for (const addressInfo of group) {
                if (addressInfo && typeof addressInfo === 'object' && addressInfo.address) {
                  addresses.push(addressInfo.address);
                }
              }
            }
          }
          
          console.log('✅ Parsed transparent addresses from groupings:', addresses.length);
          allAddresses.push(...addresses);
        }
      } catch (err) {
        console.error('❌ Failed to load transparent addresses:', err);
      }
      
      // Get private addresses from z_list_addresses
      try {
        console.log('📞 Calling z_list_addresses...');
        const privateResult = await invoke('z_list_addresses');
        
        if (Array.isArray(privateResult)) {
          console.log('✅ Parsed private addresses:', privateResult.length);
          allAddresses.push(...privateResult);
        }
      } catch (err) {
        console.error('❌ Failed to load private addresses:', err);
      }
      
      // Remove duplicates and update state
      const uniqueAddresses = [...new Set(allAddresses)];
      console.log('🎯 Final unique addresses:', uniqueAddresses.length);
      availableAddresses = uniqueAddresses;
      hasLoadedAddresses = true;
      console.log('✅ Address loading completed successfully');
      
    } catch (err) {
      console.error('💥 Critical error in loadAddresses:', err);
      hasLoadedAddresses = false;
    } finally {
      isLoadingAddresses = false;
      console.log('🏁 loadAddresses finally block - isLoadingAddresses set to false');
    }
  }


  async function handleSubmit() {
    console.log('🚀 Starting offer submission for mode:', mode);
    
    // Basic validation
    if (!formData.fromAddress || !selectedOfferType) {
      error = 'Please complete all required fields';
      return;
    }

    // Mode-specific validation
    if (mode === 'take') {
      if (!existingOffer) {
        error = 'No offer selected to take';
        return;
      }
      
      // Validate required fields based on offer type (now including all 6 types)
      if (selectedOfferType === 'id-for-id' || selectedOfferType === 'currency-for-id' || selectedOfferType === 'currency-for-ids') {
        if (!formData.deliverIdentity) {
          error = 'Please provide the identity you will deliver';
          return;
        }
      }
      
      if (selectedOfferType === 'id-for-currency' || selectedOfferType === 'currency-for-currency' || selectedOfferType === 'ids-for-currency') {
        if (!formData.deliverCurrency || !formData.deliverAmount) {
          error = 'Please provide the currency and amount you will deliver';
          return;
        }
      }
      
      if (selectedOfferType === 'id-for-id' || selectedOfferType === 'id-for-currency' || selectedOfferType === 'ids-for-currency') {
        if (!formData.acceptIdentityName) {
          error = 'Please configure the identity you will accept';
          return;
        }
      }
      
      if (selectedOfferType === 'currency-for-id' || selectedOfferType === 'currency-for-currency' || selectedOfferType === 'currency-for-ids') {
        if (!formData.acceptCurrency || !formData.acceptAmount) {
          error = 'Please configure the currency you will accept';
          return;
        }
      }
    }

    isSubmitting = true;
    error = null;

    try {
      let result;
      
      if (mode === 'make') {
        console.log('🔧 Building make offer data...');
        const offerData = buildMakeOfferData();
        console.log('📤 Sending make offer with data:', offerData);

        result = await invoke('make_offer', {
          fromAddress: formData.fromAddress,
          offerData: offerData,
          returnTx: false
        });
        
      } else if (mode === 'take') {
        console.log('🔧 Building take offer data...');
        const takeData = buildTakeOfferData();
        console.log('📤 Sending take offer with data:', JSON.stringify(takeData, null, 2));
        
        // Validate identity structure if present
        if (takeData.accept && typeof takeData.accept === 'object' && 'name' in takeData.accept) {
          console.log('✅ Identity structure validation:', {
            hasName: 'name' in takeData.accept,
            hasParent: 'parent' in takeData.accept,
            hasPrimaryAddresses: 'primaryaddresses' in takeData.accept,
            primaryAddressesType: Array.isArray(takeData.accept.primaryaddresses),
            primaryAddressesCount: takeData.accept.primaryaddresses?.length,
            hasMinimumSignatures: 'minimumsignatures' in takeData.accept,
            minimumSignaturesType: typeof takeData.accept.minimumsignatures
          });
        }

        result = await invoke('take_offer', {
          fromAddress: formData.fromAddress,
          offerData: takeData,
          returnTx: false
        });
      }

      console.log(`✅ ${mode} offer result:`, result);
      
      // Extract transaction ID and transition to success step
      offerTxId = typeof result === 'string' ? result : (result as any).txid || String(result);
      
      if (!offerTxId) {
        console.warn('⚠️ No transaction ID returned from offer submission:', result);
        offerTxId = 'Unknown';
      }
      
      // Show success toast
      const successMessage = mode === 'make' ? 'Success: offer created' : 'Success: offer taken';
      showSuccess(successMessage, { txid: offerTxId });
      
      currentStep = 'success';
      
      // Notify parent component if needed
      if (onSuccess) {
        onSuccess();
      }
      
    } catch (err) {
      console.error(`❌ ${mode} offer failed:`, err);
      
      // Try to extract more meaningful error message
      let errorMessage = `Failed to ${mode} offer`;
      if (typeof err === 'string') {
        errorMessage = err;
      } else if (err && typeof err === 'object' && 'message' in err) {
        errorMessage = (err as Error).message;
      }
      
      // Add context for common errors
      if (errorMessage.includes('insufficient funds')) {
        errorMessage += '. Please check your wallet balance.';
      } else if (errorMessage.includes('transaction already exists')) {
        errorMessage += '. This offer may have already been taken.';
      } else if (errorMessage.includes('invalid')) {
        errorMessage += '. Please check the offer data and try again.';
      }
      
      error = errorMessage;
      showError(errorMessage);
    } finally {
      isSubmitting = false;
    }
  }

  function buildMakeOfferData() {
    const baseData = {
      changeaddress: formData.changeAddress || formData.fromAddress,
      expiryheight: formData.expiryBlocks ? parseInt(formData.expiryBlocks) : undefined
    };

    switch (selectedOfferType) {
      case 'id-for-id':
        // Build clean identity definition (following UpdateIdentity/RecoverIdentity pattern)
        const idForIdIdentity: any = {
          name: formData.forIdentity.name,
          parent: formData.forIdentity.parent,
          primaryaddresses: formData.forIdentity.primaryAddresses?.filter(addr => addr?.trim()) || [],
          minimumsignatures: formData.forIdentity.minimumSignatures
        };
        // Add optional parameters only if they have values
        if (formData.forIdentity.revocationAuthority) {
          idForIdIdentity.revocationauthority = formData.forIdentity.revocationAuthority;
        }
        if (formData.forIdentity.recoveryAuthority) {
          idForIdIdentity.recoveryauthority = formData.forIdentity.recoveryAuthority;
        }
        if (formData.forIdentity.privateAddress) {
          idForIdIdentity.privateaddress = formData.forIdentity.privateAddress;
        }
        return {
          ...baseData,
          offer: { identity: formData.offerIdentityName },
          for: idForIdIdentity
        };
      
      case 'id-for-currency':
        return {
          ...baseData,
          offer: { identity: formData.offerIdentityName },
          for: {
            address: formData.forAddress || formData.fromAddress,
            currency: formData.forCurrency,
            amount: parseFloat(formData.forAmount)
          }
        };
      
      case 'currency-for-id':
        // Build clean identity definition (following UpdateIdentity/RecoverIdentity pattern)
        const currencyForIdIdentity: any = {
          name: formData.forIdentity.name,
          parent: formData.forIdentity.parent,
          primaryaddresses: formData.forIdentity.primaryAddresses?.filter(addr => addr?.trim()) || [],
          minimumsignatures: formData.forIdentity.minimumSignatures
        };
        // Add optional parameters only if they have values
        if (formData.forIdentity.revocationAuthority) {
          currencyForIdIdentity.revocationauthority = formData.forIdentity.revocationAuthority;
        }
        if (formData.forIdentity.recoveryAuthority) {
          currencyForIdIdentity.recoveryauthority = formData.forIdentity.recoveryAuthority;
        }
        if (formData.forIdentity.privateAddress) {
          currencyForIdIdentity.privateaddress = formData.forIdentity.privateAddress;
        }
        return {
          ...baseData,
          offer: {
            currency: formData.offerCurrency,
            amount: parseFloat(formData.offerAmount)
          },
          for: currencyForIdIdentity
        };
      
      case 'currency-for-currency':
        return {
          ...baseData,
          offer: {
            currency: formData.offerCurrency,
            amount: parseFloat(formData.offerAmount)
          },
          for: {
            address: formData.forAddress || formData.fromAddress,
            currency: formData.forCurrency,
            amount: parseFloat(formData.forAmount)
          }
        };
      
      case 'currency-for-ids':
        // Currency for identities (plural) - similar to currency-for-id but accepts any identity
        return {
          ...baseData,
          offer: {
            currency: formData.offerCurrency,
            amount: parseFloat(formData.offerAmount)
          },
          for: {
            ...formData.forIdentity,
            primaryaddresses: formData.forIdentity.primaryaddresses?.filter(addr => addr?.trim()) || []
          }
        };
      
      case 'ids-for-currency':
        // Identities for currency - similar to id-for-currency but multiple identities can bid
        return {
          ...baseData,
          offer: { identity: formData.offerIdentityName },
          for: {
            address: formData.forAddress || formData.fromAddress,
            currency: formData.forCurrency,
            amount: parseFloat(formData.forAmount)
          }
        };
      
      default:
        throw new Error('Invalid offer type selected');
    }
  }

  function buildTakeOfferData() {
    // Get transaction ID from the offer data - correct path based on real structure
    const txid = existingOffer?.offer?.txid;
    
    if (!txid) {
      console.error('❌ No transaction ID found in offer data:', existingOffer);
      throw new Error('No transaction ID found for offer to take. Please try refreshing the offer data.');
    }
    
    console.log('🔧 Building take offer data with txid:', txid);
    console.log('📊 Original offer structure:', {
      category: existingOffer?._category || existingOffer?.category,
      offer: existingOffer?.offer?.offer,
      accept: existingOffer?.offer?.accept
    });
    console.log('🔍 Form data state:', {
      selectedOfferType,
      fromAddress: formData.fromAddress,
      changeAddress: formData.changeAddress,
      forIdentity: formData.forIdentity,
      deliverIdentity: formData.deliverIdentity,
      deliverCurrency: formData.deliverCurrency,
      deliverAmount: formData.deliverAmount,
      acceptIdentityName: formData.acceptIdentityName,
      acceptCurrency: formData.acceptCurrency,
      acceptAmount: formData.acceptAmount,
      acceptAddress: formData.acceptAddress
    });

    const baseData = {
      txid: txid,
      changeaddress: formData.changeAddress || formData.fromAddress
    };

    // CORRECT SEMANTIC MAPPING based on takeoffer examples:
    // deliver = what user provides (to satisfy original maker's "for" requirements)
    // accept = what user receives (configured by user to receive original maker's "offer")

    switch (selectedOfferType) {
      case 'id-for-id':
        // User delivers identity (string), accepts identity (definition)
        // Build clean identity definition (following UpdateIdentity/RecoverIdentity pattern - NO timelock)
        const idForIdAccept: any = {
          name: formData.acceptIdentityName,
          parent: formData.forIdentity.parent,
          primaryaddresses: formData.forIdentity.primaryAddresses.filter(addr => addr.trim()),
          minimumsignatures: parseInt(formData.forIdentity.minimumSignatures)
        };
        if (formData.forIdentity.revocationAuthority) {
          idForIdAccept.revocationauthority = formData.forIdentity.revocationAuthority;
        }
        if (formData.forIdentity.recoveryAuthority) {
          idForIdAccept.recoveryauthority = formData.forIdentity.recoveryAuthority;
        }
        if (formData.forIdentity.privateAddress) {
          idForIdAccept.privateaddress = formData.forIdentity.privateAddress;
        }
        return {
          ...baseData,
          deliver: formData.deliverIdentity, // Identity name string user provides
          accept: idForIdAccept
        };
      
      case 'id-for-currency':
        // User delivers currency, accepts identity (definition)
        // Build clean identity definition (following UpdateIdentity/RecoverIdentity pattern - NO timelock)
        const idForCurrencyAccept: any = {
          name: formData.acceptIdentityName,
          parent: formData.forIdentity.parent,
          primaryaddresses: formData.forIdentity.primaryAddresses.filter(addr => addr.trim()),
          minimumsignatures: parseInt(formData.forIdentity.minimumSignatures)
        };
        if (formData.forIdentity.revocationAuthority) {
          idForCurrencyAccept.revocationauthority = formData.forIdentity.revocationAuthority;
        }
        if (formData.forIdentity.recoveryAuthority) {
          idForCurrencyAccept.recoveryauthority = formData.forIdentity.recoveryAuthority;
        }
        if (formData.forIdentity.privateAddress) {
          idForCurrencyAccept.privateaddress = formData.forIdentity.privateAddress;
        }
        return {
          ...baseData,
          deliver: {
            currency: formData.deliverCurrency,
            amount: parseFloat(formData.deliverAmount)
          },
          accept: idForCurrencyAccept
        };
      
      case 'currency-for-id':
        // User delivers identity (string), accepts currency (with address)
        return {
          ...baseData,
          deliver: formData.deliverIdentity, // Identity name string user provides
          accept: {
            address: formData.acceptAddress || formData.fromAddress,
            currency: formData.acceptCurrency,
            amount: parseFloat(formData.acceptAmount)
          }
        };
      
      case 'currency-for-currency':
        // User delivers currency, accepts currency (with address)
        return {
          ...baseData,
          deliver: {
            currency: formData.deliverCurrency,
            amount: parseFloat(formData.deliverAmount)
          },
          accept: {
            address: formData.acceptAddress || formData.fromAddress,
            currency: formData.acceptCurrency,
            amount: parseFloat(formData.acceptAmount)
          }
        };
      
      case 'currency-for-ids':
        // User delivers identity (string), accepts currency (with address)
        // Same as currency-for-id semantically from taker perspective
        return {
          ...baseData,
          deliver: formData.deliverIdentity, // Identity name string user provides
          accept: {
            address: formData.acceptAddress || formData.fromAddress,
            currency: formData.acceptCurrency,
            amount: parseFloat(formData.acceptAmount)
          }
        };
      
      case 'ids-for-currency':
        // User delivers currency, accepts identity (definition)
        // Same as id-for-currency semantically from taker perspective
        return {
          ...baseData,
          deliver: {
            currency: formData.deliverCurrency,
            amount: parseFloat(formData.deliverAmount)
          },
          accept: {
            name: formData.acceptIdentityName,
            parent: formData.forIdentity.parent,
            primaryaddresses: formData.forIdentity.primaryAddresses.filter(addr => addr.trim()),
            minimumsignatures: parseInt(formData.forIdentity.minimumSignatures),
            ...(formData.forIdentity.revocationAuthority && { revocationauthority: formData.forIdentity.revocationAuthority }),
            ...(formData.forIdentity.recoveryAuthority && { recoveryauthority: formData.forIdentity.recoveryAuthority }),
            ...(formData.forIdentity.privateAddress && { privateaddress: formData.forIdentity.privateAddress }),
            ...(formData.forIdentity.timelock && { timelock: parseInt(formData.forIdentity.timelock) })
          }
        };
      
      default:
        throw new Error('Invalid offer type selected');
    }
  }

  // Get modal title
  let modalTitle = $derived(() => {
    switch (currentStep) {
      case 'type-selection': return `${mode === 'make' ? 'Make' : 'Take'} Offer - Select Type`;
      case 'offer-details': return `${mode === 'make' ? 'Make' : 'Take'} Offer - What You're ${mode === 'make' ? 'Offering' : 'Providing'}`;
      case 'for-details': return `${mode === 'make' ? 'Make' : 'Take'} Offer - What You ${mode === 'make' ? 'Want' : 'Are Accepting'}`;
      case 'confirmation': return `${mode === 'make' ? 'Make' : 'Take'} Offer - Confirm`;
      case 'success': return `${mode === 'make' ? 'Offer Created' : 'Offer Accepted'} - Success`;
      default: return 'Marketplace Offer';
    }
  });
</script>

<Modal {isOpen} onclose={onClose} title={modalTitle()} size="xl">
  <div class="p-6">
    <!-- Progress bar -->
    <div class="mb-6">
      <div class="flex items-center justify-between text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-2">
        <span class={currentStep === 'type-selection' ? 'text-verusidx-turquoise-deep font-medium' : ''}>Type</span>
        <span class={currentStep === 'offer-details' ? 'text-verusidx-turquoise-deep font-medium' : ''}>
          {mode === 'make' ? 'Offering' : 'Providing'}
        </span>
        <span class={currentStep === 'for-details' ? 'text-verusidx-turquoise-deep font-medium' : ''}>
          {mode === 'make' ? 'Requesting' : 'Accepting'}
        </span>
        <span class={currentStep === 'confirmation' ? 'text-verusidx-turquoise-deep font-medium' : ''}>Confirm</span>
        <span class={currentStep === 'success' ? 'text-verusidx-turquoise-deep font-medium' : ''}>Success</span>
      </div>
      <div class="w-full h-2 bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium rounded-full overflow-hidden">
        <div 
          class="h-full bg-verusidx-turquoise-deep transition-all duration-300"
          style="width: {stepProgress}%"
        ></div>
      </div>
    </div>

    <!-- Mode indicator -->
    <div class="flex items-center space-x-3 mb-6">
      <span class="text-2xl">
        {mode === 'make' ? '💰' : '🤝'}
      </span>
      <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
        {mode === 'make' 
          ? 'Create a new offer on the marketplace' 
          : 'Accept an existing offer from the marketplace'}
      </p>
    </div>

    <!-- Existing offer info (for take mode) -->
    {#if mode === 'take' && existingOffer}
      <div class="mb-6 space-y-4">
        <div class="p-4 bg-verusidx-snow-ice dark:bg-verusidx-stone-medium rounded-lg">
          <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-2">Taking Existing Offer</h3>
          <div class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist space-y-1">
            <p><span class="font-medium">Category:</span> {existingOffer._category || existingOffer.category || 'Unknown'}</p>
            {#if selectedOfferType}
              <p><span class="font-medium">Offer Type:</span> {selectedOfferType}</p>
            {/if}
            <p><span class="font-medium">Transaction ID:</span> {existingOffer.offer?.txid || 'Unknown'}</p>
          </div>
        </div>
        
        <!-- Debug Info (development mode) -->
        {#if typeof window !== 'undefined' && window.location.hostname === 'localhost'}
          <details class="p-4 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg">
            <summary class="cursor-pointer text-sm font-medium text-yellow-700 dark:text-yellow-300">
              Raw Offer Data
            </summary>
            <pre class="mt-2 text-xs overflow-auto bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-verusidx-mountain-mist p-2 rounded border max-h-40">{JSON.stringify(existingOffer, null, 2)}</pre>
          </details>
        {/if}
      </div>
    {/if}

    <!-- Error Display -->
    {#if error}
      <div class="mb-4 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
        <p class="text-red-700 dark:text-red-300">{error}</p>
      </div>
    {/if}

    <!-- Step 1: Type Selection (only for make offers) -->
    {#if currentStep === 'type-selection' && mode === 'make'}
      <OfferTypeSelector 
        selectedType={selectedOfferType}
        onTypeSelect={handleTypeSelection}
        mode={mode}
        disabled={isSubmitting}
      />

    <!-- Step 2: Offer Details -->
    {:else if currentStep === 'offer-details'}
      <div class="space-y-6">
        <!-- From Address -->
        <div>
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
            From Address *
          </label>
          <select 
            bind:value={formData.fromAddress} 
            required
            disabled={isLoadingAddresses || isSubmitting}
            class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
          >
            <option value="">{isLoadingAddresses ? 'Loading addresses...' : 'Select address...'}</option>
            {#each availableAddresses as address}
              <option value={address}>{address}</option>
            {/each}
          </select>
        </div>

        <!-- Conditional content based on offer type and mode -->
        {#if mode === 'make'}
          <!-- MAKE MODE: User is creating an offer -->
          {#if selectedOfferType === 'id-for-id' || selectedOfferType === 'id-for-currency' || selectedOfferType === 'ids-for-currency'}
            <!-- Offering Identity -->
            <div class="space-y-4">
              <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white flex items-center space-x-2">
                <span>👤</span>
                <span>Identity You're Offering</span>
              </h3>
              <div>
                <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
                  Identity Name *
                </label>
                <input 
                  type="text" 
                  bind:value={formData.offerIdentityName}
                  required
                  disabled={isSubmitting}
                  placeholder="e.g. myidentity@"
                  class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
                />
                <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
                  Enter the complete identity name including the @ symbol
                </p>
              </div>
            </div>
          {:else if selectedOfferType === 'currency-for-id' || selectedOfferType === 'currency-for-currency' || selectedOfferType === 'currency-for-ids'}
            <!-- Offering Currency -->
            <div class="space-y-4">
              <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white flex items-center space-x-2">
                <span>💰</span>
                <span>Currency You're Offering</span>
              </h3>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
                    Currency *
                  </label>
                  <input 
                    type="text" 
                    bind:value={formData.offerCurrency}
                    required
                    disabled={isSubmitting}
                    placeholder="e.g. VRSC, DAI.vETH"
                    class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
                  />
                </div>
                <div>
                  <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
                    Amount *
                  </label>
                  <input 
                    type="number" 
                    step="any"
                    bind:value={formData.offerAmount}
                    required
                    disabled={isSubmitting}
                    placeholder="0.00"
                    class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
                  />
                </div>
              </div>
            </div>
          {/if}
        {:else}
          <!-- TAKE MODE: What you will provide (give away) -->
          {#if selectedOfferType === 'id-for-id' || selectedOfferType === 'currency-for-id' || selectedOfferType === 'currency-for-ids'}
            <!-- Providing Identity (simple input) -->
            <div class="space-y-4">
              <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white flex items-center space-x-2">
                <span>👤</span>
                <span>Identity You Will Provide</span>
              </h3>
              <div>
                <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
                  Identity ID *
                </label>
                <input 
                  type="text" 
                  bind:value={formData.deliverIdentity}
                  required
                  disabled={isSubmitting}
                  placeholder="Identity i-address"
                  class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
                />
                {#if formData.deliverIdentityName}
                  <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
                    Identity name: <span class="font-medium">{formData.deliverIdentityName}</span>
                  </p>
                {/if}
                <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
                  This identity will be transferred to the other party
                </p>
              </div>
            </div>
          {:else if selectedOfferType === 'id-for-currency' || selectedOfferType === 'currency-for-currency' || selectedOfferType === 'ids-for-currency'}
            <!-- Providing Currency (what the offer requires) -->
            <div class="space-y-4">
              <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white flex items-center space-x-2">
                <span>💰</span>
                <span>Currency You Will Provide</span>
              </h3>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
                    Currency *
                  </label>
                  <input 
                    type="text" 
                    bind:value={formData.deliverCurrency}
                    required
                    disabled={isSubmitting || formData.deliverCurrency}
                    placeholder="e.g. VRSC, DAI.vETH"
                    class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
                  />
                  {#if formData.deliverCurrency}
                    <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
                      Required by the existing offer
                    </p>
                  {/if}
                </div>
                <div>
                  <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
                    Amount *
                  </label>
                  <input 
                    type="number" 
                    step="any"
                    bind:value={formData.deliverAmount}
                    required
                    disabled={isSubmitting || formData.deliverAmount}
                    placeholder="0.00"
                    class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
                  />
                  {#if formData.deliverAmount}
                    <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
                      Required by the existing offer
                    </p>
                  {/if}
                </div>
              </div>
              <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
                This currency amount will be sent to the other party as required by their offer
              </p>
            </div>
          {/if}
        {/if}

        <!-- Change Address -->
        <div>
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
            Change Address (optional)
          </label>
          <input 
            type="text" 
            bind:value={formData.changeAddress}
            disabled={isSubmitting}
            placeholder="Leave empty to use from address"
            class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
          />
        </div>

        <!-- Expiry Blocks -->
        {#if mode === 'make'}
          <div>
            <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
              Expiry (Block Height)
            </label>
            <input 
              type="number" 
              bind:value={formData.expiryBlocks}
              disabled={isSubmitting}
              placeholder="Leave empty for default (20 blocks)"
              class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
            />
          </div>
        {/if}
      </div>

    <!-- Step 3: For Details -->
    {:else if currentStep === 'for-details'}
      <div class="space-y-6">
        {#if mode === 'make'}
          <!-- MAKE MODE: What you want in return -->
          {#if selectedOfferType === 'id-for-id' || selectedOfferType === 'currency-for-id' || selectedOfferType === 'currency-for-ids'}
            <!-- Requesting Identity -->
            <IdentityOfferForm 
              identityData={formData.forIdentity}
              onIdentityChange={handleForIdentityChange}
              mode="accept"
              disabled={isSubmitting}
            />
          {:else if selectedOfferType === 'id-for-currency' || selectedOfferType === 'currency-for-currency' || selectedOfferType === 'ids-for-currency'}
            <!-- Requesting Currency -->
            <div class="space-y-4">
              <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white flex items-center space-x-2">
                <span>💰</span>
                <span>Currency You Want</span>
              </h3>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
                    Currency *
                  </label>
                  <input 
                    type="text" 
                    bind:value={formData.forCurrency}
                    required
                    disabled={isSubmitting}
                    placeholder="e.g. VRSC, DAI.vETH"
                    class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
                  />
                </div>
                <div>
                  <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
                    Amount *
                  </label>
                  <input 
                    type="number" 
                    step="any"
                    bind:value={formData.forAmount}
                    required
                    disabled={isSubmitting}
                    placeholder="0.00"
                    class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
                  />
                </div>
                <div class="md:col-span-2">
                  <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
                    Destination Address
                  </label>
                  <input 
                    type="text" 
                    bind:value={formData.forAddress}
                    disabled={isSubmitting}
                    placeholder="Address to receive the currency"
                    class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
                  />
                  <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
                    If left blank, will default to your selected from address
                  </p>
                </div>
              </div>
            </div>
          {/if}
        {:else}
          <!-- TAKE MODE: What you're accepting (what YOU configure and control) -->
          {#if selectedOfferType === 'id-for-id' || selectedOfferType === 'id-for-currency' || selectedOfferType === 'ids-for-currency'}
            <!-- Accepting Identity - YOU configure how to receive it -->
            <div class="space-y-4">
              <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white flex items-center space-x-2">
                <span>👤</span>
                <span>Identity You're Accepting</span>
              </h3>
              
              <!-- Show the identity name you're accepting (read-only) -->
              <div class="p-3 bg-verusidx-snow-ice dark:bg-verusidx-stone-medium rounded-lg mb-4">
                <p class="text-sm text-verusidx-stone-dark dark:text-white">
                  <span class="font-medium">Identity Name:</span> {formData.acceptIdentityName || 'Loading...'}
                </p>
                <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
                  This is the identity you will receive from the offer
                </p>
              </div>
              
              <!-- Configure how YOU want to receive this identity using IdentityOfferForm -->
              <div class="space-y-4">
                <h4 class="text-md font-medium text-verusidx-stone-dark dark:text-white">
                  Configure How You Want to Receive This Identity
                </h4>
                
                <IdentityOfferForm 
                  identityData={formData.forIdentity}
                  onIdentityChange={handleForIdentityChange}
                  mode="accept"
                  disabled={isSubmitting}
                />
              </div>
              
              <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
                You are configuring how this identity will be set up for your control when you receive it.
              </p>
            </div>
          {:else if selectedOfferType === 'currency-for-id' || selectedOfferType === 'currency-for-currency' || selectedOfferType === 'currency-for-ids'}
            <!-- Accepting Currency - YOU specify your receiving address -->
            <div class="space-y-4">
              <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white flex items-center space-x-2">
                <span>💰</span>
                <span>Currency You're Accepting</span>
              </h3>
              
              <!-- Show what currency you're accepting (read-only) -->
              <div class="p-3 bg-verusidx-snow-ice dark:bg-verusidx-stone-medium rounded-lg mb-4">
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm text-verusidx-stone-dark dark:text-white">
                  <div>
                    <p class="font-medium">Currency:</p>
                    <p>{formData.acceptCurrency || 'Loading...'}</p>
                  </div>
                  <div>
                    <p class="font-medium">Amount:</p>
                    <p>{formData.acceptAmount || 'Loading...'}</p>
                  </div>
                </div>
                <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-2">
                  This is the currency and amount you will receive from the offer
                </p>
              </div>
              
              <!-- Configure where YOU want to receive the currency -->
              <div class="space-y-4">
                <h4 class="text-md font-medium text-verusidx-stone-dark dark:text-white">
                  Where Do You Want to Receive This Currency?
                </h4>
                
                <div>
                  <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
                    Your Receiving Address *
                  </label>
                  <select 
                    bind:value={formData.acceptAddress}
                    required
                    disabled={isLoadingAddresses || isSubmitting}
                    class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
                  >
                    <option value="">{isLoadingAddresses ? 'Loading addresses...' : 'Select your address...'}</option>
                    {#each availableAddresses as address}
                      <option value={address}>{address}</option>
                    {/each}
                  </select>
                  <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
                    The currency will be sent to this address you control
                  </p>
                </div>
              </div>
              
              <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
                You are specifying where you want to receive the currency from this offer.
              </p>
            </div>
          {/if}
        {/if}
      </div>

    <!-- Step 4: Confirmation -->
    {:else if currentStep === 'confirmation' && selectedOfferType}
      <OfferPreview 
        offerType={selectedOfferType}
        mode={mode}
        formData={formData}
        existingOffer={existingOffer}
      />

    <!-- Step 5: Success -->
    {:else if currentStep === 'success'}
      <div class="space-y-6">
        <!-- Success Icon and Message -->
        <div class="text-center">
          <div class="text-6xl mb-4">🎉</div>
          <h3 class="text-2xl font-bold text-verusidx-stone-dark dark:text-white mb-2">
            {mode === 'make' ? 'Offer Created Successfully!' : 'Offer Accepted Successfully!'}
          </h3>
          <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
            {mode === 'make' 
              ? 'Your offer will be visible in the marketplace after 1 blockchain confirmation.' 
              : 'The exchange will complete after 1 blockchain confirmation.'}
          </p>
        </div>

        <!-- Transaction Details -->
        <div class="bg-verusidx-snow-ice dark:bg-verusidx-stone-medium rounded-lg p-4">
          <h4 class="font-medium text-verusidx-stone-dark dark:text-white mb-3">Transaction Details</h4>
          
          {#if offerTxId}
            <div class="space-y-3">
              <div>
                <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-1">Transaction ID:</p>
                <div class="flex items-center space-x-2">
                  <code class="flex-1 text-xs font-mono bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-verusidx-mountain-mist p-2 rounded border border-verusidx-mountain-mist dark:border-verusidx-stone-light break-all">
                    {offerTxId}
                  </code>
                  <button
                    onclick={() => copyToClipboard(offerTxId || '')}
                    class="px-3 py-2 bg-verusidx-mountain-mist dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-verusidx-mountain-mist text-sm rounded hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-medium transition-colors"
                    title="Copy transaction ID"
                  >
                    Copy
                  </button>
                </div>
              </div>
              
              <div class="pt-3 border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
                <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
                  <span class="text-yellow-600 dark:text-yellow-400">⚠️ Important:</span>
                  {mode === 'make'
                    ? 'Your offer will become active and visible to other users after blockchain confirmation. This typically takes one minute but may vary based on network conditions.'
                    : 'The atomic swap will execute automatically after blockchain confirmation. Both parties will receive their assets or the transaction will be reverted.'}
                </p>
              </div>
            </div>
          {/if}
        </div>

        <!-- Additional Information -->
        <div class="text-center">
          <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
            You can track your {mode === 'make' ? 'offer' : 'transaction'} status using the transaction ID above.
          </p>
        </div>
      </div>
    {/if}

    <!-- Navigation Buttons -->
    <div class="flex justify-between space-x-4 pt-6">
      {#if currentStep === 'success'}
        <!-- Success step: only show Done button -->
        <div></div> <!-- Empty space for layout -->
        <button
          onclick={onClose}
          class="px-6 py-3 bg-verusidx-forest-light hover:bg-verusidx-forest-deep text-white rounded-lg transition-colors"
        >
          Done
        </button>
      {:else}
        <!-- Back/Cancel -->
        <button
          type="button"
          onclick={currentStep === 'type-selection' ? onClose : previousStep}
          disabled={isSubmitting}
          class="px-6 py-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium text-verusidx-stone-dark dark:text-white rounded-lg hover:bg-verusidx-mountain-mist dark:hover:bg-verusidx-stone-medium transition-colors disabled:opacity-50"
        >
          {currentStep === 'type-selection' ? 'Cancel' : 'Back'}
        </button>

        <!-- Next/Submit -->
        {#if currentStep === 'confirmation'}
          <button
            onclick={handleSubmit}
            disabled={isSubmitting || !formData.fromAddress}
            class="px-6 py-3 bg-verusidx-mountain-blue hover:bg-verusidx-lake-blue text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isSubmitting ? 'Processing...' : mode === 'make' ? 'Create Offer' : 'Accept Offer'}
          </button>
        {:else}
          <button
            onclick={nextStep}
            disabled={
              isSubmitting || 
              (currentStep === 'type-selection' && !selectedOfferType) ||
              (currentStep === 'offer-details' && !formData.fromAddress)
            }
            class="px-6 py-3 bg-verusidx-turquoise-deep hover:bg-verusidx-turquoise-bright text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Next
          </button>
        {/if}
      {/if}
    </div>
  </div>
</Modal>