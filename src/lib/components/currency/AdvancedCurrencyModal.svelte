<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { connectionStore, getChainParam } from "$lib/stores/connection";
  import { Modal } from '../cards';
  import { showSuccess, showError } from '$lib/services/notifications';

  interface Props {
    isOpen: boolean;
    onClose: () => void;
    onSuccess?: () => void;
  }

  let { isOpen = false, onClose, onSuccess }: Props = $props();

  // Form state
  let rawJsonInput = $state('');
  let isSubmitting = $state(false);
  let error = $state<string | null>(null);
  let connectionState = $state<any>(null);

  connectionStore.subscribe(state => {
    connectionState = state;
  });

  // Example JSON for user reference
  const exampleJson = `{
  "name": "MyAdvancedCurrency",
  "options": 32,
  "proofprotocol": 1,
  "notarizationprotocol": 1,
  "currencies": [],
  "weights": [],
  "conversions": [],
  "initialsupply": 0,
  "idregistrationfees": 100,
  "idreferrallevels": 3,
  "startblock": 0,
  "minpreconversion": [],
  "maxpreconversion": [],
  "preallocations": [{"myaddress@": 1000}]
}`;

  // Reset when modal opens/closes
  $effect(() => {
    if (!isOpen) {
      resetForm();
    }
  });

  function resetForm() {
    rawJsonInput = '';
    error = null;
    isSubmitting = false;
  }

  function loadExample() {
    rawJsonInput = exampleJson;
  }

  function formatJson() {
    try {
      const parsed = JSON.parse(rawJsonInput);
      rawJsonInput = JSON.stringify(parsed, null, 2);
      error = null;
    } catch (err) {
      error = 'Invalid JSON format';
    }
  }

  // Validation
  function validateForm(): boolean {
    if (!rawJsonInput.trim()) {
      error = 'Currency definition JSON is required';
      return false;
    }

    try {
      const parsed = JSON.parse(rawJsonInput);
      
      if (!parsed.name || typeof parsed.name !== 'string') {
        error = 'Currency definition must include a "name" field';
        return false;
      }

      if (parsed.options === undefined || typeof parsed.options !== 'number') {
        error = 'Currency definition must include an "options" field (number)';
        return false;
      }

      return true;
    } catch (err) {
      error = 'Invalid JSON format';
      return false;
    }
  }

  async function handleSubmit() {
    if (!validateForm()) return;

    isSubmitting = true;
    error = null;

    try {
      // Parse the JSON input
      const currencyDefinition = JSON.parse(rawJsonInput);

      // Step 1: Define currency
      const chainParam = getChainParam(connectionState?.selectedChain);
      console.log('AdvancedCurrencyModal: Defining currency on chain:', connectionState?.selectedChain, 'param:', chainParam);
      const defineResult = await invoke('define_currency', {
        currencyDefinition,
        fractionalGateway: null,
        reserves: null,
        chain: chainParam
      });

      if (!defineResult || !(defineResult as any).hex) {
        throw new Error('Failed to get transaction hex from define currency');
      }

      // Step 2: Send raw transaction
      const sendResult = await invoke('send_raw_transaction', {
        hexData: (defineResult as any).hex,
        chain: chainParam
      });

      console.log('Advanced currency created successfully:', sendResult);

      showSuccess('Advanced currency created successfully', { txid: sendResult as string });

      if (onSuccess) {
        onSuccess();
      }
      onClose();

    } catch (err) {
      console.error('Advanced currency creation failed:', err);
      error = typeof err === 'string' ? err : 'Currency creation failed. Please check your JSON and try again.';
      showError(error);
    } finally {
      isSubmitting = false;
    }
  }
</script>

<Modal {isOpen} onclose={onClose} title="Advanced Currency Creation" size="xl">
  <div class="p-6">
    <!-- Advanced Mode Info -->
    <div class="mb-6 p-4 bg-orange-50 dark:bg-orange-900/20 border border-orange-200 dark:border-orange-800 rounded-lg">
      <p class="text-orange-700 dark:text-orange-300 text-sm mb-2">
        🔧 <strong>Advanced Mode:</strong> Direct JSON interface for the Verus definecurrency RPC command.
      </p>
      <ul class="text-orange-700 dark:text-orange-300 text-xs space-y-1 ml-4">
        <li>• Full parameter control with complete definecurrency specification</li>
        <li>• Requires knowledge of Verus currency parameters</li>
        <li>• Incorrect values may cause currency creation to fail</li>
        <li>• Refer to Verus documentation for parameter details</li>
      </ul>
    </div>

    <!-- JSON Input -->
    <div class="mb-6">
      <div class="flex items-center justify-between mb-2">
        <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white">
          Currency Definition JSON *
        </label>
        <div class="space-x-2">
          <button
            type="button"
            onclick={loadExample}
            class="text-xs text-verusidx-turquoise-deep hover:text-verusidx-turquoise-bright"
          >
            Load Example
          </button>
          <button
            type="button"
            onclick={formatJson}
            class="text-xs text-verusidx-turquoise-deep hover:text-verusidx-turquoise-bright"
          >
            Format JSON
          </button>
        </div>
      </div>
      
      <textarea 
        value={rawJsonInput}
        oninput={(e) => rawJsonInput = e.target.value}
        rows="20"
        placeholder="Enter your complete currency definition JSON here..."
        class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white font-mono text-sm"
      ></textarea>
      
      <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
        Enter a complete JSON object for the currency definition. All parameters are optional except "name" and "options".
      </p>
    </div>

    <!-- Parameter Reference -->
    <div class="mb-6">
      <details class="border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg">
        <summary class="p-3 cursor-pointer bg-verusidx-sky-soft dark:bg-verusidx-stone-medium text-sm font-medium text-verusidx-stone-dark dark:text-white">
          Available Parameters Reference
        </summary>
        <div class="p-4 bg-white dark:bg-verusidx-stone-dark">
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4 text-xs">
            <div>
              <h5 class="font-medium text-verusidx-stone-dark dark:text-white mb-2">Core Parameters:</h5>
              <ul class="space-y-1 text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
                <li><code>name</code> - Currency name (required)</li>
                <li><code>options</code> - Options bitmask (required)</li>
                <li><code>proofprotocol</code> - 1=decentralized, 2=centralized, 3=eth</li>
                <li><code>notarizationprotocol</code> - Notarization method</li>
                <li><code>idregistrationfees</code> - Sub-ID registration cost</li>
                <li><code>idreferrallevels</code> - Referral levels (0-5)</li>
              </ul>
            </div>
            <div>
              <h5 class="font-medium text-verusidx-stone-dark dark:text-white mb-2">Basket Parameters:</h5>
              <ul class="space-y-1 text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
                <li><code>currencies</code> - Reserve currency names</li>
                <li><code>weights</code> - Reserve weights (sum to 1.0)</li>
                <li><code>initialsupply</code> - Initial basket supply</li>
                <li><code>initialcontributions</code> - Initial reserves</li>
                <li><code>prelaunchcarveout</code> - Reserve percentage to ID</li>
                <li><code>idimportfees</code> - Reserve currency for sub-ID pricing</li>
              </ul>
            </div>
            <div>
              <h5 class="font-medium text-verusidx-stone-dark dark:text-white mb-2">Launch Parameters:</h5>
              <ul class="space-y-1 text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
                <li><code>startblock</code> - Launch block height</li>
                <li><code>endblock</code> - End block (centralized only)</li>
                <li><code>minpreconversion</code> - Minimum preconversion amounts</li>
                <li><code>maxpreconversion</code> - Maximum preconversion amounts</li>
                <li><code>conversions</code> - Preconversion rates</li>
                <li><code>preallocations</code> - Pre-allocated amounts</li>
              </ul>
            </div>
            <div>
              <h5 class="font-medium text-verusidx-stone-dark dark:text-white mb-2">Advanced Parameters:</h5>
              <ul class="space-y-1 text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
                <li><code>systemid</code> - System identifier</li>
                <li><code>parent</code> - Parent chain</li>
                <li><code>launchsystemid</code> - Launch system</li>
                <li><code>nativecurrencyid</code> - Native currency info</li>
                <li><code>notarizationreward</code> - Notarization reward</li>
                <li><code>blocktime</code> - Target block time</li>
              </ul>
            </div>
          </div>
        </div>
      </details>
    </div>

    <!-- Options Values Reference -->
    <div class="mb-6 p-4 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium rounded-lg">
      <h4 class="text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
        Options Values (add together)
      </h4>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4 text-xs">
        <div>
          <ul class="space-y-1 text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
            <li><code>1</code> - FRACTIONAL (basket currency)</li>
            <li><code>2</code> - Only currency ID can create sub-IDs</li>
            <li><code>8</code> - Enable referrals & discounts</li>
            <li><code>16</code> - Referrals required</li>
          </ul>
        </div>
        <div>
          <ul class="space-y-1 text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
            <li><code>32</code> - TOKEN (required for all currencies)</li>
            <li><code>2048</code> - Single satoshi NFT with ID control</li>
            <li><strong>Examples:</strong> 32=simple, 33=basket, 2080=ID control</li>
          </ul>
        </div>
      </div>
    </div>

    <!-- Error Display -->
    {#if error}
      <div class="mb-4 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
        <p class="text-red-700 dark:text-red-300">{error}</p>
      </div>
    {/if}

    <!-- Buttons -->
    <div class="flex space-x-4 pt-6 border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
      <button
        type="button"
        onclick={onClose}
        class="flex-1 px-6 py-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium text-verusidx-stone-dark dark:text-white rounded-lg hover:bg-verusidx-mountain-mist dark:hover:bg-verusidx-stone-medium transition-colors"
      >
        Cancel
      </button>
      <button
        type="button"
        onclick={handleSubmit}
        disabled={isSubmitting}
        class="flex-1 px-6 py-3 bg-verusidx-stone-dark hover:bg-verusidx-mountain-grey text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {isSubmitting ? 'Creating Currency...' : 'Create Advanced Currency'}
      </button>
    </div>
  </div>
</Modal>