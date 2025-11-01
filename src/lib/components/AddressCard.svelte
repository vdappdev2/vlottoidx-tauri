<script lang="ts">
  import { ExpandableCard } from "$lib/components/cards";
  import { connectionStore } from "$lib/stores/connection";

  let connectionState = $state<any>(null);
  
  connectionStore.subscribe(state => {
    connectionState = state;  // Fix: use full state to access selectedChain
  });

  interface Props {
    title: string;
    type: 'transparent' | 'private';
    addresses: any[];
    onAddressClick: (address: string) => void;
    onGenerateNew?: () => void;
    isGenerating?: boolean;
    cardClass?: string;
  }

  let { 
    title, 
    type, 
    addresses, 
    onAddressClick, 
    onGenerateNew,
    isGenerating = false,
    cardClass = '' 
  }: Props = $props();

  async function copyToClipboard(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      // Could add a toast notification here
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

  // Create a dynamic title that includes the generate button
  let cardTitle = $derived(`${title} (${addresses.length})`);

  function getAddressBalance(addressData: any): number {
    // For transparent addresses from listaddressgroupings
    if (type === 'transparent' && addressData && typeof addressData === 'object') {
      // addressData is {address, amount, account}
      return addressData.amount || 0;
    }
    // For private addresses, we don't get balance from z_listaddresses
    return 0;
  }

  function getAddressString(addressData: any): string {
    // For transparent addresses from listaddressgroupings
    if (type === 'transparent' && addressData && typeof addressData === 'object') {
      return addressData.address;
    }
    // For private addresses, it's just a string
    return addressData;
  }
</script>

<ExpandableCard title={cardTitle} {cardClass} modalSize="lg">
  <!-- Preview Content -->
  <div slot="preview">
    <!-- Generate Button (only show if onGenerateNew is provided) -->
    {#if onGenerateNew}
      <div class="flex justify-end mb-4">
        <button
          onclick={onGenerateNew}
          disabled={isGenerating}
          class="px-3 py-1 bg-verusidx-mountain-blue dark:bg-verusidx-turquoise-deep text-white text-sm rounded-lg hover:bg-verusidx-lake-blue dark:hover:bg-verusidx-turquoise-bright disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          {isGenerating ? 'Generating...' : `Generate New ${type === 'transparent' ? 'Address' : 'Z-Address'}`}
        </button>
      </div>
    {/if}
    {#if addresses.length === 0}
      <div class="text-center py-4 text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
        <p class="text-sm">No {type} addresses found</p>
        {#if onGenerateNew}
          <p class="text-xs mt-2">Click "Generate New" to create one</p>
        {/if}
      </div>
    {:else}
      <div class="space-y-2">
        {#each addresses.slice(0, 3) as addressData}
          {@const address = getAddressString(addressData)}
          {@const balance = getAddressBalance(addressData)}
          <div class="flex justify-between items-center p-2 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium rounded">
            <div class="flex-1">
              <div class="font-mono text-sm text-verusidx-stone-dark dark:text-white">
                {formatAddress(address)}
              </div>
              {#if type === 'transparent' && balance > 0}
                <div class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
                  Balance: {balance.toFixed(8)}
                </div>
              {/if}
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
    <!-- Generate Button in Expanded View (only show if onGenerateNew is provided) -->
    {#if onGenerateNew}
      <div class="flex justify-end mb-4">
        <button
          onclick={onGenerateNew}
          disabled={isGenerating}
          class="px-4 py-2 bg-verusidx-mountain-blue dark:bg-verusidx-turquoise-deep text-white rounded-lg hover:bg-verusidx-lake-blue dark:hover:bg-verusidx-turquoise-bright disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          {isGenerating ? 'Generating...' : `Generate New ${type === 'transparent' ? 'Address' : 'Z-Address'}`}
        </button>
      </div>
    {/if}
    {#if addresses.length === 0}
      <div class="text-center py-8 text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
        <p>No {type} addresses found</p>
        {#if onGenerateNew}
          <p class="text-sm mt-2">Click "Generate New" to create one</p>
        {/if}
      </div>
    {:else}
      <div class="space-y-3 max-h-96 overflow-y-auto">
        {#each addresses as addressData}
          {@const address = getAddressString(addressData)}
          {@const balance = getAddressBalance(addressData)}
          <div class="p-4 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium rounded-lg">
            <div class="flex justify-between items-start">
              <div class="flex-1">
                <div class="font-mono text-sm text-verusidx-stone-dark dark:text-white break-all">
                  {address}
                </div>
                {#if type === 'transparent' && balance > 0}
                  <div class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
                    Balance: {balance.toFixed(8)} {connectionState?.chainName || 'coins'}
                  </div>
                {/if}
              </div>
              <div class="flex gap-2 ml-4">
                <button
                  onclick={() => copyToClipboard(address)}
                  class="px-3 py-1 bg-verusidx-mountain-mist dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-verusidx-mountain-mist text-xs rounded hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-medium transition-colors"
                  title="Copy address"
                >
                  Copy
                </button>
                <button
                  onclick={() => onAddressClick(address)}
                  class="px-3 py-1 bg-verusidx-turquoise-deep dark:bg-verusidx-turquoise-bright text-white text-xs rounded hover:bg-verusidx-turquoise-bright dark:hover:bg-verusidx-turquoise-deep transition-colors"
                  title="View currency balances"
                >
                  View Details
                </button>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</ExpandableCard>