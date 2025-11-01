<script lang="ts">
  import { ExpandableCard } from "$lib/components/cards";

  interface Props {
    transactions: any[];
    cardClass?: string;
  }

  let { transactions, cardClass = '' }: Props = $props();

  function formatTxId(txid: string): string {
    return `${txid.slice(0, 8)}...${txid.slice(-8)}`;
  }

  function formatAmount(amount: number): string {
    const formatted = amount.toFixed(8);
    return amount >= 0 ? `+${formatted}` : formatted;
  }

  function getAmountColor(amount: number): string {
    if (amount > 0) {
      return 'text-verusidx-forest-deep dark:text-verusidx-turquoise-light';
    } else if (amount < 0) {
      return 'text-verusidx-lake-deep dark:text-verusidx-turquoise-bright';
    }
    return 'text-verusidx-stone-dark dark:text-white';
  }

  function getStatusColor(confirmations: number): string {
    if (confirmations === 0) {
      return 'text-verusidx-turquoise-bright';
    } else if (confirmations < 6) {
      return 'text-verusidx-turquoise-deep';
    }
    return 'text-verusidx-forest-deep';
  }

  function getStatusText(confirmations: number): string {
    if (confirmations === 0) {
      return 'Pending';
    } else if (confirmations < 6) {
      return `${confirmations} conf`;
    }
    return 'Confirmed';
  }

  function formatDate(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleString();
  }
</script>

<ExpandableCard title="Recent Transactions" {cardClass} modalSize="xl">
  <div slot="preview">
    {#if transactions.length === 0}
      <div class="text-center py-4 text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
        <p class="text-sm">No transactions found</p>
      </div>
    {:else}
      <div class="space-y-2">
        {#each transactions.slice(0, 5) as tx}
          <div class="flex justify-between items-center p-2 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium rounded">
            <div class="flex-1">
              <div class="font-mono text-sm text-verusidx-stone-dark dark:text-white">
                {formatTxId(tx.txid)}
              </div>
              <div class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
                {tx.category} • {getStatusText(tx.confirmations || 0)}
              </div>
            </div>
            <div class="text-right">
              <div class="font-mono text-sm {getAmountColor(tx.amount)}">
                {formatAmount(tx.amount)}
              </div>
            </div>
          </div>
        {/each}
        {#if transactions.length > 5}
          <div class="text-center text-verusidx-mountain-grey dark:text-verusidx-mountain-mist text-sm">
            And {transactions.length - 5} more...
          </div>
        {/if}
      </div>
    {/if}
  </div>

  <div slot="expanded">
    {#if transactions.length === 0}
      <div class="text-center py-8 text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
        <p>No transactions found</p>
      </div>
    {:else}
      <div class="space-y-4 max-h-96 overflow-y-auto">
        {#each transactions as tx}
          <div class="p-4 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium rounded-lg">
            <!-- Transaction Header -->
            <div class="flex justify-between items-start mb-3">
              <div class="flex-1">
                <div class="font-mono text-sm text-verusidx-stone-dark dark:text-white break-all">
                  {tx.txid}
                </div>
                <div class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
                  {formatDate(tx.time || tx.timereceived)}
                </div>
              </div>
              <div class="text-right ml-4">
                <div class="font-mono text-lg {getAmountColor(tx.amount)}">
                  {formatAmount(tx.amount)}
                </div>
                <div class="text-sm {getStatusColor(tx.confirmations || 0)}">
                  {getStatusText(tx.confirmations || 0)}
                </div>
              </div>
            </div>

            <!-- Transaction Details -->
            <div class="grid grid-cols-2 gap-4 text-sm">
              <div>
                <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Category:</span>
                <span class="ml-2 text-verusidx-stone-dark dark:text-white">{tx.category}</span>
              </div>
              <div>
                <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Confirmations:</span>
                <span class="ml-2 text-verusidx-stone-dark dark:text-white">{tx.confirmations || 0}</span>
              </div>
              {#if tx.fee}
                <div>
                  <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Fee:</span>
                  <span class="ml-2 text-verusidx-stone-dark dark:text-white">{tx.fee.toFixed(8)}</span>
                </div>
              {/if}
              {#if tx.blockhash}
                <div>
                  <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Block:</span>
                  <span class="ml-2 font-mono text-xs text-verusidx-stone-dark dark:text-white">{tx.blockhash.slice(0, 16)}...</span>
                </div>
              {/if}
            </div>

            <!-- Raw JSON Toggle -->
            <details class="mt-3">
              <summary class="cursor-pointer text-sm text-verusidx-turquoise-deep dark:text-verusidx-turquoise-light hover:underline">
                View Raw JSON
              </summary>
              <pre class="mt-2 p-3 bg-verusidx-stone-dark dark:bg-verusidx-lake-deep text-verusidx-mountain-mist text-xs rounded overflow-x-auto">
{JSON.stringify(tx, null, 2)}
              </pre>
            </details>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</ExpandableCard>