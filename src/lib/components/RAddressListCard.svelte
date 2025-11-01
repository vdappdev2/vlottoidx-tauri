<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { ExpandableCard, Modal } from "$lib/components/cards";

  let addresses = $state<string[]>([]);
  let isLoading = $state(false);
  let isGenerating = $state(false);
  let error = $state<string | null>(null);
  let showNewAddressModal = $state(false);
  let newGeneratedAddress = $state<string>("");

  async function loadAddresses() {
    isLoading = true;
    error = null;

    try {
      // Use empty string for default account as per RPC docs
      const result = await invoke("get_addresses_by_account", { 
        account: "",
        chain: null
      });
      
      addresses = result as string[];
    } catch (err) {
      console.error("Failed to load R-addresses:", err);
      error = typeof err === 'string' ? err : "Failed to load addresses";
      addresses = [];
    } finally {
      isLoading = false;
    }
  }

  async function generateNewAddress() {
    isGenerating = true;
    error = null;

    try {
      const newAddress = await invoke("get_new_address", { 
        account: null,
        chain: null
      });
      
      // Store the new address and show modal
      newGeneratedAddress = newAddress as string;
      showNewAddressModal = true;
      
      // Reload addresses to show the new one
      await loadAddresses();
    } catch (err) {
      console.error("Failed to generate new address:", err);
      error = typeof err === 'string' ? err : "Failed to generate new address";
    } finally {
      isGenerating = false;
    }
  }

  async function copyToClipboard(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      console.log("Copied to clipboard:", text);
    } catch (err) {
      console.error("Failed to copy:", err);
    }
  }

  function formatAddress(address: string) {
    if (address.length > 20) {
      return `${address.slice(0, 8)}...${address.slice(-8)}`;
    }
    return address;
  }

  // Load addresses when component mounts
  $effect(() => {
    loadAddresses();
  });

  // Dynamic title with count
  let cardTitle = $derived(`R-address List (${addresses.length})`);

  function closeNewAddressModal() {
    showNewAddressModal = false;
    newGeneratedAddress = "";
  }
</script>

<ExpandableCard title={cardTitle} cardClass="bg-white dark:bg-verusidx-stone-dark" modalSize="lg">
  <!-- Preview Content -->
  <div slot="preview">
    <!-- Generate Button -->
    <div class="flex justify-end mb-4">
      <button
        onclick={generateNewAddress}
        disabled={isGenerating}
        class="px-3 py-1 bg-verusidx-mountain-blue dark:bg-verusidx-turquoise-deep text-white text-sm rounded-lg hover:bg-verusidx-lake-blue dark:hover:bg-verusidx-turquoise-bright disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
      >
        {isGenerating ? 'Generating...' : 'Generate New R-Address'}
      </button>
    </div>

    {#if isLoading}
      <div class="text-center py-4">
        <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-verusidx-mountain-blue mx-auto"></div>
        <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-2">Loading addresses...</p>
      </div>
    {:else if error}
      <div class="text-center py-4">
        <p class="text-sm text-red-600 dark:text-red-400">{error}</p>
        <button
          onclick={loadAddresses}
          class="mt-2 text-sm text-verusidx-mountain-blue hover:text-verusidx-lake-blue"
        >
          Try Again
        </button>
      </div>
    {:else if addresses.length === 0}
      <div class="text-center py-4 text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
        <p class="text-sm">No R-addresses found</p>
        <p class="text-xs mt-2">Click "Generate New R-Address" to create one</p>
      </div>
    {:else}
      <div class="space-y-2">
        {#each addresses.slice(0, 3) as address}
          <div class="flex justify-between items-center p-2 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium rounded">
            <div class="font-mono text-sm text-verusidx-stone-dark dark:text-white">
              {formatAddress(address)}
            </div>
          </div>
        {/each}
        {#if addresses.length > 3}
          <div class="text-center text-verusidx-mountain-grey dark:text-verusidx-mountain-mist text-sm">
            And {addresses.length - 3} more...
          </div>
        {/if}
      </div>
    {/if}
  </div>

  <!-- Expanded Content -->
  <div slot="expanded">
    <!-- Generate Button in Expanded View -->
    <div class="flex justify-end mb-4">
      <button
        onclick={generateNewAddress}
        disabled={isGenerating}
        class="px-4 py-2 bg-verusidx-mountain-blue dark:bg-verusidx-turquoise-deep text-white rounded-lg hover:bg-verusidx-lake-blue dark:hover:bg-verusidx-turquoise-bright disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
      >
        {isGenerating ? 'Generating...' : 'Generate New R-Address'}
      </button>
    </div>
    {#if isLoading}
      <div class="text-center py-8">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-verusidx-mountain-blue mx-auto"></div>
        <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-2">Loading addresses...</p>
      </div>
    {:else if error}
      <div class="text-center py-8">
        <p class="text-red-600 dark:text-red-400">{error}</p>
        <button
          onclick={loadAddresses}
          class="mt-4 px-4 py-2 bg-verusidx-mountain-blue text-white rounded-lg hover:bg-verusidx-lake-blue transition-colors"
        >
          Try Again
        </button>
      </div>
    {:else if addresses.length === 0}
      <div class="text-center py-8 text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
        <p>No R-addresses found</p>
        <p class="text-sm mt-2">Click "Generate New R-Address" to create one</p>
      </div>
    {:else}
      <div class="space-y-3 max-h-96 overflow-y-auto">
        {#each addresses as address}
          <div class="p-4 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium rounded-lg">
            <div class="flex justify-between items-center">
              <div class="font-mono text-sm text-verusidx-stone-dark dark:text-white break-all flex-1">
                {address}
              </div>
              <button
                onclick={() => copyToClipboard(address)}
                class="ml-4 px-3 py-1 bg-verusidx-mountain-mist dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-verusidx-mountain-mist text-xs rounded hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-medium transition-colors"
                title="Copy address"
              >
                Copy
              </button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</ExpandableCard>

<!-- New Address Generated Modal -->
<Modal isOpen={showNewAddressModal} title="New R-Address Generated!" size="md" onclose={closeNewAddressModal}>
  <div class="space-y-4">
    <div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-4 rounded-lg">
      <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-2">Your new address:</p>
      <p class="font-mono text-sm text-verusidx-stone-dark dark:text-white break-all select-all">
        {newGeneratedAddress}
      </p>
    </div>
    
    <div class="flex gap-3 justify-end">
      <button
        onclick={() => copyToClipboard(newGeneratedAddress)}
        class="px-4 py-2 bg-verusidx-turquoise-deep text-white rounded-lg hover:bg-verusidx-turquoise-bright transition-colors"
      >
        Copy Address
      </button>
      <button
        onclick={closeNewAddressModal}
        class="px-4 py-2 bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-stone-medium dark:text-verusidx-mountain-mist rounded-lg hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-dark transition-colors"
      >
        Close
      </button>
    </div>
  </div>
</Modal>