<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getChainParam, connectionStore } from "$lib/stores/connection";
  import { Modal } from "$lib/components/cards";

  interface Props {
    identity: any;
    isOpen: boolean;
    onClose: () => void;
  }

  let { identity, isOpen, onClose }: Props = $props();
  
  let transparentBalances = $state<any>(null);
  let privateBalances = $state<any>(null);
  let isLoadingTransparent = $state(false);
  let isLoadingPrivate = $state(false);
  let transparentError = $state<string | null>(null);
  let privateError = $state<string | null>(null);
  let connectionState = $state<any>(null);

  connectionStore.subscribe(state => {
    connectionState = state;
  });

  async function copyToClipboard(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      console.log("Copied to clipboard:", text);
    } catch (err) {
      console.error("Failed to copy:", err);
    }
  }

  async function loadTransparentBalance() {
    if (!identity?.identity?.identityaddress) return;

    isLoadingTransparent = true;
    transparentError = null;
    
    try {
      const chainParam = getChainParam(connectionState?.selectedChain);
      const balances = await invoke("get_currency_balance", {
        address: identity.identity.identityaddress,
        minconf: 1,
        friendly_names: true,
        include_shared: true,
        chain: chainParam
      });
      transparentBalances = balances;
    } catch (err) {
      transparentError = `Failed to load transparent balance: ${err}`;
      console.error("Failed to load transparent balance:", err);
    } finally {
      isLoadingTransparent = false;
    }
  }

  async function loadPrivateBalance() {
    if (!identity?.identity?.privateaddress) return;

    isLoadingPrivate = true;
    privateError = null;
    
    try {
      const chainParam = getChainParam(connectionState?.selectedChain);
      const balances = await invoke("get_currency_balance", {
        address: identity.identity.privateaddress,
        minconf: 1,
        friendly_names: true,
        include_shared: true,
        chain: chainParam
      });
      privateBalances = balances;
    } catch (err) {
      privateError = `Failed to load private balance: ${err}`;
      console.error("Failed to load private balance:", err);
    } finally {
      isLoadingPrivate = false;
    }
  }

  // Load balances when modal opens
  $effect(() => {
    if (isOpen && identity) {
      loadTransparentBalance();
      if (identity.identity?.privateaddress) {
        loadPrivateBalance();
      }
    } else {
      // Reset state when closed
      transparentBalances = null;
      privateBalances = null;
      transparentError = null;
      privateError = null;
    }
  });

  function formatBalance(amount: number | string): string {
    const num = typeof amount === 'string' ? parseFloat(amount) : amount;
    return num.toFixed(8);
  }

  function hasPrivateAddress(): boolean {
    return !!(identity?.identity?.privateaddress && identity.identity.privateaddress.trim() !== '');
  }

  function getIdentityName(): string {
    return identity?.identity?.name || 'Unknown Identity';
  }

  function getIdentityAddress(): string {
    return identity?.identity?.identityaddress || 'Unknown';
  }

  function getPrivateAddress(): string {
    return identity?.identity?.privateaddress || '';
  }
</script>

<Modal {isOpen} onclose={onClose} size="lg" title="Identity Balance Details">
  <div class="p-6">
    <h2 class="text-2xl font-bold text-verusidx-stone-dark dark:text-white mb-4">
      {getIdentityName()}
    </h2>
    
    <!-- Identity Info -->
    <div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-4 rounded-lg mb-6">
      <div class="space-y-2">
        <div>
          <span class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">I-Address:</span>
          <span class="ml-2 font-mono text-sm text-verusidx-stone-dark dark:text-white break-all">{getIdentityAddress()}</span>
        </div>
        {#if hasPrivateAddress()}
          <div>
            <span class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Z-Address:</span>
            <span class="ml-2 font-mono text-sm text-verusidx-stone-dark dark:text-white break-all">{getPrivateAddress()}</span>
          </div>
        {/if}
      </div>
    </div>

    <!-- Transparent Balance Section -->
    <div class="mb-6">
      <div class="flex items-center justify-between mb-3">
        <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">
          Transparent Balance
        </h3>
        <button
          onclick={() => copyToClipboard(getIdentityAddress())}
          class="px-3 py-1 bg-verusidx-mountain-blue dark:bg-verusidx-turquoise-deep text-white text-sm rounded hover:bg-verusidx-lake-blue dark:hover:bg-verusidx-turquoise-bright transition-colors"
        >
          Copy I-Address
        </button>
      </div>
      
      {#if isLoadingTransparent}
        <div class="text-center py-6">
          <div class="inline-block animate-spin rounded-full h-6 w-6 border-b-2 border-verusidx-mountain-blue"></div>
          <p class="mt-2 text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Loading transparent balance...</p>
        </div>
      {:else if transparentError}
        <div class="bg-verusidx-lake-deep/10 dark:bg-verusidx-lake-deep/30 border border-verusidx-turquoise-light dark:border-verusidx-stone-medium rounded-lg p-4">
          <p class="text-verusidx-lake-deep dark:text-verusidx-turquoise-light text-sm">{transparentError}</p>
        </div>
      {:else if transparentBalances}
        <div class="space-y-2 max-h-48 overflow-y-auto">
          {#if Object.keys(transparentBalances).length === 0}
            <p class="text-center text-verusidx-mountain-grey dark:text-verusidx-mountain-mist py-4 text-sm">
              No transparent currency balances found
            </p>
          {:else}
            {#each Object.entries(transparentBalances) as [currency, amount]}
              <div class="flex justify-between items-center p-3 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium rounded-lg">
                <span class="font-medium text-verusidx-stone-dark dark:text-white">
                  {currency}
                </span>
                <span class="font-mono text-verusidx-stone-dark dark:text-white">
                  {formatBalance(amount as number)}
                </span>
              </div>
            {/each}
          {/if}
        </div>
      {:else}
        <p class="text-center text-verusidx-mountain-grey dark:text-verusidx-mountain-mist py-4 text-sm">
          No transparent balance data available
        </p>
      {/if}
    </div>

    <!-- Private Balance Section (only if z-address exists) -->
    {#if hasPrivateAddress()}
      <div class="mb-6">
        <div class="flex items-center justify-between mb-3">
          <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">
            Private Balance
          </h3>
          <button
            onclick={() => copyToClipboard(getPrivateAddress())}
            class="px-3 py-1 bg-verusidx-mountain-blue dark:bg-verusidx-turquoise-deep text-white text-sm rounded hover:bg-verusidx-lake-blue dark:hover:bg-verusidx-turquoise-bright transition-colors"
          >
            Copy Z-Address
          </button>
        </div>
        
        {#if isLoadingPrivate}
          <div class="text-center py-6">
            <div class="inline-block animate-spin rounded-full h-6 w-6 border-b-2 border-verusidx-mountain-blue"></div>
            <p class="mt-2 text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Loading private balance...</p>
          </div>
        {:else if privateError}
          <div class="bg-verusidx-lake-deep/10 dark:bg-verusidx-lake-deep/30 border border-verusidx-turquoise-light dark:border-verusidx-stone-medium rounded-lg p-4">
            <p class="text-verusidx-lake-deep dark:text-verusidx-turquoise-light text-sm">{privateError}</p>
          </div>
        {:else if privateBalances}
          <div class="space-y-2 max-h-48 overflow-y-auto">
            {#if Object.keys(privateBalances).length === 0}
              <p class="text-center text-verusidx-mountain-grey dark:text-verusidx-mountain-mist py-4 text-sm">
                No private currency balances found
              </p>
            {:else}
              {#each Object.entries(privateBalances) as [currency, amount]}
                <div class="flex justify-between items-center p-3 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium rounded-lg">
                  <span class="font-medium text-verusidx-stone-dark dark:text-white">
                    {currency}
                  </span>
                  <span class="font-mono text-verusidx-stone-dark dark:text-white">
                    {formatBalance(amount as number)}
                  </span>
                </div>
              {/each}
            {/if}
          </div>
        {:else}
          <p class="text-center text-verusidx-mountain-grey dark:text-verusidx-mountain-mist py-4 text-sm">
            No private balance data available
          </p>
        {/if}
      </div>
    {/if}

    <!-- Close Button -->
    <div class="mt-6 flex justify-end">
      <button
        onclick={onClose}
        class="px-6 py-2 bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-stone-medium dark:text-verusidx-mountain-mist rounded-lg hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-dark transition-colors"
      >
        Close
      </button>
    </div>
  </div>
</Modal>