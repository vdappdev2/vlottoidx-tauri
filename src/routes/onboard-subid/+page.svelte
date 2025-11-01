<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { connectionStore, type ConnectionState } from "$lib/stores/connection";
  import { goto } from "$app/navigation";
  import { BlockHeightHeader } from "$lib/components";

  let connectionState = $state<ConnectionState>({ current: null, isConnecting: false, lastError: null });
  let walletInfo = $state<any>(null);
  let addressGroupings = $state<any[]>([]);
  let rAddresses = $state<string[]>([]);
  let fundingStatus = $state<{
    hasEnoughVerusIDX: boolean;
    hasEnoughVUSDC: boolean;
    hasEnoughVRSC: boolean;
    requiredVerusIDX: number;
    requiredVRSC: number;
    currentVerusIDX: number;
    currentVUSDC: number;
    currentVRSC: number;
  }>({
    hasEnoughVerusIDX: false,
    hasEnoughVUSDC: false,
    hasEnoughVRSC: false,
    requiredVerusIDX: 0,
    requiredVRSC: 0,
    currentVerusIDX: 0,
    currentVUSDC: 0,
    currentVRSC: 0
  });
  let isLoading = $state(true);
  let isProcessing = $state(false);
  let currentStep = $state<'check' | 'fund' | 'register' | 'waiting' | 'completing' | 'finalizing'>('check');
  let selectedSubIdName = $state('');
  let errorMessage = $state('');
  let selectedFromAddress = $state('');
  let selectedToAddress = $state('');
  let commitmentBlockHeight = $state<number | null>(null);
  let currentBlockHeight = $state<number | null>(null);
  let showingConversionModal = $state(false);
  let conversionCurrency = $state<'vusdc.veth' | 'VRSC'>('vusdc.veth');
  let toAddresses = $state<string[]>([]);
  let isLoadingToAddresses = $state(false);
  let conversionOperationId = $state<string | null>(null);
  let operationStatus = $state<any>(null);
  let showSuccessMessage = $state(false);
  let walletRefreshInterval = $state<number | null>(null);
  let isWaitingForFunds = $state(false);
  let blockMonitorInterval = $state<number | null>(null);
  let lastCheckedBlockHeight = $state<number | null>(null);
  let showFundsReceivedNotification = $state(false);
  let commitmentResponse = $state<any>(null);
  let controlAddresses = $state<string[]>([]);
  let selectedControlAddress = $state('');
  let primaryAddresses = $state<string[]>([]);
  let selectedPrimaryAddress = $state('');
  let isLoadingControlAddresses = $state(false);
  let isLoadingPrimaryAddresses = $state(false);
  let registrationBlockHeight = $state<number | null>(null);
  let copiedCommitment = $state(false);

  const VERUSIDX_COST_IN_VUSDC = 4.01;

  // Configuration Toggle - Comment/uncomment to switch environments
  // TESTING CONFIG
  // const CURRENCY_NAME = "Agents";
  // const PARENT_NAME = "agents"; 
  // const REFERRAL_ID = "";
  // const UI_CURRENCY_DISPLAY = "agents";

  // PRODUCTION CONFIG
  const CURRENCY_NAME = "VerusIDX";
  const PARENT_NAME = "VerusIDX";
  const REFERRAL_ID = "REF.VerusIDX@";
  const UI_CURRENCY_DISPLAY = "VerusIDX";

  connectionStore.subscribe(state => {
    connectionState = state;
    if (!state.current?.isConnected) {
      goto("/");
    }
  });

  // Disconnect and go home function
  function disconnectAndGoHome() {
    connectionStore.update(() => ({ 
      current: null, 
      isConnecting: false, 
      lastError: null,
      availableChains: [],
      selectedChain: null
    }));
    goto("/");
  }

  async function loadAddresses() {
    if (!connectionState.current?.isConnected) return;

    try {
      // Get address groupings
      addressGroupings = await invoke("list_address_groupings");
      
      // Extract R-addresses (transparent addresses starting with 'R')
      const addresses: string[] = [];
      for (const group of addressGroupings) {
        for (const addressInfo of group) {
          const address = addressInfo[0]; // First element is the address
          if (typeof address === 'string' && address.startsWith('R')) {
            addresses.push(address);
          }
        }
      }
      
      rAddresses = [...new Set(addresses)]; // Remove duplicates
      
      // Set default to address if none selected
      if (rAddresses.length > 0 && !selectedToAddress) {
        selectedToAddress = rAddresses[0];
      }
      
    } catch (error) {
      console.error("Failed to load addresses:", error);
    }
  }

  async function loadToAddresses() {
    if (!connectionState.current?.isConnected) return;
    
    isLoadingToAddresses = true;
    try {
      const addresses = new Set<string>();
      
      // Get addresses from getaddressesbyaccount
      try {
        const accountAddresses = await invoke("get_addresses_by_account", {
          account: ""
        }) as string[];
        accountAddresses.forEach(addr => addresses.add(addr));
      } catch (error) {
        console.warn("Failed to get addresses by account:", error);
      }
      
      // Get addresses from listaddressgroupings
      try {
        const groupings = await invoke("list_address_groupings") as any[];
        for (const group of groupings) {
          for (const addressInfo of group) {
            const address = addressInfo.address;
            if (typeof address === 'string') {
              addresses.add(address);
            }
          }
        }
      } catch (error) {
        console.warn("Failed to get address groupings:", error);
      }
      
      toAddresses = Array.from(addresses).sort();
      
      // Keep selectedToAddress empty to show placeholder
      // User must manually select an address
      
    } catch (error) {
      console.error("Failed to load to addresses:", error);
    } finally {
      isLoadingToAddresses = false;
    }
  }

  async function checkFundingStatus() {
    if (!connectionState.current?.isConnected) return;

    isLoading = true;
    errorMessage = '';

    try {
      // Load addresses first
      await loadAddresses();
      
      // Get wallet info
      walletInfo = await invoke("get_wallet_info");
      
      // Extract current balances
      // Add debug logging to see what we're getting
      console.log("Wallet reserve_balance keys:", Object.keys(walletInfo.reserve_balance || {}));
      console.log("Reserve balance contents:", walletInfo.reserve_balance);
      
      const currentVerusIDX = walletInfo.reserve_balance?.[CURRENCY_NAME] || 0;
      const currentVUSDC = walletInfo.reserve_balance?.["vUSDC.vETH"] || 0;
      const currentVRSC = walletInfo.balance || 0;

      console.log(`Looking for ${CURRENCY_NAME}:`, walletInfo.reserve_balance?.[CURRENCY_NAME]);
      console.log("Looking for vUSDC.vETH:", walletInfo.reserve_balance?.["vUSDC.vETH"]);
      console.log("VRSC balance:", currentVRSC);

      // Calculate how much VerusIDX we need (equivalent to 4.01 vusdc.veth)
      const conversionResult = await invoke("estimate_conversion", {
        currency: "vusdc.veth",
        amount: VERUSIDX_COST_IN_VUSDC,
        convertto: PARENT_NAME
      }) as any;

      const requiredVerusIDX = conversionResult.estimatedcurrencyout;

      // Calculate how much VRSC needed using reserve currency pricing
      let requiredVRSC = 0;
      
      // Find VRSC currency in reserve currencies array
      const vrscCurrencyId = "i5w5MuNik5NtLcYmNzcvaoixooEebB6MGV";
      const reserveCurrencies = conversionResult.estimatedcurrencystate?.reservecurrencies || [];
      const vrscReserve = reserveCurrencies.find(currency => currency.currencyid === vrscCurrencyId);
      
      if (vrscReserve) {
        // Calculate VRSC needed: verusidx_needed × vrsc_price_in_reserve
        const calculatedVRSC = requiredVerusIDX * vrscReserve.priceinreserve;
        
        // Verify calculation with estimate_conversion call
        try {
          const verificationResult = await invoke("estimate_conversion", {
            currency: "VRSC",
            amount: calculatedVRSC,
            convertto: PARENT_NAME
          }) as any;
          
          requiredVRSC = calculatedVRSC;
        } catch (e) {
          console.warn("Verification call failed, using calculated amount:", e);
          requiredVRSC = calculatedVRSC;
        }
      } else {
        console.warn("Could not find VRSC currency in reserve currencies");
      }

      // Update funding status
      fundingStatus = {
        hasEnoughVerusIDX: currentVerusIDX >= requiredVerusIDX,
        hasEnoughVUSDC: currentVUSDC >= VERUSIDX_COST_IN_VUSDC,
        hasEnoughVRSC: currentVRSC >= requiredVRSC,
        requiredVerusIDX,
        requiredVRSC,
        currentVerusIDX,
        currentVUSDC,
        currentVRSC
      };

      currentStep = fundingStatus.hasEnoughVerusIDX ? 'register' : 'fund';
      
      // If we have enough funds and were waiting, stop the refresh interval
      if (fundingStatus.hasEnoughVerusIDX && isWaitingForFunds) {
        stopWalletRefresh();
        isWaitingForFunds = false;
        
        // Show success notification
        showFundsReceivedNotification = true;
        console.log("Funds received! You can now register your Sub-ID");
        
        // Hide notification after 5 seconds
        setTimeout(() => {
          showFundsReceivedNotification = false;
        }, 5000);
      }

    } catch (error) {
      errorMessage = `Failed to check funding status: ${error}`;
      console.error("Funding check failed:", error);
    } finally {
      isLoading = false;
    }
  }

  async function convertToVerusIDX(fromCurrency: 'vusdc.veth' | 'VRSC') {
    console.log("convertToVerusIDX called with:", fromCurrency);
    console.log("From address:", selectedFromAddress);
    console.log("To address:", selectedToAddress);
    
    if (!connectionState.current?.isConnected || !selectedToAddress) {
      errorMessage = "Please select a receiving address";
      return;
    }

    isProcessing = true;
    errorMessage = '';

    try {
      let amount = fromCurrency === 'vusdc.veth' ? VERUSIDX_COST_IN_VUSDC : fundingStatus.requiredVRSC;
      
      // Add validation
      console.log("Amount to convert:", amount);
      console.log("Amount type:", typeof amount);
      
      if (!amount || amount <= 0) {
        errorMessage = `Invalid amount: ${amount}. Please wait for funding status to load.`;
        isProcessing = false;
        return;
      }
      
      // Ensure amount has proper precision (8 decimal places max for Verus)
      amount = parseFloat(amount.toFixed(8));
      console.log("Formatted amount:", amount);
      
      const result = await invoke("convert_to_verusidx", {
        fromAddress: selectedFromAddress,
        sourceCurrency: fromCurrency,
        amount: amount,
        toAddress: selectedToAddress
      });

      console.log("Conversion result:", result);
      
      // Store operation ID and show success
      conversionOperationId = result as string;
      showSuccessMessage = true;
      
      // Start periodic wallet refresh to monitor for funds
      startWalletRefresh();
      
      // Don't close modal immediately - show success message
      // Check operation status after a delay
      setTimeout(async () => {
        await checkOperationStatus(conversionOperationId);
      }, 2000);
      
    } catch (error) {
      errorMessage = `Conversion failed: ${error}`;
      console.error("Conversion failed:", error);
    } finally {
      isProcessing = false;
    }
  }

  async function loadControlAddresses() {
    if (!connectionState.current?.isConnected) return;
    
    isLoadingControlAddresses = true;
    try {
      const addresses = await invoke("get_addresses_by_account", {
        account: ""
      }) as string[];
      controlAddresses = addresses.sort();
      
      // Don't auto-select - let user choose from placeholder
    } catch (error) {
      console.error("Failed to load control addresses:", error);
    } finally {
      isLoadingControlAddresses = false;
    }
  }

  async function loadPrimaryAddresses() {
    if (!connectionState.current?.isConnected) return;
    
    isLoadingPrimaryAddresses = true;
    try {
      const addresses = await invoke("get_addresses_by_account", {
        account: ""
      }) as string[];
      primaryAddresses = addresses.sort();
    } catch (error) {
      console.error("Failed to load primary addresses:", error);
    } finally {
      isLoadingPrimaryAddresses = false;
    }
  }

  async function registerSubIdentity() {
    if (!selectedSubIdName.trim()) {
      errorMessage = "Please enter a sub-identity name";
      return;
    }

    // Validate Verus name rules - no leading/trailing spaces, no invalid characters
    const trimmedName = selectedSubIdName.trim();
    if (trimmedName !== selectedSubIdName) {
      errorMessage = "Name cannot have leading or trailing spaces";
      return;
    }
    
    if (trimmedName.includes('\\') || trimmedName.includes('/') || trimmedName.includes(':') || 
        trimmedName.includes('*') || trimmedName.includes('?') || trimmedName.includes('"') || 
        trimmedName.includes('<') || trimmedName.includes('>') || trimmedName.includes('|') || 
        trimmedName.includes('@')) {
      errorMessage = "Name cannot contain these characters: \\ / : * ? \" < > | @";
      return;
    }

    if (!selectedControlAddress) {
      errorMessage = "Please select a control address";
      return;
    }

    isProcessing = true;
    errorMessage = '';

    try {
      // Step 1: Register name commitment
      console.log("Registering name commitment for:", selectedSubIdName);
      
      const commitmentResult = await invoke("register_name_commitment", {
        name: selectedSubIdName,
        controlAddress: selectedControlAddress,
        referral: REFERRAL_ID,
        parentNameOrId: PARENT_NAME,
        sourceOfFunds: ""
      });

      console.log("Name commitment result:", commitmentResult);
      
      // Store the full commitment response
      commitmentResponse = commitmentResult;
      
      // Get current block height
      currentBlockHeight = await invoke("get_block_count");
      commitmentBlockHeight = currentBlockHeight;
      
      // Move to waiting step
      currentStep = 'waiting';
      
      // Start polling for block confirmation
      pollForBlockConfirmation();

    } catch (error) {
      errorMessage = `Name commitment failed: ${error}`;
      console.error("Registration failed:", error);
      isProcessing = false;
    }
  }

  async function pollForBlockConfirmation() {
    if (!commitmentBlockHeight) return;
    
    const pollInterval = setInterval(async () => {
      try {
        currentBlockHeight = await invoke("get_block_count");
        
        // Check if we have confirmation (next block)
        if (currentBlockHeight && commitmentBlockHeight && currentBlockHeight > commitmentBlockHeight) {
          clearInterval(pollInterval);
          await completeIdentityRegistration();
        }
      } catch (error) {
        console.error("Error polling block count:", error);
        // Continue polling despite error
      }
    }, 5000); // Poll every 5 seconds

    // Timeout after 15 minutes
    setTimeout(() => {
      clearInterval(pollInterval);
      if (currentStep === 'waiting') {
        errorMessage = "Block confirmation timeout. Please try again.";
        currentStep = 'register';
        isProcessing = false;
      }
    }, 900000);
  }

  async function completeIdentityRegistration() {
    // Move to completing step to get primary address selection
    currentStep = 'completing';
    
    // Load primary addresses
    if (primaryAddresses.length === 0) {
      await loadPrimaryAddresses();
    }
    
    isProcessing = false; // Allow user to select address
  }

  async function finalizeIdentityRegistration() {
    if (!commitmentResponse || !selectedPrimaryAddress) {
      errorMessage = "Missing commitment data or primary address";
      return;
    }

    isProcessing = true;
    errorMessage = '';

    try {
      // Construct proper identity object
      const identityObject = {
        name: selectedSubIdName,
        parent: PARENT_NAME,
        primaryaddresses: [selectedPrimaryAddress],
        minimumsignatures: 1
      };

      // First attempt: Call with null feeOffer to get daemon's expected fee
      try {
        const registrationResult = await invoke("register_identity", {
          txid: commitmentResponse.txid,
          namereservation: commitmentResponse.namereservation,
          identity: identityObject,
          return_tx: false,
          fee_offer: null,
          source_of_funds: null
        });

        // If first attempt succeeds (shouldn't happen for reserve-priced fees but handle it)
        console.log("Identity registration result:", registrationResult);

        // Get current block height for final confirmation wait
        currentBlockHeight = await invoke("get_block_count");
        registrationBlockHeight = currentBlockHeight;

        // Move to finalizing step
        currentStep = 'finalizing';

        // Start polling for block confirmation before redirect
        pollForRegistrationConfirmation();

      } catch (firstErr) {
        // Parse the error message to extract required fee
        const errorStr = typeof firstErr === 'string' ? firstErr : String(firstErr);
        const feeMatch = errorStr.match(/Fee offer must be at least ([\d.]+)/);

        if (feeMatch && feeMatch[1]) {
          const requiredFee = parseFloat(feeMatch[1]);

          // Second attempt: Retry with the required fee
          const registrationResult = await invoke("register_identity", {
            txid: commitmentResponse.txid,
            namereservation: commitmentResponse.namereservation,
            identity: identityObject,
            return_tx: false,
            fee_offer: requiredFee,
            source_of_funds: null
          });

          console.log("Identity registration result (with fee):", registrationResult);

          // Get current block height for final confirmation wait
          currentBlockHeight = await invoke("get_block_count");
          registrationBlockHeight = currentBlockHeight;

          // Move to finalizing step
          currentStep = 'finalizing';

          // Start polling for block confirmation before redirect
          pollForRegistrationConfirmation();
        } else {
          // Error didn't match expected pattern, show original error to user
          throw firstErr;
        }
      }

    } catch (error) {
      errorMessage = `Identity registration failed: ${error}`;
      console.error("Identity registration failed:", error);
      currentStep = 'completing'; // Go back to address selection
    } finally {
      isProcessing = false;
    }
  }

  async function pollForRegistrationConfirmation() {
    if (!registrationBlockHeight) return;
    
    const pollInterval = setInterval(async () => {
      try {
        const latestBlockHeight = await invoke("get_block_count") as number;
        
        // Check if we have confirmation (next block)
        if (latestBlockHeight > registrationBlockHeight) {
          clearInterval(pollInterval);
          // Redirect to access page for validation
          goto("/access");
        }
      } catch (error) {
        console.error("Error polling block count for registration:", error);
        // Continue polling despite error
      }
    }, 5000); // Poll every 5 seconds
    
    // Timeout after 10 minutes
    setTimeout(() => {
      clearInterval(pollInterval);
      if (currentStep === 'finalizing') {
        errorMessage = "Registration confirmation timeout. Please check manually.";
        // Still redirect to access page as fallback
        goto("/access");
      }
    }, 600000);
  }

  function showConversionModal(currency: 'vusdc.veth' | 'VRSC') {
    // Check if funding status is loaded
    if (currency === 'VRSC' && (!fundingStatus.requiredVRSC || fundingStatus.requiredVRSC <= 0)) {
      errorMessage = "Please wait for funding requirements to load";
      return;
    }
    
    conversionCurrency = currency;
    showingConversionModal = true;
    // Load addresses when modal opens
    loadToAddresses();
  }

  function closeConversionModal() {
    showingConversionModal = false;
    showSuccessMessage = false;
    conversionOperationId = null;
    operationStatus = null;
  }
  
  async function checkOperationStatus(opid: string) {
    if (!opid) return;

    try {
      const status = await invoke("z_get_operation_status", {
        operation_ids: [opid]
      });
      
      if (Array.isArray(status) && status.length > 0) {
        operationStatus = status[0];
        console.log("Operation status:", operationStatus);
        
        // If operation is complete and successful, close modal
        if (operationStatus.status === "success") {
          setTimeout(() => {
            showingConversionModal = false;
            showSuccessMessage = false;
            conversionOperationId = null;
          }, 2000);
        }
      }
    } catch (error) {
      console.error("Failed to check operation status:", error);
    }
  }

  async function executeConversion() {
    console.log("Convert button clicked");
    console.log("Selected from address:", selectedFromAddress);
    console.log("Selected to address:", selectedToAddress);
    console.log("Conversion currency:", conversionCurrency);
    console.log("Is processing:", isProcessing);

    if (!selectedFromAddress || !selectedToAddress) {
      errorMessage = "Please select both from and to addresses";
      return;
    }

    await convertToVerusIDX(conversionCurrency);
    // Don't close modal immediately - let convertToVerusIDX handle success display
  }

  async function copyCommitmentToClipboard() {
    if (!commitmentResponse) return;

    try {
      // Reorder the JSON to show txid first and namereservation fields in specific order
      const orderedResult = {
        txid: commitmentResponse.txid,
        namereservation: {
          version: commitmentResponse.namereservation?.version,
          name: commitmentResponse.namereservation?.name,
          parent: commitmentResponse.namereservation?.parent,
          salt: commitmentResponse.namereservation?.salt,
          referral: commitmentResponse.namereservation?.referral,
          nameid: commitmentResponse.namereservation?.nameid
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

  // Start periodic wallet refresh after conversion
  function startWalletRefresh() {
    // Clear any existing intervals
    stopWalletRefresh();
    
    console.log("Starting periodic wallet refresh every 60 seconds");
    isWaitingForFunds = true;
    
    // Initial check after 5 seconds
    setTimeout(() => {
      checkFundingStatus();
    }, 5000);
    
    // Then check every 60 seconds
    walletRefreshInterval = setInterval(() => {
      console.log("Periodic wallet refresh triggered");
      checkFundingStatus();
    }, 60000); // 60 seconds
    
    // Also start block monitoring
    startBlockMonitoring();
    
    // Set a maximum timeout of 15 minutes
    setTimeout(() => {
      if (isWaitingForFunds) {
        console.log("Wallet refresh timeout reached (15 minutes)");
        stopWalletRefresh();
        isWaitingForFunds = false;
      }
    }, 900000); // 15 minutes
  }

  // Stop periodic wallet refresh
  function stopWalletRefresh() {
    if (walletRefreshInterval) {
      console.log("Stopping periodic wallet refresh");
      clearInterval(walletRefreshInterval);
      walletRefreshInterval = null;
    }
    stopBlockMonitoring();
  }

  // Start monitoring block height changes
  async function startBlockMonitoring() {
    console.log("Starting block height monitoring");
    
    // Get initial block height
    try {
      lastCheckedBlockHeight = await invoke("get_block_count");
    } catch (error) {
      console.error("Failed to get initial block height:", error);
    }
    
    // Check block height every 10 seconds
    blockMonitorInterval = setInterval(async () => {
      try {
        const currentHeight = await invoke("get_block_count") as number;
        
        if (lastCheckedBlockHeight && currentHeight > lastCheckedBlockHeight) {
          console.log(`New block detected: ${currentHeight} (was ${lastCheckedBlockHeight})`);
          lastCheckedBlockHeight = currentHeight;
          
          // Trigger wallet refresh on new block
          console.log("Block change detected, refreshing wallet info");
          await checkFundingStatus();
        }
      } catch (error) {
        console.error("Failed to check block height:", error);
      }
    }, 10000); // Check every 10 seconds
  }

  // Stop block monitoring
  function stopBlockMonitoring() {
    if (blockMonitorInterval) {
      console.log("Stopping block height monitoring");
      clearInterval(blockMonitorInterval);
      blockMonitorInterval = null;
    }
    lastCheckedBlockHeight = null;
  }

  // Check funding status when component mounts
  $effect(() => {
    if (connectionState.current?.isConnected) {
      checkFundingStatus();
    }
  });

  // Load addresses when appropriate steps become active
  $effect(() => {
    if (currentStep === 'register') {
      loadControlAddresses();
    } else if (currentStep === 'completing') {
      loadPrimaryAddresses();
    }
  });

  // Cleanup intervals on component unmount
  $effect(() => {
    return () => {
      stopWalletRefresh();
      stopBlockMonitoring();
    };
  });
</script>

<div class="min-h-screen max-h-screen overflow-y-auto bg-verusidx-sky-soft dark:bg-verusidx-lake-deep">
  <!-- Block Height Header -->
  <BlockHeightHeader />
  
  <div class="max-w-4xl mx-auto p-8">
    <!-- Header -->
    <div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl p-6 mb-8">
      <div class="flex justify-between items-center">
        <div>
          <h1 class="text-3xl font-bold text-verusidx-stone-dark dark:text-white">Create {UI_CURRENCY_DISPLAY} Sub-Identity</h1>
          <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Get access by creating a {UI_CURRENCY_DISPLAY} Sub-ID</p>
        </div>
        <button 
          onclick={disconnectAndGoHome}
          class="px-4 py-2 bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-stone-medium dark:text-verusidx-mountain-mist rounded-lg hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-dark transition-colors"
        >
          Back to Home
        </button>
      </div>
    </div>

    {#if errorMessage}
      <div class="bg-verusidx-lake-deep/10 dark:bg-verusidx-lake-deep/30 border border-verusidx-turquoise-light dark:border-verusidx-stone-medium rounded-lg p-4 mb-8">
        <p class="text-verusidx-lake-deep dark:text-verusidx-turquoise-light">{errorMessage}</p>
      </div>
    {/if}

    {#if isLoading}
      <div class="text-center py-12">
        <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Checking your wallet<span class="dots-animation">...</span></p>
      </div>
    {:else}
      <!-- Step Indicator -->
      <div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl p-6 mb-8">
        <div class="flex items-center justify-center space-x-4">
          <div class="flex items-center">
            <div class="h-8 w-8 rounded-full {currentStep === 'check' ? 'bg-verusidx-mountain-blue text-white' : 'bg-verusidx-forest-deep text-white'} flex items-center justify-center text-sm font-medium">
              {currentStep === 'check' ? '1' : '✓'}
            </div>
            <span class="ml-2 text-sm font-medium text-verusidx-stone-dark dark:text-white">Check Funding</span>
          </div>
          <div class="h-0.5 w-12 bg-verusidx-mountain-mist"></div>
          <div class="flex items-center">
            <div class="h-8 w-8 rounded-full {currentStep === 'fund' ? 'bg-verus-600 text-white' : (currentStep === 'register' || currentStep === 'waiting' || currentStep === 'completing') ? 'bg-verusidx-forest-deep text-white' : 'bg-verusidx-mountain-mist text-verusidx-mountain-grey'} flex items-center justify-center text-sm font-medium">
              {fundingStatus.hasEnoughVerusIDX ? '✓' : '2'}
            </div>
            <span class="ml-2 text-sm font-medium text-verusidx-stone-dark dark:text-white">Get {UI_CURRENCY_DISPLAY}</span>
          </div>
          <div class="h-0.5 w-12 bg-verusidx-mountain-mist"></div>
          <div class="flex items-center">
            <div class="h-8 w-8 rounded-full {currentStep === 'register' ? 'bg-verus-600 text-white ring-2 ring-verusidx-turquoise-bright ring-offset-2 animate-pulse' : currentStep === 'waiting' ? 'bg-verusidx-turquoise-bright text-white' : currentStep === 'completing' ? 'bg-verusidx-mountain-blue text-white' : currentStep === 'finalizing' ? 'bg-verusidx-forest-deep text-white' : 'bg-verusidx-mountain-mist text-verusidx-mountain-grey'} flex items-center justify-center text-sm font-medium transition-all duration-300">
              {currentStep === 'waiting' ? '⏳' : currentStep === 'completing' ? '🔄' : currentStep === 'finalizing' ? '✅' : '3'}
            </div>
            <span class="ml-2 text-sm font-medium text-verusidx-stone-dark dark:text-white {currentStep === 'register' || currentStep === 'completing' ? 'text-verusidx-turquoise-deep dark:text-verusidx-turquoise-bright font-semibold' : ''}">
              {currentStep === 'waiting' ? 'Waiting for Block' : currentStep === 'completing' ? 'Complete Registration' : currentStep === 'finalizing' ? 'Finalizing...' : 'Register Sub-ID'}
            </span>
          </div>
        </div>
        
        {#if currentStep === 'waiting' && commitmentBlockHeight && currentBlockHeight}
          <div class="mt-4 text-center text-sm text-verusidx-mountain-grey">
            Block {currentBlockHeight} / {commitmentBlockHeight + 1} (waiting for confirmation)
          </div>
        {/if}
      </div>

      {#if showFundsReceivedNotification}
        <div class="bg-verusidx-forest-deep/10 dark:bg-verusidx-turquoise-deep/20 border border-verusidx-turquoise-light dark:border-verusidx-turquoise-bright rounded-lg p-4 mb-8 animate-pulse">
          <div class="flex items-center">
            <svg class="w-6 h-6 text-verusidx-forest-deep dark:text-verusidx-turquoise-light mr-2" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"></path>
            </svg>
            <span class="text-verusidx-forest-deep dark:text-verusidx-turquoise-light font-semibold">
              Funds received! You can now register your Sub-ID.
            </span>
          </div>
        </div>
      {/if}

      <!-- Funding Status -->
      <div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl p-6 mb-8">
        <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-4">Funding Status</h3>
        <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-4">
          Sub-identity registration costs {fundingStatus.requiredVerusIDX.toFixed(8)} {UI_CURRENCY_DISPLAY}
          (equivalent to {VERUSIDX_COST_IN_VUSDC} vUSDC.vETH)
        </p>
        
        {#if isWaitingForFunds && !fundingStatus.hasEnoughVerusIDX}
          <div class="mb-4 p-4 bg-verusidx-turquoise-bright/10 dark:bg-verusidx-lake-deep/50 border border-verusidx-turquoise-light dark:border-verusidx-turquoise-light rounded-lg">
            <div class="flex items-center justify-between">
              <div>
                <span class="text-verusidx-turquoise-deep dark:text-verusidx-turquoise-light font-medium">
                  Conversion processing<span class="dots-animation">...</span>
                </span>
                <p class="text-sm text-verusidx-turquoise-deep dark:text-verusidx-mountain-mist mt-1">
                  Waiting for your funds to be confirmed on the blockchain.
                </p>
              </div>
              <button
                onclick={() => {
                  console.log("Manual refresh requested");
                  checkFundingStatus();
                }}
                class="px-3 py-1 bg-verusidx-mountain-blue dark:bg-verusidx-turquoise-deep text-white text-sm rounded hover:bg-verusidx-lake-blue dark:hover:bg-verusidx-turquoise-bright transition-colors"
              >
                Refresh
              </button>
            </div>
          </div>
        {/if}
        
        <div class="space-y-4">
          <!-- VerusIDX Balance -->
          <div class="flex justify-between items-center p-4 border rounded-lg {fundingStatus.hasEnoughVerusIDX ? 'border-verusidx-turquoise-light bg-verusidx-forest-deep/10' : 'border-verusidx-mountain-mist'}">
            <div>
              <div class="font-medium text-verusidx-stone-dark dark:text-white">{UI_CURRENCY_DISPLAY}</div>
              <div class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Current: {fundingStatus.currentVerusIDX.toFixed(8)}</div>
              <div class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Required: {fundingStatus.requiredVerusIDX.toFixed(8)}</div>
            </div>
            <div class="text-sm">
              {#if fundingStatus.hasEnoughVerusIDX}
                <span class="px-3 py-1 bg-verusidx-forest-deep/20 text-verusidx-forest-deep rounded">✓ Sufficient</span>
              {:else}
                <span class="px-3 py-1 bg-verusidx-lake-deep/20 text-verusidx-lake-deep rounded">✗ Insufficient</span>
              {/if}
            </div>
          </div>

          <!-- VUSDC Balance -->
          <div class="flex justify-between items-center p-4 border rounded-lg {fundingStatus.hasEnoughVUSDC ? 'border-verusidx-turquoise-light bg-verusidx-forest-deep/10' : 'border-verusidx-mountain-mist'}">
            <div>
              <div class="font-medium text-verusidx-stone-dark dark:text-white">vUSDC.vETH</div>
              <div class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Current: {fundingStatus.currentVUSDC.toFixed(8)}</div>
              <div class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Required: {VERUSIDX_COST_IN_VUSDC}</div>
            </div>
            <div class="text-sm flex items-center space-x-2">
              {#if fundingStatus.hasEnoughVUSDC}
                <span class="px-3 py-1 bg-verusidx-forest-deep/20 text-verusidx-forest-deep rounded">✓ Sufficient</span>
              {:else}
                <span class="px-3 py-1 bg-verusidx-lake-deep/20 text-verusidx-lake-deep rounded">✗ Insufficient</span>
              {/if}
              {#if !fundingStatus.hasEnoughVerusIDX && fundingStatus.hasEnoughVUSDC}
                <button
                  onclick={() => showConversionModal('vusdc.veth')}
                  disabled={isProcessing}
                  class="px-3 py-1 bg-verusidx-mountain-blue dark:bg-verusidx-turquoise-deep text-white text-xs rounded hover:bg-verusidx-lake-blue dark:hover:bg-verusidx-turquoise-bright disabled:opacity-50 transition-colors"
                >
                  Convert
                </button>
              {/if}
            </div>
          </div>

          <!-- VRSC Balance -->
          <div class="flex justify-between items-center p-4 border rounded-lg {fundingStatus.hasEnoughVRSC ? 'border-verusidx-turquoise-light bg-verusidx-forest-deep/10' : 'border-verusidx-mountain-mist'}">
            <div>
              <div class="font-medium text-verusidx-stone-dark dark:text-white">VRSC</div>
              <div class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Current: {fundingStatus.currentVRSC.toFixed(8)}</div>
              <div class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Required: {fundingStatus.requiredVRSC.toFixed(8)}</div>
            </div>
            <div class="text-sm flex items-center space-x-2">
              {#if fundingStatus.hasEnoughVRSC}
                <span class="px-3 py-1 bg-verusidx-forest-deep/20 text-verusidx-forest-deep rounded">✓ Sufficient</span>
              {:else}
                <span class="px-3 py-1 bg-verusidx-lake-deep/20 text-verusidx-lake-deep rounded">✗ Insufficient</span>
              {/if}
              {#if !fundingStatus.hasEnoughVerusIDX && !fundingStatus.hasEnoughVUSDC && fundingStatus.hasEnoughVRSC}
                <button
                  onclick={() => showConversionModal('VRSC')}
                  disabled={isProcessing}
                  class="px-3 py-1 bg-verusidx-mountain-blue dark:bg-verusidx-turquoise-deep text-white text-xs rounded hover:bg-verusidx-lake-blue dark:hover:bg-verusidx-turquoise-bright disabled:opacity-50 transition-colors"
                >
                  Convert
                </button>
              {/if}
            </div>
          </div>
        </div>
      </div>

      <!-- Sub-Identity Registration -->
      {#if currentStep === 'register'}
        <div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl p-6">
          <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-4">Register Your Sub-Identity</h3>
          <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-4">
            Choose a name for your {UI_CURRENCY_DISPLAY} subID. Names can include letters, numbers, spaces, and emojis.
          </p>
          
          <div class="space-y-4">
            <div>
              <label for="subid-name" class="block text-sm font-medium text-verusidx-stone-medium dark:text-verusidx-mountain-mist mb-1">
                Sub-Identity Name
              </label>
              <input
                type="text"
                id="subid-name"
                bind:value={selectedSubIdName}
                placeholder="yourname"
                class="w-full px-3 py-2 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium bg-white dark:bg-verusidx-stone-medium text-verusidx-stone-dark dark:text-white rounded-lg focus:ring-2 focus:ring-verusidx-mountain-blue dark:focus:ring-verusidx-turquoise-deep focus:border-transparent outline-none"
              />
              <p class="mt-1 text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
                Cannot contain: \ / : * ? " &lt; &gt; | @ (e.g., "trader", "my name", "name 🚀")
              </p>
            </div>
            
            <div>
              <label for="control-address" class="block text-sm font-medium text-verusidx-stone-medium dark:text-verusidx-mountain-mist mb-1">
                Control Address
              </label>
              {#if isLoadingControlAddresses}
                <div class="w-full px-3 py-2 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium bg-white dark:bg-verusidx-stone-medium text-verusidx-stone-dark dark:text-white rounded-lg">
                  <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Loading addresses...</span>
                </div>
              {:else if controlAddresses.length > 0}
                <select
                  id="control-address"
                  bind:value={selectedControlAddress}
                  class="w-full px-3 py-2 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium bg-white dark:bg-verusidx-stone-medium text-verusidx-stone-dark dark:text-white rounded-lg focus:ring-2 focus:ring-verusidx-mountain-blue dark:focus:ring-verusidx-turquoise-deep focus:border-transparent outline-none"
                >
                  <option value="">Select control address</option>
                  {#each controlAddresses as address}
                    <option value={address}>{address}</option>
                  {/each}
                </select>
                <p class="mt-1 text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
                  This address will control the name commitment
                </p>
              {:else}
                <p class="text-sm text-verusidx-lake-deep">No addresses found. Loading addresses...</p>
              {/if}
            </div>
            
            
            <button
              onclick={registerSubIdentity}
              disabled={isProcessing || !selectedSubIdName.trim() || !selectedControlAddress}
              class="w-full px-6 py-3 bg-verusidx-mountain-blue dark:bg-verusidx-turquoise-deep text-white rounded-lg hover:bg-verusidx-lake-blue dark:hover:bg-verusidx-turquoise-bright focus:ring-2 focus:ring-verusidx-mountain-blue dark:focus:ring-verusidx-turquoise-deep focus:ring-offset-2 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isProcessing ? 'Submitting Commitment...' : 'Submit Name Commitment'}
            </button>
          </div>
        </div>
      {/if}

      <!-- Waiting for Block Confirmation -->
      {#if currentStep === 'waiting'}
        <div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl p-6">
          <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-4">Waiting for Block Confirmation</h3>

          {#if commitmentResponse}
            <div class="bg-verusidx-lake-deep/10 dark:bg-verusidx-turquoise-deep/20 border border-verusidx-turquoise-light dark:border-verusidx-turquoise-deep rounded-lg p-4 mb-4">
              <p class="text-sm text-verusidx-stone-dark dark:text-white">
                💡 <span class="font-medium">Tip:</span> Save the commitment data below for your records. You'll need it if you need to recover this registration.
              </p>
            </div>

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
  txid: commitmentResponse.txid,
  namereservation: {
    version: commitmentResponse.namereservation?.version,
    name: commitmentResponse.namereservation?.name,
    parent: commitmentResponse.namereservation?.parent,
    salt: commitmentResponse.namereservation?.salt,
    referral: commitmentResponse.namereservation?.referral,
    nameid: commitmentResponse.namereservation?.nameid
  }
}, null, 2)}</pre>
              <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-2">
                Save this data for your records. You'll need it if you need to recover this registration.
              </p>
            </div>
          {/if}

          <div class="text-center py-8">
            <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-2">Name commitment submitted successfully!</p>
            <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
              Waiting for the next block to confirm before completing registration<span class="dots-animation">...</span>
            </p>
            {#if commitmentBlockHeight && currentBlockHeight}
              <p class="text-xs text-verusidx-mountain-mist mt-2">
                Current block: {currentBlockHeight} | Required: {commitmentBlockHeight + 1}
              </p>
            {/if}
          </div>
        </div>
      {/if}

      <!-- Completing Registration -->
      {#if currentStep === 'completing'}
        <div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl p-6">
          <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-4">Complete Identity Registration</h3>
          <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-4">
            Select the primary address for your identity. This address will be in control of your sub-identity.
          </p>
          
          <div class="space-y-4">
            <div>
              <label for="primary-address" class="block text-sm font-medium text-verusidx-stone-medium dark:text-verusidx-mountain-mist mb-1">
                Primary Address
              </label>  
              {#if isLoadingPrimaryAddresses}
                <div class="w-full px-3 py-2 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium bg-white dark:bg-verusidx-stone-medium text-verusidx-stone-dark dark:text-white rounded-lg">
                  <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Loading addresses...</span>
                </div>
              {:else if primaryAddresses.length > 0}
                <select
                  id="primary-address"
                  bind:value={selectedPrimaryAddress}
                  class="w-full px-3 py-2 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium bg-white dark:bg-verusidx-stone-medium text-verusidx-stone-dark dark:text-white rounded-lg focus:ring-2 focus:ring-verusidx-mountain-blue dark:focus:ring-verusidx-turquoise-deep focus:border-transparent outline-none"
                >
                  <option value="">Select primary address</option>
                  {#each primaryAddresses as address}
                    <option value={address}>{address}</option>
                  {/each}
                </select>
                <p class="mt-1 text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
                  This address will be the primary address for the identity
                </p>
              {:else}
                <p class="text-sm text-verusidx-lake-deep">No addresses found. Loading addresses...</p>
              {/if}
            </div>
            
            <button
              onclick={finalizeIdentityRegistration}
              disabled={isProcessing || !selectedPrimaryAddress}
              class="w-full px-6 py-3 bg-verusidx-mountain-blue dark:bg-verusidx-turquoise-deep text-white rounded-lg hover:bg-verusidx-lake-blue dark:hover:bg-verusidx-turquoise-bright focus:ring-2 focus:ring-verusidx-mountain-blue dark:focus:ring-verusidx-turquoise-deep focus:ring-offset-2 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isProcessing ? 'Registering Identity...' : 'Complete Identity Registration'}
            </button>
          </div>
        </div>
      {/if}

      <!-- Finalizing Registration -->
      {#if currentStep === 'finalizing'}
        <div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl p-6">
          <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-4">Registration Complete</h3>
          <div class="text-center py-8">
            <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-2">Waiting for ID registration to complete<span class="dots-animation">...</span></p>
            <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
              Your identity has been submitted. Waiting for blockchain confirmation before redirecting.
            </p>
            {#if registrationBlockHeight && currentBlockHeight}
              <p class="text-xs text-verusidx-mountain-mist mt-2">
                Current block: {currentBlockHeight} | Waiting for: {registrationBlockHeight + 1}
              </p>
            {/if}
          </div>
        </div>
      {/if}

      <!-- Insufficient Funds Message -->
      {#if !fundingStatus.hasEnoughVerusIDX && !fundingStatus.hasEnoughVUSDC && !fundingStatus.hasEnoughVRSC}
        <div class="bg-verusidx-turquoise-bright/10 dark:bg-verusidx-lake-deep/50 border border-verusidx-turquoise-light dark:border-verusidx-turquoise-light rounded-lg p-6">
          <h3 class="text-lg font-medium text-verusidx-turquoise-deep dark:text-verusidx-turquoise-light mb-2">Insufficient Funds</h3>
          <p class="text-verusidx-turquoise-deep dark:text-verusidx-mountain-mist mb-4">
            You need either {fundingStatus.requiredVerusIDX.toFixed(8)} {UI_CURRENCY_DISPLAY},
            {VERUSIDX_COST_IN_VUSDC} vUSDC.vETH, or {fundingStatus.requiredVRSC.toFixed(8)} VRSC
            to create a subID.
          </p>
          <p class="text-verusidx-turquoise-deep dark:text-verusidx-mountain-mist text-sm">
            Please acquire the necessary funds and return to this page to continue.
          </p>
        </div>
      {/if}
    {/if}

    <!-- Conversion Modal -->
    {#if showingConversionModal}
      <div class="fixed inset-0 bg-black/50 overflow-y-auto h-full w-full z-50 flex items-center justify-center"
           onclick={closeConversionModal}
           onkeydown={(e) => e.key === 'Escape' && closeConversionModal()}
           role="button"
           tabindex="0">
        <div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl p-6 max-w-md w-full mx-4"
             onclick={(e) => e.stopPropagation()}
             onkeydown={(e) => e.key === 'Escape' && closeConversionModal()}
             role="dialog">
          <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-4">
            Convert {conversionCurrency} to {UI_CURRENCY_DISPLAY}
          </h3>
          
          <div class="space-y-4 mb-6">
            <!-- From Address Selection -->
            <div>
              <label for="from-address" class="block text-sm font-medium text-verusidx-stone-medium dark:text-verusidx-mountain-mist mb-1">
                From Address
              </label>
              <select
                id="from-address"
                bind:value={selectedFromAddress}
                class="w-full px-3 py-2 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium bg-white dark:bg-verusidx-stone-medium text-verusidx-stone-dark dark:text-white rounded-lg focus:ring-2 focus:ring-verusidx-mountain-blue dark:focus:ring-verusidx-turquoise-deep focus:border-transparent outline-none"
              >
                <option value="">Select Address To Send From</option>
                <option value="*">* (Any transparent address)</option>
                <option value="R*">R* (Any R-address)</option>
                <option value="i*">i* (Any i-address)</option>
                {#each rAddresses as address}
                  <option value={address}>{address}</option>
                {/each}
              </select>
              <p class="mt-1 text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
                Choose specific address or use all available addresses
              </p>
            </div>
            
            <!-- To Address Selection -->
            <div>
              <label for="to-address" class="block text-sm font-medium text-verusidx-stone-medium dark:text-verusidx-mountain-mist mb-1">
                To Address (receiving address)
              </label>
              {#if isLoadingToAddresses}
                <div class="w-full px-3 py-2 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium bg-white dark:bg-verusidx-stone-medium text-verusidx-stone-dark dark:text-white rounded-lg">
                  <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Loading addresses...</span>
                </div>
              {:else if toAddresses.length > 0}
                <select
                  id="to-address"
                  bind:value={selectedToAddress}
                  class="w-full px-3 py-2 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium bg-white dark:bg-verusidx-stone-medium text-verusidx-stone-dark dark:text-white rounded-lg focus:ring-2 focus:ring-verusidx-mountain-blue dark:focus:ring-verusidx-turquoise-deep focus:border-transparent outline-none"
                >
                  <option value="">Select Receiving Address</option>
                  {#each toAddresses as address}
                    <option value={address}>{address}</option>
                  {/each}
                </select>
              {:else}
                <p class="text-sm text-verusidx-lake-deep dark:text-verusidx-turquoise-light">No addresses found. Loading addresses...</p>
              {/if}
            </div>
            
            <!-- Conversion Details -->
            <div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded-lg">
              <div class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
                <div class="flex justify-between">
                  <span>Converting:</span>
                  <span>{conversionCurrency === 'vusdc.veth' ? VERUSIDX_COST_IN_VUSDC : (fundingStatus.requiredVRSC > 0 ? fundingStatus.requiredVRSC.toFixed(8) : 'Loading...')} {conversionCurrency}</span>
                </div>
                <div class="flex justify-between">
                  <span>To:</span>
                  <span>~{fundingStatus.requiredVerusIDX.toFixed(8)} {UI_CURRENCY_DISPLAY}</span>
                </div>
              </div>
            </div>
          </div>
          
          {#if showSuccessMessage && conversionOperationId}
            <div class="mb-4 p-4 bg-verusidx-turquoise-bright/10 dark:bg-verusidx-lake-deep/50 border border-verusidx-turquoise-light dark:border-verusidx-turquoise-light rounded-lg relative">
              <button
                onclick={closeConversionModal}
                class="absolute top-2 right-2 rounded-md text-verusidx-turquoise-deep dark:text-verusidx-turquoise-light hover:text-verusidx-mountain-blue dark:hover:text-verusidx-turquoise-bright focus:outline-none focus:ring-2 focus:ring-verusidx-mountain-blue"
                aria-label="Close success message"
              >
                <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
              <div class="flex items-center mb-2">
                <svg class="w-5 h-5 text-verusidx-turquoise-deep dark:text-verusidx-turquoise-light mr-2" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"></path>
                </svg>
                <span class="font-medium text-verusidx-turquoise-deep dark:text-verusidx-turquoise-light">Conversion Submitted Successfully!</span>
              </div>
              <p class="text-sm text-verusidx-turquoise-deep dark:text-verusidx-mountain-mist">
                Your conversion will complete in 2-10 minutes.
              </p>
              <p class="text-sm text-verusidx-turquoise-deep dark:text-verusidx-mountain-mist mt-1">
                Operation ID: <span class="font-mono text-xs">{conversionOperationId}</span>
              </p>
              {#if operationStatus}
                <p class="text-sm text-verusidx-turquoise-deep dark:text-verusidx-mountain-mist mt-1">
                  Status: <span class="font-medium">{operationStatus.status}</span>
                  {#if operationStatus.result?.txid}
                    <br>Transaction: <span class="font-mono text-xs">{operationStatus.result.txid}</span>
                  {/if}
                </p>
              {/if}
            </div>
          {/if}
          
          <div class="flex space-x-3">
            <button
              onclick={closeConversionModal}
              class="flex-1 px-4 py-2 bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-stone-medium dark:text-verusidx-mountain-mist rounded-lg hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-dark transition-colors"
            >
              {showSuccessMessage ? 'Close' : 'Cancel'}
            </button>
            {#if !showSuccessMessage}
              <button
                onclick={executeConversion}
                disabled={isProcessing || !selectedToAddress || !selectedFromAddress}
                class="flex-1 px-4 py-2 bg-verusidx-mountain-blue dark:bg-verusidx-turquoise-deep text-white rounded-lg hover:bg-verusidx-lake-blue dark:hover:bg-verusidx-turquoise-bright transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isProcessing ? 'Converting...' : 'Convert'}
              </button>
            {/if}
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  @keyframes dots {
    0%, 20% {
      content: '.';
    }
    40% {
      content: '..';
    }
    60% {
      content: '...';
    }
    80% {
      content: '....';
    }
    100% {
      content: '.....';
    }
  }
  
  .dots-animation::after {
    content: '';
    animation: dots 2s infinite;
  }
</style>