<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Modal } from './cards';
  import { connectionStore, getChainParam } from "$lib/stores/connection";
  import { showSuccess, showError } from "$lib/services/notifications";

  interface Props {
    isOpen: boolean;
    onClose: () => void;
    onSuccess: () => void;
  }

  let { isOpen = false, onClose, onSuccess }: Props = $props();

  // Form state
  let formData = $state({
    fromAddress: '',
    toAddress: '',
    currency: '',
    amount: ''
  });

  // Available addresses and currencies
  let fromAddresses = $state<string[]>([]);
  let toAddresses = $state<string[]>([]);
  let currencies = $state<{name: string, balance: number}[]>([]);

  // Loading states
  let isLoadingFromAddresses = $state(false);
  let isLoadingToAddresses = $state(false);
  let isLoadingCurrencies = $state(false);
  let isSubmitting = $state(false);
  let error = $state<string | null>(null);

  // Operation tracking
  let operationId = $state<string | null>(null);
  let opStatus = $state<string | null>(null);
  let isCheckingStatus = $state(false);

  // Track what we've loaded
  let hasLoadedFromAddresses = $state(false);
  let hasLoadedToAddresses = $state(false);
  let lastLoadedFromAddress = $state<string | null>(null);

  // Connection state
  let connectionState = $state<any>(null);

  connectionStore.subscribe(state => {
    connectionState = state;
  });

  // Reset form when modal closes
  $effect(() => {
    if (!isOpen) {
      resetForm();
    }
  });

  // Load addresses when modal opens
  $effect(() => {
    if (isOpen && !hasLoadedFromAddresses) {
      loadFromAddresses();
    }
  });

  // Load to addresses when modal opens
  $effect(() => {
    if (isOpen && !hasLoadedToAddresses) {
      loadToAddresses();
    }
  });

  // Load currencies when fromAddress changes
  $effect(() => {
    if (formData.fromAddress && formData.fromAddress !== '' &&
        !formData.fromAddress.includes('*') && // Skip wildcards
        lastLoadedFromAddress !== formData.fromAddress) {
      loadCurrencies();
    } else if (!formData.fromAddress || formData.fromAddress === '' || formData.fromAddress.includes('*')) {
      currencies = [];
      lastLoadedFromAddress = null;
    }
  });

  function resetForm() {
    formData = {
      fromAddress: '',
      toAddress: '',
      currency: '',
      amount: ''
    };
    error = null;
    operationId = null;
    opStatus = null;
    hasLoadedFromAddresses = false;
    hasLoadedToAddresses = false;
    lastLoadedFromAddress = null;
    fromAddresses = [];
    toAddresses = [];
    currencies = [];
  }

  async function loadFromAddresses() {
    if (isLoadingFromAddresses) return;

    isLoadingFromAddresses = true;

    try {
      const addresses: string[] = [];

      // Get addresses from listaddressgroupings
      try {
        const groupings = await invoke('list_address_groupings') as any[];
        for (const group of groupings) {
          if (Array.isArray(group)) {
            for (const addressInfo of group) {
              if (addressInfo && typeof addressInfo === 'object' && addressInfo.address) {
                addresses.push(addressInfo.address);
              }
            }
          }
        }
      } catch (err) {
        console.error('Failed to load from addresses:', err);
      }

      // Remove duplicates and sort
      fromAddresses = [...new Set(addresses)].sort();
      hasLoadedFromAddresses = true;

    } catch (err) {
      console.error('Critical error in loadFromAddresses:', err);
      error = `Failed to load addresses: ${err}`;
    } finally {
      isLoadingFromAddresses = false;
    }
  }

  async function loadToAddresses() {
    if (isLoadingToAddresses) return;

    isLoadingToAddresses = true;

    try {
      const addresses = new Set<string>();

      // Get addresses from getaddressesbyaccount
      try {
        const accountAddresses = await invoke("get_addresses_by_account", {
          account: ""
        }) as string[];
        accountAddresses.forEach(addr => addresses.add(addr));
      } catch (error) {
        console.warn("Failed to get addresses by account:", error);
      }

      // Get addresses from listaddressgroupings
      try {
        const groupings = await invoke("list_address_groupings") as any[];
        for (const group of groupings) {
          for (const addressInfo of group) {
            if (addressInfo && typeof addressInfo === 'object' && addressInfo.address) {
              addresses.add(addressInfo.address);
            }
          }
        }
      } catch (error) {
        console.warn("Failed to get address groupings:", error);
      }

      toAddresses = Array.from(addresses).sort();
      hasLoadedToAddresses = true;

    } catch (err) {
      console.error('Failed to load to addresses:', err);
      error = `Failed to load addresses: ${err}`;
    } finally {
      isLoadingToAddresses = false;
    }
  }

  async function loadCurrencies() {
    if (isLoadingCurrencies || !formData.fromAddress) return;

    isLoadingCurrencies = true;

    try {
      const result = await invoke('get_currency_balance', {
        address: formData.fromAddress
      });

      if (result && typeof result === 'object') {
        const currencyList = Object.entries(result).map(([name, balance]) => ({
          name,
          balance: balance as number
        }));

        // Filter to only show VRSC and vUSDC.vETH
        currencies = currencyList.filter(c =>
          c.name === 'VRSC' ||
          c.name === 'vUSDC.vETH' ||
          c.name.toLowerCase() === 'vrsc' ||
          c.name.toLowerCase() === 'vusdc.veth'
        );

        lastLoadedFromAddress = formData.fromAddress;
      }
    } catch (err) {
      console.error('Failed to load currencies:', err);
      currencies = [];
    } finally {
      isLoadingCurrencies = false;
    }
  }

  async function handleSubmit(event: SubmitEvent) {
    event.preventDefault();

    // Validation
    if (!formData.fromAddress || !formData.toAddress) {
      error = 'Please select both from and to addresses';
      return;
    }

    if (!formData.currency) {
      error = 'Please select a currency to convert';
      return;
    }

    if (!formData.amount || parseFloat(formData.amount) <= 0) {
      error = 'Please enter a valid amount';
      return;
    }

    isSubmitting = true;
    error = null;

    try {
      // Build sendcurrency parameters for reserve-to-basket conversion
      const output = {
        currency: formData.currency,
        amount: parseFloat(formData.amount),
        address: formData.toAddress,
        convertto: "verusidx" // Hardcoded target currency
      };

      // Call sendcurrency RPC
      const chainParam = getChainParam(connectionState?.selectedChain);
      const result = await invoke('send_currency', {
        fromAddress: formData.fromAddress,
        outputs: [output],
        chain: chainParam
      });

      // Store operation ID for status checking
      operationId = typeof result === 'string' ? result : String(result);

      // Show success message
      showSuccess('Conversion submitted successfully');
      onSuccess();

    } catch (err) {
      error = typeof err === 'string' ? err : 'Conversion failed';
      console.error('Conversion error:', err);
      showError(error);
    } finally {
      isSubmitting = false;
    }
  }

  async function refreshOperationStatus() {
    if (!operationId) return;

    isCheckingStatus = true;

    try {
      const chainParam = getChainParam(connectionState?.selectedChain);
      const result = await invoke('z_get_operation_status', {
        operationIds: [operationId],
        chain: chainParam
      });

      if (result && Array.isArray(result) && result[0]) {
        opStatus = result[0].status; // "queued" | "executing" | "success" | "failed"
        console.log('Operation status:', opStatus);
      }
    } catch (err) {
      console.error('Failed to check operation status:', err);
      error = `Failed to check status: ${err}`;
    } finally {
      isCheckingStatus = false;
    }
  }

  async function copyToClipboard(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      console.log('Copied to clipboard:', text);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  }
</script>

<Modal {isOpen} onclose={onClose} title="Convert to VerusIDX" size="lg">
  <div class="p-6">
    {#if !operationId}
      <!-- Conversion Form -->
      <div class="flex items-center space-x-3 mb-6">
        <span class="text-3xl">🔄</span>
        <div>
          <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
            Convert VRSC or vUSDC.vETH to VerusIDX
          </p>
        </div>
      </div>

      <!-- Error Display -->
      {#if error}
        <div class="mb-4 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
          <p class="text-red-700 dark:text-red-300">{error}</p>
        </div>
      {/if}

      <!-- Form -->
      <form onsubmit={handleSubmit} class="space-y-4">
        <!-- From Address -->
        <div>
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
            From Address *
          </label>
          <select
            bind:value={formData.fromAddress}
            required
            disabled={isLoadingFromAddresses}
            class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
          >
            <option value="">{isLoadingFromAddresses ? 'Loading addresses...' : 'Select address to send from...'}</option>
            {#each fromAddresses as address}
              <option value={address}>{address}</option>
            {/each}
          </select>
          <p class="mt-1 text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
            Select the specific address to send from
          </p>
        </div>

        <!-- To Address -->
        <div>
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
            To Address (receiving address) *
          </label>
          {#if isLoadingToAddresses}
            <div class="w-full px-3 py-2 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium bg-white dark:bg-verusidx-stone-medium text-verusidx-stone-dark dark:text-white rounded-lg">
              <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Loading addresses...</span>
            </div>
          {:else if toAddresses.length > 0}
            <select
              bind:value={formData.toAddress}
              required
              class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white rounded-lg"
            >
              <option value="">Select receiving address...</option>
              {#each toAddresses as address}
                <option value={address}>{address}</option>
              {/each}
            </select>
          {:else}
            <p class="text-sm text-verusidx-lake-deep dark:text-verusidx-turquoise-light">No addresses found.</p>
          {/if}
          <p class="mt-1 text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
            Select the specific address to receive the converted VerusIDX
          </p>
        </div>

        <!-- Currency -->
        <div>
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
            Currency to Convert *
          </label>
          <select
            bind:value={formData.currency}
            required
            disabled={isLoadingCurrencies || !formData.fromAddress || formData.fromAddress.includes('*')}
            class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white disabled:opacity-50"
          >
            <option value="">
              {isLoadingCurrencies ? 'Loading currencies...' :
               formData.fromAddress.includes('*') ? 'Select a specific address first...' :
               !formData.fromAddress ? 'Select from address first...' :
               'Select currency...'}
            </option>
            {#each currencies as currency}
              <option value={currency.name}>{currency.name} (Balance: {currency.balance.toFixed(8)})</option>
            {/each}
          </select>
          <p class="mt-1 text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
            Only VRSC and vUSDC.vETH can be converted to VerusIDX
          </p>
        </div>

        <!-- Amount -->
        <div>
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
            Amount *
          </label>
          <input
            type="number"
            step="any"
            bind:value={formData.amount}
            required
            placeholder="Enter amount to convert"
            class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
          />
          <p class="mt-1 text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
            Enter the amount you want to convert
          </p>
        </div>

        <!-- Convert To (shown for info) -->
        <div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded-lg">
          <div class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
            <div class="flex justify-between">
              <span>Converting to:</span>
              <span class="font-semibold text-verusidx-stone-dark dark:text-white">VerusIDX</span>
            </div>
          </div>
        </div>

        <!-- Action Buttons -->
        <div class="flex space-x-3 pt-4">
          <button
            type="button"
            onclick={onClose}
            class="flex-1 px-4 py-2 bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-stone-medium dark:text-verusidx-mountain-mist rounded-lg hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-light transition-colors"
          >
            Cancel
          </button>
          <button
            type="submit"
            disabled={isSubmitting}
            class="flex-1 px-4 py-2 bg-verusidx-mountain-blue dark:bg-verusidx-turquoise-deep text-white rounded-lg hover:bg-verusidx-lake-blue dark:hover:bg-verusidx-turquoise-bright transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isSubmitting ? 'Converting...' : 'Convert'}
          </button>
        </div>
      </form>
    {:else}
      <!-- Success Panel with Operation Status -->
      <div class="space-y-6">
        <div class="flex items-center space-x-3">
          <svg class="w-8 h-8 text-verusidx-forest-deep dark:text-verusidx-turquoise-light" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"></path>
          </svg>
          <div>
            <h3 class="text-lg font-semibold text-verusidx-forest-deep dark:text-verusidx-turquoise-light">
              Conversion Transaction Submitted
            </h3>
          </div>
        </div>

        <!-- Operation ID -->
        <div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium rounded-lg p-4">
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
            Operation ID:
          </label>
          <div class="flex items-center space-x-2">
            <code class="flex-1 text-xs font-mono bg-white dark:bg-verusidx-stone-dark p-2 rounded border border-verusidx-mountain-mist dark:border-verusidx-stone-light break-all text-verusidx-stone-dark dark:text-white">
              {operationId}
            </code>
            <button
              onclick={() => copyToClipboard(operationId || '')}
              class="px-3 py-2 bg-verusidx-mountain-blue dark:bg-verusidx-turquoise-deep text-white text-sm rounded hover:bg-verusidx-lake-blue dark:hover:bg-verusidx-turquoise-bright transition-colors"
            >
              Copy
            </button>
          </div>
        </div>

        <!-- Operation Status -->
        <div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium rounded-lg p-4">
          <div class="flex items-center justify-between mb-2">
            <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white">
              Operation Status:
            </label>
            <button
              onclick={refreshOperationStatus}
              disabled={isCheckingStatus}
              class="px-3 py-1 text-sm bg-verusidx-mountain-blue dark:bg-verusidx-turquoise-deep text-white rounded hover:bg-verusidx-lake-blue dark:hover:bg-verusidx-turquoise-bright transition-colors disabled:opacity-50"
            >
              {isCheckingStatus ? 'Checking...' : 'Refresh Status'}
            </button>
          </div>
          {#if opStatus}
            <div class="mt-2">
              {#if opStatus === 'queued'}
                <span class="px-3 py-1 bg-yellow-100 dark:bg-yellow-900/30 text-yellow-800 dark:text-yellow-300 rounded text-sm">
                  ⏳ Queued
                </span>
              {:else if opStatus === 'executing'}
                <span class="px-3 py-1 bg-blue-100 dark:bg-blue-900/30 text-blue-800 dark:text-blue-300 rounded text-sm">
                  ⚙️ Executing
                </span>
              {:else if opStatus === 'success'}
                <span class="px-3 py-1 bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-300 rounded text-sm">
                  ✓ Success
                </span>
              {:else if opStatus === 'failed'}
                <span class="px-3 py-1 bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-300 rounded text-sm">
                  ✗ Failed
                </span>
              {:else}
                <span class="px-3 py-1 bg-gray-100 dark:bg-gray-900/30 text-gray-800 dark:text-gray-300 rounded text-sm">
                  {opStatus}
                </span>
              {/if}
            </div>
          {:else}
            <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
              Not checked yet - click "Refresh Status" to check
            </p>
          {/if}
        </div>

        <!-- Instructions -->
        <div class="bg-verusidx-turquoise-bright/10 dark:bg-verusidx-lake-deep/50 border border-verusidx-turquoise-light dark:border-verusidx-turquoise-light rounded-lg p-4">
          <p class="text-sm text-verusidx-turquoise-deep dark:text-verusidx-turquoise-light">
            Once the operation status shows "success", your conversion will complete in 2-10 blocks (~2-10 minutes).
            You can check your balance using the "Refresh Balances" button in Step 1.
          </p>
        </div>

        <!-- Close Button -->
        <button
          onclick={onClose}
          class="w-full px-6 py-3 bg-verusidx-mountain-blue dark:bg-verusidx-turquoise-deep text-white rounded-lg hover:bg-verusidx-lake-blue dark:hover:bg-verusidx-turquoise-bright transition-colors"
        >
          Close
        </button>
      </div>
    {/if}
  </div>
</Modal>
