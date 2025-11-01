<script lang="ts">
  import { ExpandableCard } from "$lib/components/cards";
  import { connectionStore } from "$lib/stores/connection";
  import { goto } from "$app/navigation";

  let connectionState = $state<any>(null);
  
  connectionStore.subscribe(state => {
    connectionState = state;  // Fix: use full state to access selectedChain
  });

  interface Props {
    walletInfo: any;
    cardClass?: string;
  }

  let { walletInfo, cardClass = '' }: Props = $props();


  // Get currency entries for display
  let currencyEntries = $derived(() => {
    if (!walletInfo) return [];
    
    const entries: Array<{currency: string; amount: number; isNative: boolean}> = [];
    
    // Add native balance
    if (walletInfo.balance !== undefined) {
      const chainName = connectionState?.current?.chainName || 'VRSC';
      entries.push({
        currency: `${chainName} Balance`,
        amount: walletInfo.balance,
        isNative: true
      });
    }
    
    // Add reserve currencies
    if (walletInfo.reserve_balance && Object.keys(walletInfo.reserve_balance).length > 0) {
      for (const [currency, amount] of Object.entries(walletInfo.reserve_balance)) {
        entries.push({
          currency,
          amount: amount as number,
          isNative: false
        });
      }
    }
    
    return entries;
  });

  // Create dynamic title with count
  let cardTitle = $derived(() => {
    const count = currencyEntries().length;
    return `Currencies (${count})`;
  });

  // Navigation functions for defi operations
  function navigateToSendTransparent() {
    goto("/defi?operation=send-transparent");
  }

  function navigateToSendPrivate() {
    goto("/defi?operation=send-private");
  }
</script>

<ExpandableCard title={cardTitle()} {cardClass} modalSize="lg">
  <!-- Preview Content -->
  <div slot="preview">
    <!-- Send Buttons -->
    <div class="flex gap-2 mb-4">
      <button
        onclick={navigateToSendTransparent}
        class="flex-1 px-3 py-2 bg-verusidx-mountain-blue text-white text-sm rounded-lg hover:bg-verusidx-lake-blue transition-colors"
      >
        Send from Transparent
      </button>
      <button
        onclick={navigateToSendPrivate}
        class="flex-1 px-3 py-2 bg-verusidx-turquoise-deep text-white text-sm rounded-lg hover:bg-verusidx-turquoise-bright transition-colors"
      >
        Send from Private
      </button>
    </div>
    {#if currencyEntries().length === 0}
      <div class="text-center py-4 text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
        <p class="text-sm">No currencies found</p>
      </div>
    {:else}
      <div class="space-y-3">
        {#each currencyEntries().slice(0, 3) as entry}
          <div class="flex justify-between items-center p-3 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium rounded-lg">
            <span class="font-medium text-verusidx-stone-dark dark:text-white">
              {entry.currency}
            </span>
            <span class="font-mono text-verusidx-stone-dark dark:text-white">
              {entry.amount.toFixed(8)}
            </span>
          </div>
        {/each}
        {#if currencyEntries().length > 3}
          <div class="text-center text-verusidx-mountain-grey dark:text-verusidx-mountain-mist text-sm">
            And {currencyEntries().length - 3} more currencies...
          </div>
        {/if}
      </div>
    {/if}
  </div>

  <!-- Expanded Content -->
  <div slot="expanded">
    <!-- Send Buttons in Expanded View -->
    <div class="flex gap-2 mb-6">
      <button
        onclick={navigateToSendTransparent}
        class="flex-1 px-4 py-2 bg-verusidx-mountain-blue text-white rounded-lg hover:bg-verusidx-lake-blue transition-colors"
      >
        Send from Transparent
      </button>
      <button
        onclick={navigateToSendPrivate}
        class="flex-1 px-4 py-2 bg-verusidx-turquoise-deep text-white rounded-lg hover:bg-verusidx-turquoise-bright transition-colors"
      >
        Send from Private
      </button>
    </div>
    {#if currencyEntries().length === 0}
      <div class="text-center py-8 text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
        <p>No currencies found</p>
      </div>
    {:else}
      <div class="space-y-4">
        <!-- Native Balance Section -->
        {#if currencyEntries().some((entry: any) => entry.isNative)}
          <div class="mb-6">
            <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-3">
              Native Balance
            </h3>
            {#each currencyEntries().filter((entry: any) => entry.isNative) as entry}
              <div class="p-4 bg-verusidx-turquoise-deep text-white rounded-lg">
                <div class="text-sm opacity-90 mb-1">{entry.currency}</div>
                <div class="text-2xl font-bold">{entry.amount.toFixed(8)}</div>
              </div>
            {/each}
          </div>
        {/if}

        <!-- Reserve Currencies Section -->
        {#if currencyEntries().some((entry: any) => !entry.isNative)}
          <div>
            <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-3">
              Reserve Currencies
            </h3>
            <div class="space-y-3 max-h-96 overflow-y-auto">
              {#each currencyEntries().filter((entry: any) => !entry.isNative) as entry}
                <div class="flex justify-between items-center p-4 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium rounded-lg">
                  <span class="font-medium text-verusidx-stone-dark dark:text-white">
                    {entry.currency}
                  </span>
                  <span class="font-mono text-verusidx-stone-dark dark:text-white">
                    {entry.amount.toFixed(8)}
                  </span>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</ExpandableCard>