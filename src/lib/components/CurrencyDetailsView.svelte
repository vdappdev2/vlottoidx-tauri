<script lang="ts">
  import IdentityModal from "./IdentityModal.svelte";

  interface Props {
    currencyData: any; // Response from getcurrency RPC command
    isLoading?: boolean;
    error?: string | null;
  }

  let { currencyData, isLoading = false, error = null }: Props = $props();

  // Collapsible section states
  let generalInfoExpanded = $state(false);
  let nativeCurrencyExpanded = $state(false);
  let supplyStateExpanded = $state(false);
  let currencyNamesExpanded = $state(false);
  let basketReserveExpanded = $state(false);
  let idFeeExpanded = $state(false);
  let currencyFeesExpanded = $state(false);
  let preconversionExpanded = $state(false);
  let multicurrencyPBaaSExpanded = $state(false);
  let gatewayInfoExpanded = $state(false);
  let blockchainExpanded = $state(false);

  // Identity modal state
  let showIdentityModal = $state(false);
  let selectedIdentityAddress = $state("");

  // State variables for processed data
  let generalInfo = $state<any>(null);
  let nativeCurrency = $state<any>(null);
  let supplyState = $state<any>(null);
  let currencyNames = $state<any>(null);
  let basketReserve = $state<any>(null);
  let idFee = $state<any>(null);
  let currencyFees = $state<any>(null);
  let preconversion = $state<any>(null);
  let multicurrencyPBaaS = $state<any>(null);
  let gatewayInfo = $state<any>(null);
  let blockchain = $state<any>(null);

  // Effect to process currencyData when it changes
  $effect(() => {
    if (!currencyData) {
      generalInfo = null;
      nativeCurrency = null;
      supplyState = null;
      currencyNames = null;
      basketReserve = null;
      idFee = null;
      currencyFees = null;
      preconversion = null;
      multicurrencyPBaaS = null;
      gatewayInfo = null;
      blockchain = null;
      return;
    }
    
    // Handle getcurrency response structure
    const currency = currencyData;
    
    // Process generalInfo
    const general = {
      name: currency?.fullyqualifiedname || currency?.name || 'Unknown',
      fullyqualifiedname: currency?.fullyqualifiedname || currency?.name || 'Unknown',
      currencyid: currency?.currencyid || 'Unknown',
      currencyidhex: currency?.currencyidhex || 'Unknown',
      parent: currency?.parent || 'Unknown',
      systemid: currency?.systemid || 'Unknown',
      launchsystemid: currency?.launchsystemid || 'Unknown',
      proofprotocol: currency?.proofprotocol || null,
      options: currency?.options || 0
    };
    
    // Process nativeCurrency
    const native = {
      nativecurrencyid: currency?.nativecurrencyid || null
    };
    
    // Process supplyState
    const supply = {
      initialsupply: currency?.initialsupply || 0,
      supply: currency?.bestcurrencystate?.supply || 0
    };
    
    // Process currencyNames
    const names = {
      currencynames: currency?.currencynames || null
    };
    
    // Process basketReserve
    const basket = {
      reservecurrencies: currency?.bestcurrencystate?.reservecurrencies || [],
      currencyid: currency?.bestcurrencystate?.currencyid || [],
      weight: currency?.bestcurrencystate?.weight || [],
      reserves: currency?.bestcurrencystate?.reserves || [],
      priceinreserve: currency?.bestcurrencystate?.priceinreserve || []
    };
    
    // Process idFee
    const fees = {
      idregistrationfees: currency?.idregistrationfees || 0,
      idreferrallevels: currency?.idreferrallevels || 0,
      idimportfees: currency?.idimportfees || 0
    };

    // Process multicurrencyPBaaS
    const pbaas = {
      currencyregistrationfee: currency?.currencyregistrationfee || null,
      pbaassystemregistrationfee: currency?.pbaassystemregistrationfee || null,
      currencyimportfee: currency?.currencyimportfee || null,
      transactionimportfee: currency?.transactionimportfee || null,
      transactionexportfee: currency?.transactionexportfee || null
    };

    // Process currencyFees
    const currFees = {
      primarycurrencyfees: currency?.bestcurrencystate?.primarycurrencyfees || null,
      primarycurrencyconversionfees: currency?.bestcurrencystate?.primarycurrencyconversionfees || null,
      primarycurrencyout: currency?.bestcurrencystate?.primarycurrencyout || null,
      preconvertedout: currency?.bestcurrencystate?.preconvertedout || null
    };
    
    // Process preconversion
    const preconv = {
      currencies: currency?.currencies || [],
      conversions: currency?.conversions || [],
      minpreconversion: currency?.minpreconversion || [],
      maxpreconversion: currency?.maxpreconversion || [],
      initialcontributions: currency?.initialcontributions || [],
      prelaunchcarveout: currency?.prelaunchcarveout || null,
      prelaunchdiscount: currency?.prelaunchdiscount || null,
      preallocations: currency?.preallocations || []
    };
    
    // Process gateway info
    const gateway = {
      gateway: currency?.gateway || null,
      gatewayconverterid: currency?.gatewayconverterid || null,
      gatewayconvertername: currency?.gatewayconvertername || null,
      gatewayconverterissuance: currency?.gatewayconverterissuance || null
    };
    
    // Process blockchain
    const blockchainInfo = {
      startblock: currency?.startblock || 0,
      endblock: currency?.endblock || 0,
      besttxid: currency?.besttxid || null,
      confirmedtxid: currency?.confirmedtxid || null,
      confirmednotarization: currency?.confirmednotarization || null,
      blockheight: currency?.blockheight || 0,
      txid: currency?.txid || null,
      vout: currency?.vout || 0,
      definitiontxid: currency?.definitiontxid || null,
      definitiontxout: currency?.definitiontxout || null,
      bestheight: currency?.bestheight || 0,
      lastconfirmedheight: currency?.lastconfirmedheight || 0,
      lastconfirmedtxid: currency?.lastconfirmedtxid || null,
      notarizationprotocol: currency?.notarizationprotocol || null,
      version: currency?.version || 0,
      magicnumber: currency?.magicnumber || null
    };
    
    generalInfo = general;
    nativeCurrency = native;
    supplyState = supply;
    currencyNames = names;
    basketReserve = basket;
    idFee = fees;
    currencyFees = currFees;
    preconversion = preconv;
    multicurrencyPBaaS = pbaas;
    gatewayInfo = gateway;
    blockchain = blockchainInfo;
  });

  function copyToClipboard(text: string) {
    navigator.clipboard.writeText(text);
  }

  function toggleSection(section: string) {
    switch (section) {
      case 'generalInfo':
        generalInfoExpanded = !generalInfoExpanded;
        break;
      case 'nativeCurrency':
        nativeCurrencyExpanded = !nativeCurrencyExpanded;
        break;
      case 'supplyState':
        supplyStateExpanded = !supplyStateExpanded;
        break;
      case 'currencyNames':
        currencyNamesExpanded = !currencyNamesExpanded;
        break;
      case 'basketReserve':
        basketReserveExpanded = !basketReserveExpanded;
        break;
      case 'idFee':
        idFeeExpanded = !idFeeExpanded;
        break;
      case 'currencyFees':
        currencyFeesExpanded = !currencyFeesExpanded;
        break;
      case 'preconversion':
        preconversionExpanded = !preconversionExpanded;
        break;
      case 'multicurrencyPBaaS':
        multicurrencyPBaaSExpanded = !multicurrencyPBaaSExpanded;
        break;
      case 'gatewayInfo':
        gatewayInfoExpanded = !gatewayInfoExpanded;
        break;
      case 'blockchain':
        blockchainExpanded = !blockchainExpanded;
        break;
    }
  }

  // Helper functions to determine when sections should be displayed
  function shouldShowNativeCurrency(): boolean {
    return nativeCurrency && nativeCurrency.nativecurrencyid !== null;
  }

  function shouldShowSupplyState(): boolean {
    return supplyState && (
      supplyState.initialsupply > 0 || 
      supplyState.supply > 0 || 
      supplyState.lastconfirmedcurrencystate || 
      supplyState.bestcurrencystate
    );
  }

  function shouldShowCurrencyNames(): boolean {
    return currencyNames && currencyNames.currencynames && 
           Object.keys(currencyNames.currencynames).length > 0;
  }

  function shouldShowBasketReserve(): boolean {
    return basketReserve && (
      basketReserve.reservecurrencies.length > 0 || 
      basketReserve.currencyid.length > 0 ||
      basketReserve.weight.length > 0 ||
      basketReserve.reserves.length > 0 ||
      basketReserve.priceinreserve.length > 0
    );
  }

  function shouldShowIdFee(): boolean {
    return idFee && (
      idFee.idregistrationfees > 0 || 
      idFee.idreferrallevels > 0 || 
      idFee.idimportfees > 0
    );
  }

  function shouldShowCurrencyFees(): boolean {
    return currencyFees && (
      currencyFees.primarycurrencyfees !== null ||
      currencyFees.primarycurrencyconversionfees !== null ||
      currencyFees.primarycurrencyout !== null ||
      currencyFees.preconvertedout !== null
    );
  }

  function shouldShowPreconversion(): boolean {
    return preconversion && (
      preconversion.currencies.length > 0 ||
      preconversion.conversions.length > 0 ||
      preconversion.minpreconversion.length > 0 ||
      preconversion.maxpreconversion.length > 0 ||
      preconversion.initialcontributions.length > 0 ||
      preconversion.prelaunchcarveout !== null ||
      preconversion.prelaunchdiscount !== null ||
      preconversion.preallocations.length > 0
    );
  }

  function shouldShowMulticurrencyPBaaS(): boolean {
    return multicurrencyPBaaS && (
      multicurrencyPBaaS.currencyregistrationfee !== null ||
      multicurrencyPBaaS.pbaassystemregistrationfee !== null ||
      multicurrencyPBaaS.currencyimportfee !== null ||
      multicurrencyPBaaS.transactionimportfee !== null ||
      multicurrencyPBaaS.transactionexportfee !== null
    );
  }
  
  function shouldShowGatewayInfo(): boolean {
    return gatewayInfo && (
      gatewayInfo.gateway !== null ||
      gatewayInfo.gatewayconverterid !== null ||
      gatewayInfo.gatewayconvertername !== null ||
      gatewayInfo.gatewayconverterissuance !== null
    );
  }

  function shouldShowBlockchain(): boolean {
    return blockchain && (
      blockchain.besttxid !== null ||
      blockchain.confirmedtxid !== null ||
      blockchain.definitiontxid !== null ||
      blockchain.blockheight > 0 ||
      blockchain.bestheight > 0 ||
      blockchain.lastconfirmedheight > 0
    );
  }

  function formatInteger(num: number): string {
    return Math.floor(num).toLocaleString();
  }

  function formatCurrency(num: number): string {
    return num.toFixed(8);
  }

  // Helper function to get friendly name from currencynames object
  function getCurrencyFriendlyName(currencyId: string): string {
    if (currencyNames?.currencynames && currencyNames.currencynames[currencyId]) {
      return currencyNames.currencynames[currencyId];
    }
    return currencyId; // fallback to currency id if no friendly name found
  }

  // Identity modal handlers
  function openIdentityModal(identityAddress: string) {
    selectedIdentityAddress = identityAddress;
    showIdentityModal = true;
    // Scroll to top for better modal visibility
    window.scrollTo({ top: 0, behavior: 'smooth' });
  }

  function closeIdentityModal() {
    showIdentityModal = false;
    selectedIdentityAddress = "";
  }

</script>

<div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl">
  {#if isLoading}
    <div class="p-6">
      <div class="animate-pulse">
        <div class="h-4 bg-verusidx-mountain-mist rounded w-3/4 mb-4"></div>
        <div class="h-4 bg-verusidx-mountain-mist rounded w-1/2 mb-4"></div>
        <div class="h-4 bg-verusidx-mountain-mist rounded w-5/6"></div>
      </div>
    </div>
  {:else if error}
    <div class="p-6">
      <div class="text-center text-verusidx-stone-dark dark:text-verusidx-mountain-mist">
        <p class="text-sm text-red-600 dark:text-red-400">{error}</p>
      </div>
    </div>
  {:else if !currencyData}
    <div class="p-6">
      <div class="text-center text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
        <p class="text-sm">No currency data available</p>
      </div>
    </div>
  {:else}
    <!-- General Info Section -->
    <div class="border-b border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
      <button
        class="w-full p-4 flex items-center justify-between hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors"
        onclick={() => toggleSection('generalInfo')}
      >
        <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">General Info</h3>
        <svg
          class="h-5 w-5 text-verusidx-mountain-grey transition-transform duration-200 {generalInfoExpanded ? 'rotate-180' : ''}"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
        </svg>
      </button>
      {#if generalInfoExpanded && generalInfo}
        <div class="px-4 pb-4 space-y-3">
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Name:</span>
            <span class="font-medium text-verusidx-stone-dark dark:text-white">{generalInfo.name}</span>
          </div>
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Fully Qualified Name:</span>
            <span class="font-medium text-verusidx-stone-dark dark:text-white">{generalInfo.fullyqualifiedname}</span>
          </div>
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Currency ID:</span>
            <div class="flex items-center space-x-2">
              <span class="font-medium text-verusidx-stone-dark dark:text-white text-sm font-mono">{generalInfo.currencyid}</span>
              <button
                onclick={() => copyToClipboard(generalInfo.currencyid)}
                class="p-1 hover:bg-verusidx-mountain-mist rounded text-verusidx-mountain-grey hover:text-verusidx-stone-dark"
                title="Copy currency ID"
              >
                <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                </svg>
              </button>
            </div>
          </div>
          {#if generalInfo.currencyidhex !== 'Unknown'}
            <div class="flex justify-between items-center">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Currency ID Hex:</span>
              <div class="flex items-center space-x-2">
                <span class="font-medium text-verusidx-stone-dark dark:text-white text-sm font-mono">{generalInfo.currencyidhex}</span>
                <button
                  onclick={() => copyToClipboard(generalInfo.currencyidhex)}
                  class="p-1 hover:bg-verusidx-mountain-mist rounded text-verusidx-mountain-grey hover:text-verusidx-stone-dark"
                  title="Copy currency ID hex"
                >
                  <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                  </svg>
                </button>
              </div>
            </div>
          {/if}
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Parent:</span>
            <span class="font-medium text-verusidx-stone-dark dark:text-white text-sm font-mono">{generalInfo.parent}</span>
          </div>
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">System ID:</span>
            <span class="font-medium text-verusidx-stone-dark dark:text-white text-sm font-mono">{generalInfo.systemid}</span>
          </div>
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Launch System ID:</span>
            <span class="font-medium text-verusidx-stone-dark dark:text-white text-sm font-mono">{generalInfo.launchsystemid}</span>
          </div>
          {#if generalInfo.proofprotocol !== null}
            <div class="flex justify-between items-center">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Proof Protocol:</span>
              <span class="font-medium text-verusidx-stone-dark dark:text-white">{formatInteger(generalInfo.proofprotocol)}</span>
            </div>
          {/if}
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Options:</span>
            <span class="font-medium text-verusidx-stone-dark dark:text-white">{formatInteger(generalInfo.options)}</span>
          </div>
        </div>
      {/if}
    </div>

    <!-- Native Currency Section -->
    {#if shouldShowNativeCurrency()}
    <div class="border-b border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
      <button
        class="w-full p-4 flex items-center justify-between hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors"
        onclick={() => toggleSection('nativeCurrency')}
      >
        <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">Native Currency</h3>
        <svg
          class="h-5 w-5 text-verusidx-mountain-grey transition-transform duration-200 {nativeCurrencyExpanded ? 'rotate-180' : ''}"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
        </svg>
      </button>
      {#if nativeCurrencyExpanded && nativeCurrency}
        <div class="px-4 pb-4 space-y-3">
          {#if nativeCurrency.nativecurrencyid}
            {@const address = typeof nativeCurrency.nativecurrencyid === 'object' 
              ? nativeCurrency.nativecurrencyid.address 
              : nativeCurrency.nativecurrencyid}
            {@const type = typeof nativeCurrency.nativecurrencyid === 'object' && 'type' in nativeCurrency.nativecurrencyid
              ? nativeCurrency.nativecurrencyid.type
              : null}
            <div class="flex justify-between items-center">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Address:</span>
              <div class="flex items-center space-x-2">
                <span class="font-medium text-verusidx-stone-dark dark:text-white text-sm font-mono">{address}</span>
                <button
                  onclick={() => copyToClipboard(address)}
                  class="p-1 hover:bg-verusidx-mountain-mist rounded text-verusidx-mountain-grey hover:text-verusidx-stone-dark"
                  title="Copy native currency ID"
                >
                  <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                  </svg>
                </button>
              </div>
            </div>
            {#if type}
              <div class="flex justify-between items-center">
                <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Type:</span>
                <span class="font-medium text-verusidx-stone-dark dark:text-white">{type}</span>
              </div>
            {/if}
          {/if}
        </div>
      {/if}
    </div>
    {/if}

    <!-- Supply & State Section -->
    {#if shouldShowSupplyState()}
    <div class="border-b border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
      <button
        class="w-full p-4 flex items-center justify-between hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors"
        onclick={() => toggleSection('supplyState')}
      >
        <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">Supply & State</h3>
        <svg
          class="h-5 w-5 text-verusidx-mountain-grey transition-transform duration-200 {supplyStateExpanded ? 'rotate-180' : ''}"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
        </svg>
      </button>
      {#if supplyStateExpanded && supplyState}
        <div class="px-4 pb-4 space-y-3">
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Initial Supply:</span>
            <span class="font-medium text-verusidx-stone-dark dark:text-white">{formatCurrency(supplyState.initialsupply)}</span>
          </div>
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Current Supply:</span>
            <span class="font-medium text-verusidx-stone-dark dark:text-white">{formatCurrency(supplyState.supply)}</span>
          </div>
          {#if supplyState.lastconfirmedcurrencystate}
            <div class="mt-3">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Last Confirmed State:</span>
              <div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded text-sm mt-2">
                <pre class="text-verusidx-stone-dark dark:text-white whitespace-pre-wrap">{JSON.stringify(supplyState.lastconfirmedcurrencystate, null, 2)}</pre>
              </div>
            </div>
          {/if}
        </div>
      {/if}
    </div>
    {/if}

    <!-- Currency Names Section -->
    {#if shouldShowCurrencyNames()}
    <div class="border-b border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
      <button
        class="w-full p-4 flex items-center justify-between hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors"
        onclick={() => toggleSection('currencyNames')}
      >
        <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">Currency Names</h3>
        <svg
          class="h-5 w-5 text-verusidx-mountain-grey transition-transform duration-200 {currencyNamesExpanded ? 'rotate-180' : ''}"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
        </svg>
      </button>
      {#if currencyNamesExpanded && currencyNames && basketReserve}
        <div class="px-4 pb-4 space-y-3">
          {#if basketReserve.reservecurrencies.length > 0}
            <!-- Use the same order as basket/reserve info -->
            {#each basketReserve.reservecurrencies as currency}
              {@const friendlyName = getCurrencyFriendlyName(currency.currencyid)}
              <div class="flex justify-between items-center bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-2 rounded">
                <span class="text-sm font-medium text-verusidx-stone-dark dark:text-white">{friendlyName}</span>
                <span class="font-mono text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">{currency.currencyid}</span>
              </div>
            {/each}
          {:else}
            <!-- Fallback to original method if no reserve currencies -->
            {#each Object.entries(currencyNames.currencynames) as [currencyId, name]}
              <div class="flex justify-between items-center bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-2 rounded">
                <span class="text-sm font-medium text-verusidx-stone-dark dark:text-white">{name}</span>
                <span class="font-mono text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">{currencyId}</span>
              </div>
            {/each}
          {/if}
        </div>
      {/if}
    </div>
    {/if}

    <!-- Basket/Reserve Info Section -->
    {#if shouldShowBasketReserve()}
    <div class="border-b border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
      <button
        class="w-full p-4 flex items-center justify-between hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors"
        onclick={() => toggleSection('basketReserve')}
      >
        <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">Basket/Reserve Info</h3>
        <svg
          class="h-5 w-5 text-verusidx-mountain-grey transition-transform duration-200 {basketReserveExpanded ? 'rotate-180' : ''}"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
        </svg>
      </button>
      {#if basketReserveExpanded && basketReserve}
        <div class="px-4 pb-4">
          {#if basketReserve.reservecurrencies.length > 0}
            <div class="overflow-x-auto">
              <table class="w-full text-sm">
                <thead>
                  <tr class="border-b border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
                    <th class="text-left py-2 px-3 font-medium text-verusidx-stone-dark dark:text-white">Currency ID</th>
                    <th class="text-right py-2 px-3 font-medium text-verusidx-stone-dark dark:text-white">Weight</th>
                    <th class="text-right py-2 px-3 font-medium text-verusidx-stone-dark dark:text-white">Reserves</th>
                    <th class="text-right py-2 px-3 font-medium text-verusidx-stone-dark dark:text-white">Price in Reserve</th>
                  </tr>
                </thead>
                <tbody>
                  {#each basketReserve.reservecurrencies as currency, index}
                    {@const weight = currency.weight || 0}
                    {@const reserves = currency.reserves || 0}
                    {@const price = currency.priceinreserve || 0}
                    <tr class="border-b border-verusidx-mountain-mist/50 dark:border-verusidx-stone-medium/50 {index % 2 === 0 ? 'bg-verusidx-sky-soft/30 dark:bg-verusidx-stone-medium/30' : ''}">
                      <td class="py-2 px-3">
                        <div class="flex flex-col">
                          <span class="font-medium text-verusidx-stone-dark dark:text-white">{getCurrencyFriendlyName(currency.currencyid)}</span>
                          <span class="text-xs font-mono text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">{currency.currencyid}</span>
                        </div>
                      </td>
                      <td class="py-2 px-3 text-right font-mono text-verusidx-stone-dark dark:text-white">{formatCurrency(weight)}</td>
                      <td class="py-2 px-3 text-right font-mono text-verusidx-stone-dark dark:text-white">{formatCurrency(reserves)}</td>
                      <td class="py-2 px-3 text-right font-mono text-verusidx-stone-dark dark:text-white">{formatCurrency(price)}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          {:else}
            <div class="text-center py-4">
              <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">No reserve currency data available</p>
            </div>
          {/if}
        </div>
      {/if}
    </div>
    {/if}

    <!-- ID/Fee Information Section -->
    {#if shouldShowIdFee()}
    <div class="border-b border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
      <button
        class="w-full p-4 flex items-center justify-between hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors"
        onclick={() => toggleSection('idFee')}
      >
        <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">ID/Fee Information</h3>
        <svg
          class="h-5 w-5 text-verusidx-mountain-grey transition-transform duration-200 {idFeeExpanded ? 'rotate-180' : ''}"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
        </svg>
      </button>
      {#if idFeeExpanded && idFee}
        <div class="px-4 pb-4 space-y-3">
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">ID Registration Fees:</span>
            <span class="font-medium text-verusidx-stone-dark dark:text-white">{formatCurrency(idFee.idregistrationfees)}</span>
          </div>
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">ID Referral Levels:</span>
            <span class="font-medium text-verusidx-stone-dark dark:text-white">{formatInteger(idFee.idreferrallevels)}</span>
          </div>
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">ID Import Fees:</span>
            <span class="font-medium text-verusidx-stone-dark dark:text-white">{formatCurrency(idFee.idimportfees)}</span>
          </div>
        </div>
      {/if}
    </div>
    {/if}


    <!-- Preconversion Section -->
    {#if shouldShowPreconversion()}
    <div class="border-b border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
      <button
        class="w-full p-4 flex items-center justify-between hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors"
        onclick={() => toggleSection('preconversion')}
      >
        <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">Preconversion</h3>
        <svg
          class="h-5 w-5 text-verusidx-mountain-grey transition-transform duration-200 {preconversionExpanded ? 'rotate-180' : ''}"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
        </svg>
      </button>
      {#if preconversionExpanded && preconversion}
        <div class="px-4 pb-4 space-y-4">
          <!-- Currency Data Table -->
          {#if preconversion.currencies.length > 0}
            <div>
              <h4 class="font-medium text-verusidx-stone-dark dark:text-white mb-3">Currency Data</h4>
              <div class="overflow-x-auto">
                <table class="w-full text-sm">
                  <thead>
                    <tr class="border-b border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
                      <th class="text-left py-2 px-3 font-medium text-verusidx-stone-dark dark:text-white">Currency</th>
                      <th class="text-right py-2 px-3 font-medium text-verusidx-stone-dark dark:text-white">Conversions</th>
                      <th class="text-right py-2 px-3 font-medium text-verusidx-stone-dark dark:text-white">Min Preconversion</th>
                      <th class="text-right py-2 px-3 font-medium text-verusidx-stone-dark dark:text-white">Max Preconversion</th>
                      <th class="text-right py-2 px-3 font-medium text-verusidx-stone-dark dark:text-white">Initial Contributions</th>
                    </tr>
                  </thead>
                  <tbody>
                    {#each preconversion.currencies as currency, index}
                      {@const conversion = preconversion.conversions[index] || 0}
                      {@const minPreconv = preconversion.minpreconversion[index] || 0}
                      {@const maxPreconv = preconversion.maxpreconversion[index] || 0}
                      {@const initialContrib = preconversion.initialcontributions[index] || 0}
                      <tr class="border-b border-verusidx-mountain-mist/50 dark:border-verusidx-stone-medium/50 {index % 2 === 0 ? 'bg-verusidx-sky-soft/30 dark:bg-verusidx-stone-medium/30' : ''}">
                        <td class="py-2 px-3">
                          <div class="flex flex-col">
                            <span class="font-medium text-verusidx-stone-dark dark:text-white">{getCurrencyFriendlyName(currency)}</span>
                            <span class="text-xs font-mono text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">{currency}</span>
                          </div>
                        </td>
                        <td class="py-2 px-3 text-right font-mono text-verusidx-stone-dark dark:text-white">{formatCurrency(conversion)}</td>
                        <td class="py-2 px-3 text-right font-mono text-verusidx-stone-dark dark:text-white">{formatCurrency(minPreconv)}</td>
                        <td class="py-2 px-3 text-right font-mono text-verusidx-stone-dark dark:text-white">{formatCurrency(maxPreconv)}</td>
                        <td class="py-2 px-3 text-right font-mono text-verusidx-stone-dark dark:text-white">{formatCurrency(initialContrib)}</td>
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>
            </div>
          {/if}

          <!-- Prelaunch Settings -->
          {#if preconversion.prelaunchcarveout !== null || preconversion.prelaunchdiscount !== null}
            <div>
              <h4 class="font-medium text-verusidx-stone-dark dark:text-white mb-3">Prelaunch Settings</h4>
              <div class="space-y-2">
                {#if preconversion.prelaunchcarveout !== null}
                  <div class="flex justify-between items-center">
                    <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Prelaunch Carveout:</span>
                    <span class="font-mono text-verusidx-stone-dark dark:text-white">{formatCurrency(preconversion.prelaunchcarveout)}</span>
                  </div>
                {/if}
                {#if preconversion.prelaunchdiscount !== null}
                  <div class="flex justify-between items-center">
                    <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Prelaunch Discount:</span>
                    <span class="font-mono text-verusidx-stone-dark dark:text-white">{formatCurrency(preconversion.prelaunchdiscount)}</span>
                  </div>
                {/if}
              </div>
            </div>
          {/if}

          <!-- Preallocations Table -->
          {#if preconversion.preallocations.length > 0}
            <div>
              <h4 class="font-medium text-verusidx-stone-dark dark:text-white mb-3">Preallocations</h4>
              <div class="overflow-x-auto">
                <table class="w-full text-sm">
                  <thead>
                    <tr class="border-b border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
                      <th class="text-left py-2 px-3 font-medium text-verusidx-stone-dark dark:text-white">Identity Address</th>
                      <th class="text-right py-2 px-3 font-medium text-verusidx-stone-dark dark:text-white">Amount</th>
                    </tr>
                  </thead>
                  <tbody>
                    {#each preconversion.preallocations as allocation, index}
                      {#each Object.entries(allocation) as [address, amount]}
                        <tr class="border-b border-verusidx-mountain-mist/50 dark:border-verusidx-stone-medium/50 {index % 2 === 0 ? 'bg-verusidx-sky-soft/30 dark:bg-verusidx-stone-medium/30' : ''}">
                          <td class="py-2 px-3">
                            <button
                              onclick={() => openIdentityModal(address)}
                              class="font-mono text-sm text-verusidx-mountain-blue hover:text-verusidx-lake-blue dark:text-verusidx-turquoise-light dark:hover:text-verusidx-turquoise-deep cursor-pointer transition-colors"
                              title="Click to view identity details"
                            >
                              {address}
                            </button>
                          </td>
                          <td class="py-2 px-3 text-right font-mono text-verusidx-stone-dark dark:text-white">{formatCurrency(amount)}</td>
                        </tr>
                      {/each}
                    {/each}
                  </tbody>
                </table>
              </div>
            </div>
          {/if}
        </div>
      {/if}
    </div>
    {/if}

    <!-- Multicurrency PBaaS Section -->
    {#if shouldShowMulticurrencyPBaaS()}
    <div class="border-b border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
      <button
        class="w-full p-4 flex items-center justify-between hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors"
        onclick={() => toggleSection('multicurrencyPBaaS')}
      >
        <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">Multicurrency PBaaS</h3>
        <svg
          class="h-5 w-5 text-verusidx-mountain-grey transition-transform duration-200 {multicurrencyPBaaSExpanded ? 'rotate-180' : ''}"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
        </svg>
      </button>
      {#if multicurrencyPBaaSExpanded && multicurrencyPBaaS}
        <div class="px-4 pb-4 space-y-3">
          {#if multicurrencyPBaaS.currencyregistrationfee !== null}
            <div class="flex justify-between items-center">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Currency Registration Fee:</span>
              <span class="font-medium text-verusidx-stone-dark dark:text-white">{formatCurrency(multicurrencyPBaaS.currencyregistrationfee)}</span>
            </div>
          {/if}
          {#if multicurrencyPBaaS.pbaassystemregistrationfee !== null}
            <div class="flex justify-between items-center">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">PBaaS System Registration Fee:</span>
              <span class="font-medium text-verusidx-stone-dark dark:text-white">{formatCurrency(multicurrencyPBaaS.pbaassystemregistrationfee)}</span>
            </div>
          {/if}
          {#if multicurrencyPBaaS.currencyimportfee !== null}
            <div class="flex justify-between items-center">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Currency Import Fee:</span>
              <span class="font-medium text-verusidx-stone-dark dark:text-white">{formatCurrency(multicurrencyPBaaS.currencyimportfee)}</span>
            </div>
          {/if}
          {#if multicurrencyPBaaS.transactionimportfee !== null}
            <div class="flex justify-between items-center">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Transaction Import Fee:</span>
              <span class="font-medium text-verusidx-stone-dark dark:text-white">{formatCurrency(multicurrencyPBaaS.transactionimportfee)}</span>
            </div>
          {/if}
          {#if multicurrencyPBaaS.transactionexportfee !== null}
            <div class="flex justify-between items-center">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Transaction Export Fee:</span>
              <span class="font-medium text-verusidx-stone-dark dark:text-white">{formatCurrency(multicurrencyPBaaS.transactionexportfee)}</span>
            </div>
          {/if}
        </div>
      {/if}
    </div>
    {/if}

    <!-- Currency Fees Section -->
    {#if shouldShowCurrencyFees()}
    <div class="border-b border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
      <button
        class="w-full p-4 flex items-center justify-between hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors"
        onclick={() => toggleSection('currencyFees')}
      >
        <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">Currency Fees</h3>
        <svg
          class="h-5 w-5 text-verusidx-mountain-grey transition-transform duration-200 {currencyFeesExpanded ? 'rotate-180' : ''}"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
        </svg>
      </button>
      {#if currencyFeesExpanded && currencyFees}
        <div class="px-4 pb-4 space-y-3">
          {#if currencyFees.primarycurrencyfees !== null}
            <div class="flex justify-between items-center">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Primary Currency Fees:</span>
              <span class="font-medium text-verusidx-stone-dark dark:text-white">{formatCurrency(currencyFees.primarycurrencyfees)}</span>
            </div>
          {/if}
          {#if currencyFees.primarycurrencyconversionfees !== null}
            <div class="flex justify-between items-center">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Primary Currency Conversion Fees:</span>
              <span class="font-medium text-verusidx-stone-dark dark:text-white">{formatCurrency(currencyFees.primarycurrencyconversionfees)}</span>
            </div>
          {/if}
          {#if currencyFees.primarycurrencyout !== null}
            <div class="flex justify-between items-center">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Primary Currency Out:</span>
              <span class="font-medium text-verusidx-stone-dark dark:text-white">{formatCurrency(currencyFees.primarycurrencyout)}</span>
            </div>
          {/if}
          {#if currencyFees.preconvertedout !== null}
            <div class="flex justify-between items-center">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Preconverted Out:</span>
              <span class="font-medium text-verusidx-stone-dark dark:text-white">{formatCurrency(currencyFees.preconvertedout)}</span>
            </div>
          {/if}
        </div>
      {/if}
    </div>
    {/if}

    <!-- Gateway Info Section -->
    {#if shouldShowGatewayInfo()}
    <div class="border-b border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
      <button
        class="w-full p-4 flex items-center justify-between hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors"
        onclick={() => toggleSection('gatewayInfo')}
      >
        <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">Gateway Info</h3>
        <svg
          class="h-5 w-5 text-verusidx-mountain-grey transition-transform duration-200 {gatewayInfoExpanded ? 'rotate-180' : ''}"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
        </svg>
      </button>
      {#if gatewayInfoExpanded && gatewayInfo}
        <div class="px-4 pb-4 space-y-3">
          {#if gatewayInfo.gateway}
            <div class="flex justify-between items-center">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Gateway:</span>
              <span class="font-medium text-verusidx-stone-dark dark:text-white">{gatewayInfo.gateway}</span>
            </div>
          {/if}
          {#if gatewayInfo.gatewayconverterid}
            <div class="flex justify-between items-center">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Gateway Converter ID:</span>
              <div class="flex items-center space-x-2">
                <span class="font-medium text-verusidx-stone-dark dark:text-white text-sm font-mono">{gatewayInfo.gatewayconverterid}</span>
                <button
                  onclick={() => copyToClipboard(gatewayInfo.gatewayconverterid)}
                  class="p-1 hover:bg-verusidx-mountain-mist rounded text-verusidx-mountain-grey hover:text-verusidx-stone-dark"
                  title="Copy gateway converter ID"
                >
                  <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                  </svg>
                </button>
              </div>
            </div>
          {/if}
          {#if gatewayInfo.gatewayconvertername}
            <div class="flex justify-between items-center">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Gateway Converter Name:</span>
              <span class="font-medium text-verusidx-stone-dark dark:text-white">{gatewayInfo.gatewayconvertername}</span>
            </div>
          {/if}
          {#if gatewayInfo.gatewayconverterissuance !== null}
            <div class="flex justify-between items-center">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Gateway Converter Issuance:</span>
              <span class="font-medium text-verusidx-stone-dark dark:text-white">{formatCurrency(gatewayInfo.gatewayconverterissuance)}</span>
            </div>
          {/if}
        </div>
      {/if}
    </div>
    {/if}

    <!-- Blockchain Transaction Info Section -->
    {#if shouldShowBlockchain()}
    <div>
      <button
        class="w-full p-4 flex items-center justify-between hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors"
        onclick={() => toggleSection('blockchain')}
      >
        <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">Blockchain Transaction Info</h3>
        <svg
          class="h-5 w-5 text-verusidx-mountain-grey transition-transform duration-200 {blockchainExpanded ? 'rotate-180' : ''}"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
        </svg>
      </button>
      {#if blockchainExpanded && blockchain}
        <div class="px-4 pb-4 space-y-3">
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Start Block:</span>
            <span class="font-medium text-verusidx-stone-dark dark:text-white">{formatInteger(blockchain.startblock)}</span>
          </div>
          {#if blockchain.endblock > 0}
            <div class="flex justify-between items-center">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">End Block:</span>
              <span class="font-medium text-verusidx-stone-dark dark:text-white">{formatInteger(blockchain.endblock)}</span>
            </div>
          {:else}
            <div class="flex justify-between items-center">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">End Block:</span>
              <span class="font-medium text-verusidx-stone-dark dark:text-white">Never</span>
            </div>
          {/if}
          {#if blockchain.besttxid}
            <div class="flex justify-between items-center">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Best TXID:</span>
              <div class="flex items-center space-x-2">
                <span class="font-medium text-verusidx-stone-dark dark:text-white text-sm font-mono">{blockchain.besttxid}</span>
                <button
                  onclick={() => copyToClipboard(blockchain.besttxid)}
                  class="p-1 hover:bg-verusidx-mountain-mist rounded text-verusidx-mountain-grey hover:text-verusidx-stone-dark"
                  title="Copy transaction ID"
                >
                  <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                  </svg>
                </button>
              </div>
            </div>
          {/if}
          {#if blockchain.confirmedtxid}
            <div class="flex justify-between items-center">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Confirmed TXID:</span>
              <div class="flex items-center space-x-2">
                <span class="font-medium text-verusidx-stone-dark dark:text-white text-sm font-mono">{blockchain.confirmedtxid}</span>
                <button
                  onclick={() => copyToClipboard(blockchain.confirmedtxid)}
                  class="p-1 hover:bg-verusidx-mountain-mist rounded text-verusidx-mountain-grey hover:text-verusidx-stone-dark"
                  title="Copy transaction ID"
                >
                  <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                  </svg>
                </button>
              </div>
            </div>
          {/if}
          {#if blockchain.blockheight > 0}
            <div class="flex justify-between items-center">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Block Height:</span>
              <span class="font-medium text-verusidx-stone-dark dark:text-white">{formatInteger(blockchain.blockheight)}</span>
            </div>
          {/if}
          {#if blockchain.vout > 0}
            <div class="flex justify-between items-center">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">VOUT:</span>
              <span class="font-medium text-verusidx-stone-dark dark:text-white">{formatInteger(blockchain.vout)}</span>
            </div>
          {/if}
          {#if blockchain.definitiontxid}
            <div class="flex justify-between items-center">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Definition TXID:</span>
              <div class="flex items-center space-x-2 max-w-[60%]">
                <div class="overflow-x-auto">
                  <span class="font-medium text-verusidx-stone-dark dark:text-white text-sm font-mono whitespace-nowrap">{blockchain.definitiontxid}</span>
                </div>
                <button
                  onclick={() => copyToClipboard(blockchain.definitiontxid)}
                  class="p-1 hover:bg-verusidx-mountain-mist rounded text-verusidx-mountain-grey hover:text-verusidx-stone-dark flex-shrink-0"
                  title="Copy definition transaction ID"
                >
                  <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                  </svg>
                </button>
              </div>
            </div>
          {/if}
          {#if blockchain.definitiontxout !== null}
            <div class="flex justify-between items-center">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Definition TXOUT:</span>
              <span class="font-medium text-verusidx-stone-dark dark:text-white">{formatInteger(blockchain.definitiontxout)}</span>
            </div>
          {/if}
          {#if blockchain.bestheight > 0}
            <div class="flex justify-between items-center">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Best Height:</span>
              <span class="font-medium text-verusidx-stone-dark dark:text-white">{formatInteger(blockchain.bestheight)}</span>
            </div>
          {/if}
          {#if blockchain.lastconfirmedheight > 0}
            <div class="flex justify-between items-center">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Last Confirmed Height:</span>
              <span class="font-medium text-verusidx-stone-dark dark:text-white">{formatInteger(blockchain.lastconfirmedheight)}</span>
            </div>
          {/if}
          {#if blockchain.lastconfirmedtxid}
            <div class="flex justify-between items-center">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Last Confirmed TXID:</span>
              <div class="flex items-center space-x-2">
                <span class="font-medium text-verusidx-stone-dark dark:text-white text-sm font-mono">{blockchain.lastconfirmedtxid}</span>
                <button
                  onclick={() => copyToClipboard(blockchain.lastconfirmedtxid)}
                  class="p-1 hover:bg-verusidx-mountain-mist rounded text-verusidx-mountain-grey hover:text-verusidx-stone-dark"
                  title="Copy last confirmed transaction ID"
                >
                  <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                  </svg>
                </button>
              </div>
            </div>
          {/if}
          {#if blockchain.notarizationprotocol !== null}
            <div class="flex justify-between items-center">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Notarization Protocol:</span>
              <span class="font-medium text-verusidx-stone-dark dark:text-white">{formatInteger(blockchain.notarizationprotocol)}</span>
            </div>
          {/if}
          {#if blockchain.magicnumber !== null}
            <div class="flex justify-between items-center">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Magic Number:</span>
              <span class="font-medium text-verusidx-stone-dark dark:text-white">{formatInteger(blockchain.magicnumber)}</span>
            </div>
          {/if}
          {#if blockchain.version > 0}
            <div class="flex justify-between items-center">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Version:</span>
              <span class="font-medium text-verusidx-stone-dark dark:text-white">{formatInteger(blockchain.version)}</span>
            </div>
          {/if}
          {#if currencyData?.bestcurrencystate?.flags !== undefined}
            <div class="flex justify-between items-center">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Flags:</span>
              <span class="font-medium text-verusidx-stone-dark dark:text-white">{currencyData.bestcurrencystate.flags}</span>
            </div>
          {/if}
        </div>
      {/if}
    </div>
    {/if}
  {/if}
</div>

<!-- Identity Modal -->
<IdentityModal 
  isOpen={showIdentityModal}
  identityAddress={selectedIdentityAddress}
  onclose={closeIdentityModal}
/>