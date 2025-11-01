<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { connectionStore, getChainParam } from "$lib/stores/connection";
  import IdentityDetailsView from "./IdentityDetailsView.svelte";
  import IdentityBalanceModal from "./IdentityBalanceModal.svelte";
  import Modal from "./cards/Modal.svelte";

  let identities = $state<any[]>([]);
  let isLoading = $state(true);
  let error = $state<string | null>(null);
  let selectedIdentity = $state<any>(null);
  let isViewingDetails = $state(false);
  let connectionState = $state<any>(null);

  connectionStore.subscribe(state => {
    connectionState = state;
  });
  let isLoadingDetails = $state(false);
  let detailsError = $state<string | null>(null);

  // Balance modal state
  let isViewingBalance = $state(false);
  let selectedIdentityForBalance = $state<any>(null);

  // Dropdown state for manage actions
  let manageDropdownOpen = $state<string | null>(null);
  let dropdownPositions = $state<Record<string, 'above' | 'below'>>({});

  // Filter and search
  let searchFilter = $state("");
  let idFilter = $state("all");

  // State for filtered identities
  let filteredIdentities = $state<any[]>([]);

  // Effect to filter identities when data or filters change
  $effect(() => {
    console.log("IdentityListCard: $effect triggered for filtering");
    console.log("IdentityListCard: Input identities:", identities);
    console.log("IdentityListCard: Search filter:", searchFilter);
    console.log("IdentityListCard: ID filter:", idFilter);
    
    if (!identities || identities.length === 0) {
      console.log("IdentityListCard: No identities to filter");
      filteredIdentities = [];
      return;
    }
    
    const filtered = identities.filter(identity => {
      console.log("IdentityListCard: Processing identity:", identity);
      console.log("IdentityListCard: Identity keys:", Object.keys(identity));
      
      // Extract data from identity structure
      const name = identity.identity?.name || '';
      const parent = identity.identity?.parent || '';
      const systemid = identity.identity?.systemid || '';
      const timelock = identity.identity?.timelock || 0;
      const baseStatus = identity.status || 'unknown';
      const privateaddress = identity.identity?.privateaddress || '';
      const minimumsignatures = identity.identity?.minimumsignatures || 1;
      
      console.log("IdentityListCard: Extracted data:", {
        name, parent, systemid, timelock, baseStatus, 
        hasPrivateAddress: !!privateaddress, minimumsignatures
      });
      
      // Apply search filter
      const matchesSearch = !searchFilter || name.toLowerCase().includes(searchFilter.toLowerCase());
      
      // Apply ID filter based on category
      let matchesFilter = false;
      if (idFilter === 'all') {
        matchesFilter = true;
      } else if (idFilter === 'active') {
        matchesFilter = baseStatus === 'active';
      } else if (idFilter === 'timelocked') {
        matchesFilter = timelock > 0;
      } else if (idFilter === 'revoked') {
        matchesFilter = baseStatus === 'revoked';
      } else if (idFilter === 'rootid') {
        matchesFilter = parent === systemid;
      } else if (idFilter === 'subid') {
        matchesFilter = parent !== systemid;
      } else if (idFilter === 'privateaddress') {
        matchesFilter = privateaddress && privateaddress.trim() !== '';
      } else if (idFilter === 'multisig') {
        matchesFilter = minimumsignatures > 1;
      }
      
      console.log("IdentityListCard: Matches search:", matchesSearch, "Matches filter:", matchesFilter, "Filter:", idFilter);
      
      return matchesSearch && matchesFilter;
    });
    
    console.log("IdentityListCard: Filtered result:", filtered);
    filteredIdentities = filtered;
  });

  async function loadIdentities() {
    isLoading = true;
    error = null;
    
    try {
      const chainParam = getChainParam(connectionState?.selectedChain);
      console.log("IdentityListCard: Loading identities for chain:", connectionState?.selectedChain, "param:", chainParam);
      
      const result = await invoke("list_identities", { chain: chainParam });
      
      console.log("IdentityListCard: list_identities raw result:", result);
      console.log("IdentityListCard: result type:", typeof result);
      console.log("IdentityListCard: result isArray:", Array.isArray(result));
      
      if (Array.isArray(result)) {
        console.log("IdentityListCard: result length:", result.length);
        if (result.length > 0) {
          console.log("IdentityListCard: first identity structure:", result[0]);
          console.log("IdentityListCard: first identity keys:", Object.keys(result[0]));
          console.log("IdentityListCard: first identity JSON:", JSON.stringify(result[0], null, 2));
          
          // Check for nested identity structure
          if (result[0].identity) {
            console.log("IdentityListCard: nested identity structure:", result[0].identity);
            console.log("IdentityListCard: nested identity keys:", Object.keys(result[0].identity));
          }
        }
      }
      
      identities = Array.isArray(result) ? result : [];
      console.log("IdentityListCard: Set identities to:", identities);
    } catch (err) {
      console.error("Failed to load identities:", err);
      error = typeof err === 'string' ? err : "Failed to load identities. Please try again.";
      identities = [];
    } finally {
      isLoading = false;
    }
  }

  async function viewIdentityDetails(identity: any) {
    console.log("IdentityListCard: viewIdentityDetails called with identity:", identity);
    
    // Use identity address instead of name for reliable lookups
    const identityAddress = identity.identity?.identityaddress;
    console.log("IdentityListCard: Using identity address for lookup:", identityAddress);
    
    if (!identityAddress) {
      detailsError = "Invalid identity address";
      return;
    }

    isViewingDetails = true;
    isLoadingDetails = true;
    detailsError = null;
    selectedIdentity = null;

    try {
      const chainParam = getChainParam(connectionState?.selectedChain);
      console.log("IdentityListCard: Loading identity details for chain:", connectionState?.selectedChain, "param:", chainParam);
      
      // Use identity address which works for both root and sub-identities
      const result = await invoke("get_identity", { 
        name: identityAddress,
        chain: chainParam
      });

      console.log("IdentityListCard: get_identity result:", result);
      selectedIdentity = result;
    } catch (err) {
      console.error("Failed to load identity details:", err);
      detailsError = typeof err === 'string' ? err : "Failed to load identity details. Please try again.";
    } finally {
      isLoadingDetails = false;
    }
  }

  function closeDetailsModal() {
    isViewingDetails = false;
    selectedIdentity = null;
    detailsError = null;
  }

  function viewIdentityBalance(identity: any) {
    selectedIdentityForBalance = identity;
    isViewingBalance = true;
    closeManageDropdown();
  }

  function closeBalanceModal() {
    isViewingBalance = false;
    selectedIdentityForBalance = null;
  }

  function toggleManageDropdown(identityAddress: string, event: MouseEvent) {
    if (manageDropdownOpen === identityAddress) {
      manageDropdownOpen = null;
      return;
    }
    
    // Calculate optimal dropdown position
    const button = event.currentTarget as HTMLElement;
    const rect = button.getBoundingClientRect();
    const dropdownHeight = 80; // Approximate height for one option
    
    const spaceBelow = window.innerHeight - rect.bottom;
    const shouldShowAbove = spaceBelow < dropdownHeight + 20; // 20px buffer
    
    dropdownPositions = {
      ...dropdownPositions,
      [identityAddress]: shouldShowAbove ? 'above' : 'below'
    };
    
    manageDropdownOpen = identityAddress;
  }

  function closeManageDropdown() {
    manageDropdownOpen = null;
  }

  function getStatusColor(status: string) {
    switch (status) {
      case 'active':
        return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200';
      case 'timelocked':
        return 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200';
      case 'revoked':
        return 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200';
      default:
        return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200';
    }
  }

  function getBadges(identity: any) {
    const badges = [];
    
    // Extract identity data
    const timelock = identity.identity?.timelock || 0;
    const baseStatus = identity.status || 'unknown';
    const parent = identity.identity?.parent || '';
    const systemid = identity.identity?.systemid || '';
    const privateaddress = identity.identity?.privateaddress || '';
    const minimumsignatures = identity.identity?.minimumsignatures || 1;
    const flags = identity.identity?.flags || 0;
    
    // Status badge (active, revoked, etc.) - always show the original status
    badges.push({
      text: baseStatus,
      classes: getStatusColor(baseStatus)
    });
    
    // Timelocked badge
    if (timelock > 0) {
      badges.push({
        text: 'timelocked',
        classes: 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200'
      });
    }
    
    // Root ID vs Sub-ID
    if (parent === systemid) {
      badges.push({
        text: 'rootID',
        classes: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200'
      });
    } else {
      badges.push({
        text: 'subID', 
        classes: 'bg-teal-100 text-teal-800 dark:bg-teal-900 dark:text-teal-200'
      });
    }
    
    // Private Address badge
    if (privateaddress && privateaddress.trim() !== '') {
      badges.push({
        text: 'z-address',
        classes: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200'
      });
    }
    
    // Multisig badge
    if (minimumsignatures > 1) {
      badges.push({
        text: 'multisig',
        classes: 'bg-pink-100 text-pink-800 dark:bg-pink-900 dark:text-pink-200'
      });
    }
    
    // Currency badge
    if (flags === 1) {
      badges.push({
        text: 'currency',
        classes: 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200'
      });
    }
    
    // Token Control badge
    if (flags === 5) {
      badges.push({
        text: 'token-control',
        classes: 'bg-pink-100 text-pink-800 dark:bg-pink-900 dark:text-pink-200'
      });
    }
    
    return badges;
  }

  function isSubIdentity(identity: any): boolean {
    const name = identity.identity?.name || '';
    return name.includes('@');
  }

  // Load identities when component mounts
  $effect(() => {
    loadIdentities();
  });

  // Reload identities when chain changes
  $effect(() => {
    if (connectionState?.selectedChain) {
      loadIdentities();
    }
  });

  // Close dropdown when clicking outside
  $effect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (manageDropdownOpen && !(event.target as Element)?.closest('.manage-dropdown-container')) {
        closeManageDropdown();
      }
    }

    if (manageDropdownOpen) {
      document.addEventListener('click', handleClickOutside, true);
      return () => document.removeEventListener('click', handleClickOutside, true);
    }
  });
</script>

<div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl">
  <!-- Header -->
  <div class="p-6 border-b border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-xl font-semibold text-verusidx-stone-dark dark:text-white">Your Identities</h2>
      <button
        onclick={loadIdentities}
        class="p-2 text-verusidx-mountain-grey hover:text-verusidx-stone-dark dark:hover:text-white transition-colors"
        title="Refresh identities"
      >
        <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
        </svg>
      </button>
    </div>

    <!-- Filters -->
    <div class="flex flex-col sm:flex-row gap-4">
      <div class="flex-1">
        <input
          type="text"
          bind:value={searchFilter}
          placeholder="Search identities..."
          class="w-full px-3 py-2 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg text-verusidx-stone-dark dark:text-white placeholder-verusidx-mountain-grey dark:placeholder-verusidx-mountain-mist focus:ring-2 focus:ring-verusidx-mountain-blue focus:border-transparent outline-none"
        />
      </div>
      <div class="sm:w-48">
        <select
          bind:value={idFilter}
          class="w-full px-3 py-2 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg text-verusidx-stone-dark dark:text-white focus:ring-2 focus:ring-verusidx-mountain-blue focus:border-transparent outline-none"
        >
          <option value="all">All IDs</option>
          <option value="active">Active</option>
          <option value="timelocked">Timelocked</option>
          <option value="revoked">Revoked</option>
          <option value="rootid">Root ID</option>
          <option value="subid">Sub-ID</option>
          <option value="privateaddress">Private Address</option>
          <option value="multisig">Multisig</option>
        </select>
      </div>
    </div>
  </div>

  <!-- Content -->
  <div class="p-6">
    {#if isLoading}
      <div class="flex items-center justify-center py-8">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-verusidx-mountain-blue"></div>
        <span class="ml-3 text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Loading identities...</span>
      </div>
    {:else if error}
      <div class="text-center py-8">
        <div class="text-red-600 dark:text-red-400 mb-2">
          <svg class="h-12 w-12 mx-auto mb-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.732-.833-2.5 0L4.268 16.5c-.77.833.192 2.5 1.732 2.5z" />
          </svg>
        </div>
        <p class="text-red-600 dark:text-red-400 font-medium">{error}</p>
        <button
          onclick={loadIdentities}
          class="mt-4 px-4 py-2 bg-verusidx-mountain-blue text-white rounded-lg hover:bg-verusidx-lake-blue transition-colors"
        >
          Try Again
        </button>
      </div>
    {:else if filteredIdentities.length === 0}
      <div class="text-center py-8">
        <div class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-2">
          <svg class="h-12 w-12 mx-auto mb-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
          </svg>
        </div>
        <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist font-medium">
          {identities.length === 0 ? 'No identities found' : 'No identities match your filters'}
        </p>
        <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-2">
          {identities.length === 0 ? 'Create your first identity to get started' : 'Try adjusting your search or filter criteria'}
        </p>
      </div>
    {:else}
      <div class="space-y-4">
        <div class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-4">
          Showing {filteredIdentities.length} of {identities.length} identities
        </div>
        
        <div class="grid gap-4">
          {#each filteredIdentities as identity}
            {@const name = identity.identity?.name || 'Unknown'}
            {@const timelock = identity.identity?.timelock || 0}
            {@const baseStatus = identity.status || 'unknown'}
            {@const identityAddress = identity.identity?.identityaddress || 'Unknown'}
            {@const parent = identity.identity?.parent || 'Unknown'}
            {@const badges = getBadges(identity)}
            
            <div class="border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg p-4 hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors">
              <div class="flex items-start justify-between">
                <div class="flex-1">
                  <div class="flex items-center flex-wrap gap-2 mb-3">
                    <h3 class="text-lg font-medium text-verusidx-stone-dark dark:text-white">{name}</h3>
                    {#each badges as badge}
                      <span class="px-2 py-1 text-xs rounded-full {badge.classes}">
                        {badge.text}
                      </span>
                    {/each}
                  </div>
                  
                  <div class="space-y-1 text-sm">
                    <div>
                      <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Identity Address:</span>
                      <span class="font-mono text-verusidx-stone-dark dark:text-white ml-2">{identityAddress}</span>
                    </div>
                    <div>
                      <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Parent:</span>
                      <span class="font-mono text-verusidx-stone-dark dark:text-white ml-2">{parent}</span>
                    </div>
                  </div>
                </div>
                
                <div class="flex items-center space-x-2 ml-4">
                  <button
                    onclick={() => viewIdentityDetails(identity)}
                    class="px-3 py-1 text-sm bg-verusidx-mountain-blue text-white rounded hover:bg-verusidx-lake-blue transition-colors"
                  >
                    View Details
                  </button>
                  
                  {#if baseStatus === 'active'}
                    <div class="relative manage-dropdown-container">
                      <button
                        onclick={(e) => toggleManageDropdown(identityAddress, e)}
                        class="px-3 py-1 text-sm bg-verusidx-turquoise-deep text-white rounded hover:bg-verusidx-turquoise-bright transition-colors flex items-center gap-1"
                      >
                        Manage
                        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                        </svg>
                      </button>
                      
                      {#if manageDropdownOpen === identityAddress}
                        <div class="absolute right-0 w-40 bg-white dark:bg-verusidx-stone-dark border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg shadow-lg z-50 {dropdownPositions[identityAddress] === 'above' ? 'bottom-full mb-1' : 'top-full mt-1'}">
                          <button
                            onclick={() => viewIdentityBalance(identity)}
                            class="w-full text-left px-3 py-2 text-xs text-verusidx-stone-dark dark:text-white hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors rounded-lg"
                          >
                            Balance
                          </button>
                        </div>
                      {/if}
                    </div>
                  {/if}
                </div>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</div>

<!-- Identity Details Modal -->
{#if isViewingDetails}
  <Modal isOpen={isViewingDetails} title="Identity Details" size="lg" onclose={closeDetailsModal}>
    <IdentityDetailsView 
      identityData={selectedIdentity} 
      isLoading={isLoadingDetails}
      error={detailsError}
    />
  </Modal>
{/if}

<!-- Identity Balance Modal -->
<IdentityBalanceModal
  identity={selectedIdentityForBalance}
  isOpen={isViewingBalance}
  onClose={closeBalanceModal}
/>