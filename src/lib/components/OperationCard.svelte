<script lang="ts">
  import { ExpandableCard } from "$lib/components/cards";

  interface Props {
    operations: any[];
    cardClass?: string;
    onRefresh?: () => void;
  }

  let { operations, cardClass = '', onRefresh }: Props = $props();

  function formatOpId(opid: string): string {
    return `${opid.slice(0, 8)}...${opid.slice(-8)}`;
  }

  function getStatusColor(status: string): string {
    switch (status.toLowerCase()) {
      case 'success':
        return 'text-verusidx-forest-deep dark:text-verusidx-turquoise-light';
      case 'failed':
        return 'text-verusidx-lake-deep dark:text-verusidx-turquoise-bright';
      case 'executing':
        return 'text-verusidx-turquoise-deep dark:text-verusidx-turquoise-bright';
      case 'queued':
        return 'text-verusidx-mountain-grey dark:text-verusidx-mountain-mist';
      default:
        return 'text-verusidx-stone-dark dark:text-white';
    }
  }

  function getStatusIcon(status: string): string {
    switch (status.toLowerCase()) {
      case 'success':
        return '✓';
      case 'failed':
        return '✗';
      case 'executing':
        return '⏳';
      case 'queued':
        return '⏸';
      default:
        return '?';
    }
  }

  function formatCreationTime(creationTime: number): string {
    return new Date(creationTime * 1000).toLocaleString();
  }

  function getOperationType(operation: any): string {
    return operation.method || 'Unknown';
  }
</script>

<ExpandableCard title="Recent Operations" {cardClass} modalSize="xl" {onRefresh}>
  <div slot="preview">
    {#if operations.length === 0}
      <div class="text-center py-4 text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
        <p class="text-sm">No pending operations</p>
      </div>
    {:else}
      <div class="space-y-2">
        {#each operations.slice(0, 5) as op}
          <div class="flex justify-between items-center p-2 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium rounded">
            <div class="flex-1">
              <div class="font-mono text-sm text-verusidx-stone-dark dark:text-white">
                {formatOpId(op.id)}
              </div>
              <div class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
                {getOperationType(op)}
              </div>
            </div>
            <div class="text-right">
              <div class="text-sm {getStatusColor(op.status)} flex items-center">
                <span class="mr-1">{getStatusIcon(op.status)}</span>
                {op.status}
              </div>
            </div>
          </div>
        {/each}
        {#if operations.length > 5}
          <div class="text-center text-verusidx-mountain-grey dark:text-verusidx-mountain-mist text-sm">
            And {operations.length - 5} more...
          </div>
        {/if}
      </div>
    {/if}
  </div>

  <div slot="expanded">
    {#if operations.length === 0}
      <div class="text-center py-8 text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
        <p>No pending operations</p>
      </div>
    {:else}
      <div class="space-y-4 max-h-96 overflow-y-auto">
        {#each operations as op}
          <div class="p-4 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium rounded-lg">
            <!-- Operation Header -->
            <div class="flex justify-between items-start mb-3">
              <div class="flex-1">
                <div class="font-mono text-sm text-verusidx-stone-dark dark:text-white break-all">
                  {op.id}
                </div>
                <div class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
                  {getOperationType(op)}
                </div>
              </div>
              <div class="text-right ml-4">
                <div class="text-lg {getStatusColor(op.status)} flex items-center">
                  <span class="mr-2">{getStatusIcon(op.status)}</span>
                  {op.status}
                </div>
              </div>
            </div>

            <!-- Operation Details -->
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
              {#if op.creation_time}
                <div>
                  <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Created:</span>
                  <span class="ml-2 text-verusidx-stone-dark dark:text-white">{formatCreationTime(op.creation_time)}</span>
                </div>
              {/if}
              {#if op.result && op.result.txid}
                <div>
                  <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">TxID:</span>
                  <span class="ml-2 font-mono text-xs text-verusidx-stone-dark dark:text-white">{op.result.txid.slice(0, 16)}...</span>
                </div>
              {/if}
              {#if op.error}
                <div class="col-span-2">
                  <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Error:</span>
                  <div class="mt-1 p-2 bg-verusidx-lake-deep/10 text-verusidx-lake-deep dark:text-verusidx-turquoise-bright rounded">
                    {op.error.message || op.error}
                  </div>
                </div>
              {/if}
            </div>

            <!-- Result Details -->
            {#if op.result && op.status === 'success'}
              <div class="mt-3 p-3 bg-verusidx-forest-deep/10 rounded">
                <div class="text-sm text-verusidx-forest-deep dark:text-verusidx-turquoise-light font-medium mb-2">
                  Operation Result:
                </div>
                {#if op.result.txid}
                  <div class="text-sm">
                    <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Transaction ID:</span>
                    <span class="ml-2 font-mono text-verusidx-stone-dark dark:text-white">{op.result.txid}</span>
                  </div>
                {/if}
                {#if op.result.value}
                  <div class="text-sm">
                    <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Value:</span>
                    <span class="ml-2 text-verusidx-stone-dark dark:text-white">{op.result.value}</span>
                  </div>
                {/if}
              </div>
            {/if}

            <!-- Raw JSON Toggle -->
            <details class="mt-3">
              <summary class="cursor-pointer text-sm text-verusidx-turquoise-deep dark:text-verusidx-turquoise-light hover:underline">
                View Raw JSON
              </summary>
              <pre class="mt-2 p-3 bg-verusidx-stone-dark dark:bg-verusidx-lake-deep text-verusidx-mountain-mist text-xs rounded overflow-x-auto">
{JSON.stringify(op, null, 2)}
              </pre>
            </details>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</ExpandableCard>