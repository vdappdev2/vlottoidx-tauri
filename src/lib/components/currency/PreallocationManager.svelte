<script lang="ts">
  interface Preallocation {
    address: string;
    amount: number;
  }

  interface Props {
    preallocations: Preallocation[];
    onUpdate: (preallocations: Preallocation[]) => void;
    isIdControlToken?: boolean;
    error?: string;
  }

  let { preallocations, onUpdate, isIdControlToken = false, error }: Props = $props();

  // For ID Control Tokens, ensure we have exactly one entry with 0.00000001
  $effect(() => {
    if (isIdControlToken && preallocations.length === 0) {
      onUpdate([{ address: '', amount: 0.00000001 }]);
    }
  });

  function addPreallocation() {
    if (isIdControlToken) return; // ID Control Tokens can only have one preallocation
    
    const newPreallocations = [...preallocations, { address: '', amount: 0 }];
    onUpdate(newPreallocations);
  }

  function removePreallocation(index: number) {
    if (isIdControlToken) return; // ID Control Tokens must have exactly one preallocation
    
    const newPreallocations = preallocations.filter((_, i) => i !== index);
    onUpdate(newPreallocations);
  }

  function updateAddress(index: number, address: string) {
    const updated = [...preallocations];
    updated[index] = { ...updated[index], address };
    onUpdate(updated);
  }

  function updateAmount(index: number, amount: number) {
    const updated = [...preallocations];
    if (isIdControlToken) {
      // For ID Control Tokens, amount is always 0.00000001
      updated[index] = { ...updated[index], amount: 0.00000001 };
    } else {
      updated[index] = { ...updated[index], amount: amount || 0 };
    }
    onUpdate(updated);
  }
</script>

<div class="space-y-4">
  <div>
    <div class="flex items-center justify-between mb-2">
      <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white">
        {#if isIdControlToken}
          ID Control Token Recipient *
        {:else}
          Preallocations
        {/if}
      </label>
      {#if !isIdControlToken && preallocations.length < 10}
        <button
          type="button"
          onclick={addPreallocation}
          class="text-xs text-verusidx-turquoise-deep hover:text-verusidx-turquoise-bright"
        >
          + Add preallocation
        </button>
      {/if}
    </div>

    <div class="space-y-3">
      {#each preallocations as preallocation, index}
        <div class="flex space-x-2 items-start">
          <!-- Address Input -->
          <div class="flex-1">
            <label class="block text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-1">
              {#if isIdControlToken}
                Recipient Address (ID@ or R-address)
              {:else}
                Address {index + 1}
              {/if}
            </label>
            <input 
              type="text" 
              value={preallocation.address}
              oninput={(e) => updateAddress(index, e.target.value)}
              placeholder={isIdControlToken ? "alice@ or RXXXxxxXXX..." : "alice@ or RXXXxxxXXX..."}
              required={isIdControlToken}
              class="w-full p-2 text-sm border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
            />
          </div>

          <!-- Amount Input -->
          <div class="{isIdControlToken ? 'w-32' : 'w-28'}">
            <label class="block text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-1">
              Amount
            </label>
            <input 
              type="number" 
              value={preallocation.amount}
              oninput={(e) => updateAmount(index, parseFloat(e.target.value) || 0)}
              min="0"
              step="0.00000001"
              placeholder="0.00000000"
              disabled={isIdControlToken}
              required={!isIdControlToken}
              class="w-full p-2 text-sm border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50 disabled:cursor-not-allowed"
            />
          </div>

          <!-- Remove Button -->
          {#if !isIdControlToken && preallocations.length > 0}
            <div class="pt-6">
              <button
                type="button"
                onclick={() => removePreallocation(index)}
                class="px-2 py-2 text-xs text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-colors"
                title="Remove preallocation"
              >
                ×
              </button>
            </div>
          {/if}
        </div>
      {/each}

      {#if preallocations.length === 0 && !isIdControlToken}
        <div class="text-center py-4 text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
          <p class="text-sm">No preallocations configured</p>
          <button
            type="button"
            onclick={addPreallocation}  
            class="mt-2 text-sm text-verusidx-turquoise-deep hover:text-verusidx-turquoise-bright"
          >
            + Add your first preallocation
          </button>
        </div>
      {/if}
    </div>

    <!-- Validation Messages -->
    <div class="mt-2 space-y-1">
      {#if error}
        <p class="text-xs text-red-600 dark:text-red-400">{error}</p>
      {/if}

      {#if isIdControlToken}
        <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
          ID Control Tokens must allocate exactly 0.00000001 (1 satoshi) to a single recipient. 
          The holder of this token gains revoke/recover authority over the currency's root identity.
        </p>
      {:else}
        <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
          Preallocations distribute currency directly to specified addresses at launch. 
          For basket currencies, this lowers the reserve ratio.
        </p>
      {/if}
    </div>
  </div>
</div>