<script lang="ts">
  interface Props {
    identityData: any;
    isLoading?: boolean;
    error?: string | null;
  }

  let { identityData, isLoading = false, error = null }: Props = $props();

  // Direct prop logging
  console.log("IdentityDetailsView: Component initialized with identityData:", identityData);

  // Collapsible section states
  let nameIdExpanded = $state(false);
  let authoritiesExpanded = $state(false);
  let technicalExpanded = $state(false);
  let contentMapExpanded = $state(false);

  // State variables for processed data
  let nameIdData = $state(null);
  let authoritiesData = $state(null);
  let technicalData = $state(null);
  let contentMapData = $state(null);

  // Effect to process identityData when it changes
  $effect(() => {
    console.log("IdentityDetailsView: $effect triggered with identityData:", identityData);
    
    if (!identityData) {
      console.log("IdentityDetailsView: No identityData, setting all data to null");
      nameIdData = null;
      authoritiesData = null;
      technicalData = null;
      contentMapData = null;
      return;
    }
    
    console.log("IdentityDetailsView: Processing identityData:", identityData);
    
    // Handle both direct response (from getidentity) and nested structure (from listidentities)
    const identity = identityData.identity || identityData;
    console.log("IdentityDetailsView: Extracted identity object:", identity);
    
    // Process nameIdData
    const nameId = {
      name: identity?.name || 'Unknown',
      friendlyName: identityData.friendlyname || `${identity?.name || 'Unknown'}@`,
      fullyQualifiedName: identityData.fullyqualifiedname || `${identity?.name || 'Unknown'}@`,
      identityAddress: identity?.identityaddress || 'Unknown',
      parent: identity?.parent || 'Unknown',
      systemId: identity?.systemid || 'Unknown'
    };
    
    // Process authoritiesData
    const authorities = {
      primaryAddresses: identity?.primaryaddresses || [],
      minimumSignatures: identity?.minimumsignatures || 0,
      revocationAuthority: identity?.revocationauthority || 'Unknown',
      recoveryAuthority: identity?.recoveryauthority || 'Unknown',
      privateAddress: identity?.privateaddress || 'None',
      timelock: identity?.timelock || 0
    };

    // Process technicalData
    const technical = {
      status: identityData.status || 'unknown',
      flags: identity?.flags || 0,
      version: identity?.version || 0,
      blockHeight: identityData.blockheight || 0,
      txid: identityData.txid || 'Unknown',
      vout: identityData.vout || 0,
      canSpendFor: identityData.canspendfor || false,
      canSignFor: identityData.cansignfor || false
    };

    // Process contentMapData
    const contentMap = {
      contentMap: identity?.contentmap || {},
      contentMultimap: identity?.contentmultimap || {}
    };
    
    console.log("IdentityDetailsView: Setting processed data:", {
      nameId,
      authorities,
      technical,
      contentMap
    });
    
    nameIdData = nameId;
    authoritiesData = authorities;
    technicalData = technical;
    contentMapData = contentMap;
  });

  function copyToClipboard(text: string) {
    navigator.clipboard.writeText(text);
  }

  function toggleSection(section: string) {
    switch (section) {
      case 'nameId':
        nameIdExpanded = !nameIdExpanded;
        break;
      case 'authorities':
        authoritiesExpanded = !authoritiesExpanded;
        break;
      case 'technical':
        technicalExpanded = !technicalExpanded;
        break;
      case 'contentMap':
        contentMapExpanded = !contentMapExpanded;
        break;
    }
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
  {:else if !identityData}
    <div class="p-6">
      <div class="text-center text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
        <p class="text-sm">No identity data available</p>
      </div>
    </div>
  {:else}
    <!-- Name ID Section -->
    <div class="border-b border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
      <button
        class="w-full p-4 flex items-center justify-between hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors"
        onclick={() => toggleSection('nameId')}
      >
        <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">Name ID</h3>
        <svg
          class="h-5 w-5 text-verusidx-mountain-grey transition-transform duration-200 {nameIdExpanded ? 'rotate-180' : ''}"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
        </svg>
      </button>
      {#if nameIdExpanded && nameIdData}
        <div class="px-4 pb-4 space-y-3">
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Name:</span>
            <span class="font-medium text-verusidx-stone-dark dark:text-white">{nameIdData.name}</span>
          </div>
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Friendly Name:</span>
            <span class="font-medium text-verusidx-stone-dark dark:text-white">{nameIdData.friendlyName}</span>
          </div>
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Fully Qualified Name:</span>
            <span class="font-medium text-verusidx-stone-dark dark:text-white">{nameIdData.fullyQualifiedName}</span>
          </div>
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Identity Address:</span>
            <div class="flex items-center space-x-2">
              <span class="font-medium text-verusidx-stone-dark dark:text-white text-sm font-mono">{nameIdData.identityAddress}</span>
              <button
                onclick={() => copyToClipboard(nameIdData.identityAddress)}
                class="p-1 hover:bg-verusidx-mountain-mist rounded text-verusidx-mountain-grey hover:text-verusidx-stone-dark"
              >
                <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                </svg>
              </button>
            </div>
          </div>
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Parent:</span>
            <span class="font-medium text-verusidx-stone-dark dark:text-white text-sm font-mono">{nameIdData.parent}</span>
          </div>
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">System ID:</span>
            <span class="font-medium text-verusidx-stone-dark dark:text-white text-sm font-mono">{nameIdData.systemId}</span>
          </div>
        </div>
      {/if}
    </div>

    <!-- Authorities Section -->
    <div class="border-b border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
      <button
        class="w-full p-4 flex items-center justify-between hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors"
        onclick={() => toggleSection('authorities')}
      >
        <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">Authorities</h3>
        <svg
          class="h-5 w-5 text-verusidx-mountain-grey transition-transform duration-200 {authoritiesExpanded ? 'rotate-180' : ''}"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
        </svg>
      </button>
      {#if authoritiesExpanded && authoritiesData}
        <div class="px-4 pb-4 space-y-3">
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Primary Addresses:</span>
            <div class="text-right">
              {#each authoritiesData.primaryAddresses as address}
                <div class="flex items-center space-x-2 mb-1">
                  <span class="font-medium text-verusidx-stone-dark dark:text-white text-sm font-mono">{address}</span>
                  <button
                    onclick={() => copyToClipboard(address)}
                    class="p-1 hover:bg-verusidx-mountain-mist rounded text-verusidx-mountain-grey hover:text-verusidx-stone-dark"
                  >
                    <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                    </svg>
                  </button>
                </div>
              {/each}
            </div>
          </div>
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Minimum Signatures:</span>
            <span class="font-medium text-verusidx-stone-dark dark:text-white">{authoritiesData.minimumSignatures}</span>
          </div>
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Revocation Authority:</span>
            <span class="font-medium text-verusidx-stone-dark dark:text-white text-sm font-mono">{authoritiesData.revocationAuthority}</span>
          </div>
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Recovery Authority:</span>
            <span class="font-medium text-verusidx-stone-dark dark:text-white text-sm font-mono">{authoritiesData.recoveryAuthority}</span>
          </div>
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Private Address:</span>
            <div class="flex items-center space-x-2 max-w-[60%]">
              <div class="overflow-x-auto">
                <span class="font-medium text-verusidx-stone-dark dark:text-white text-sm font-mono whitespace-nowrap">{authoritiesData.privateAddress}</span>
              </div>
              {#if authoritiesData.privateAddress !== 'None'}
                <button
                  onclick={() => copyToClipboard(authoritiesData.privateAddress)}
                  class="p-1 hover:bg-verusidx-mountain-mist rounded text-verusidx-mountain-grey hover:text-verusidx-stone-dark flex-shrink-0"
                >
                  <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                  </svg>
                </button>
              {/if}
            </div>
          </div>
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Timelock:</span>
            <span class="font-medium text-verusidx-stone-dark dark:text-white">{authoritiesData.timelock}</span>
          </div>
        </div>
      {/if}
    </div>

    <!-- Technical Details Section -->
    <div class="border-b border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
      <button
        class="w-full p-4 flex items-center justify-between hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors"
        onclick={() => toggleSection('technical')}
      >
        <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">Technical Details</h3>
        <svg
          class="h-5 w-5 text-verusidx-mountain-grey transition-transform duration-200 {technicalExpanded ? 'rotate-180' : ''}"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
        </svg>
      </button>
      {#if technicalExpanded && technicalData}
        <div class="px-4 pb-4 space-y-3">
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Status:</span>
            <span class="font-medium text-verusidx-stone-dark dark:text-white">
              <span class="px-2 py-1 text-xs rounded {technicalData.status === 'active' ? 'bg-green-100 text-green-800' : 'bg-yellow-100 text-yellow-800'}">
                {technicalData.status}
              </span>
            </span>
          </div>
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Flags:</span>
            <span class="font-medium text-verusidx-stone-dark dark:text-white">{technicalData.flags}</span>
          </div>
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Version:</span>
            <span class="font-medium text-verusidx-stone-dark dark:text-white">{technicalData.version}</span>
          </div>
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Block Height:</span>
            <span class="font-medium text-verusidx-stone-dark dark:text-white">{technicalData.blockHeight.toLocaleString()}</span>
          </div>
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">TXID:</span>
            <div class="flex items-center space-x-2 max-w-[60%]">
              <div class="overflow-x-auto">
                <span class="font-medium text-verusidx-stone-dark dark:text-white text-sm font-mono whitespace-nowrap">{technicalData.txid}</span>
              </div>
              <button
                onclick={() => copyToClipboard(technicalData.txid)}
                class="p-1 hover:bg-verusidx-mountain-mist rounded text-verusidx-mountain-grey hover:text-verusidx-stone-dark flex-shrink-0"
              >
                <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                </svg>
              </button>
            </div>
          </div>
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">VOUT:</span>
            <span class="font-medium text-verusidx-stone-dark dark:text-white">{technicalData.vout}</span>
          </div>
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Can Spend For:</span>
            <span class="font-medium text-verusidx-stone-dark dark:text-white">{technicalData.canSpendFor ? 'Yes' : 'No'}</span>
          </div>
          <div class="flex justify-between items-center">
            <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Can Sign For:</span>
            <span class="font-medium text-verusidx-stone-dark dark:text-white">{technicalData.canSignFor ? 'Yes' : 'No'}</span>
          </div>
        </div>
      {/if}
    </div>

    <!-- Content/Data Section -->
    <div>
      <button
        class="w-full p-4 flex items-center justify-between hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors"
        onclick={() => toggleSection('contentMap')}
      >
        <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">Content/Data</h3>
        <svg
          class="h-5 w-5 text-verusidx-mountain-grey transition-transform duration-200 {contentMapExpanded ? 'rotate-180' : ''}"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
        </svg>
      </button>
      {#if contentMapExpanded && contentMapData}
        <div class="px-4 pb-4 space-y-4">
          {#if Object.keys(contentMapData.contentMap).length > 0 || Object.keys(contentMapData.contentMultimap).length > 0}
            {#if Object.keys(contentMapData.contentMap).length > 0}
              <div>
                <h4 class="font-medium text-verusidx-stone-dark dark:text-white mb-2">Content Map</h4>
                <div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded text-sm overflow-auto max-h-[300px]">
                  <pre class="text-verusidx-stone-dark dark:text-white whitespace-pre">{JSON.stringify(contentMapData.contentMap, null, 2)}</pre>
                </div>
              </div>
            {/if}
            {#if Object.keys(contentMapData.contentMultimap).length > 0}
              <div>
                <h4 class="font-medium text-verusidx-stone-dark dark:text-white mb-2">Content Multimap</h4>
                <div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded text-sm overflow-auto max-h-[300px]">
                  <pre class="text-verusidx-stone-dark dark:text-white whitespace-pre">{JSON.stringify(contentMapData.contentMultimap, null, 2)}</pre>
                </div>
              </div>
            {/if}
          {:else}
            <div class="text-center text-verusidx-mountain-grey dark:text-verusidx-mountain-mist py-4">
              <p class="text-sm">No content map data available</p>
            </div>
          {/if}
        </div>
      {/if}
    </div>
  {/if}
</div>