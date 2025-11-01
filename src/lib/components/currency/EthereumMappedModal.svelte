<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { connectionStore, getChainParam } from "$lib/stores/connection";
  import { Modal } from '../cards';

  interface Props {
    isOpen: boolean;
    onClose: () => void;
    onSuccess?: () => void;
  }

  let { isOpen = false, onClose, onSuccess }: Props = $props();

  // Form state - simplified for Ethereum mapping
  let formData = $state({
    name: '',
    contractAddress: ''
  });

  let isSubmitting = $state(false);
  let error = $state<string | null>(null);
  let connectionState = $state<any>(null);

  connectionStore.subscribe(state => {
    connectionState = state;
  });

  // Reset when modal opens/closes
  $effect(() => {
    if (!isOpen) {
      resetForm();
    }
  });

  function resetForm() {
    formData = {
      name: '',
      contractAddress: ''
    };
    error = null;
    isSubmitting = false;
  }

  // Validation
  function validateForm(): boolean {
    if (!formData.name.trim()) {
      error = 'Currency name is required';
      return false;
    }

    if (!formData.contractAddress.trim()) {
      error = 'Contract address is required';
      return false;
    }

    // Basic Ethereum address validation
    if (!/^0x[a-fA-F0-9]{40}$/.test(formData.contractAddress)) {
      error = 'Contract address must be a valid Ethereum address (0x followed by 40 hex characters)';
      return false;
    }

    return true;
  }

  async function handleSubmit() {
    if (!validateForm()) return;

    isSubmitting = true;
    error = null;

    try {
      // Build currency definition for Ethereum-mapped token
      const currencyDefinition: any = {
        name: formData.name,
        options: 32,
        systemid: "veth",
        parent: "vrsc", 
        launchsystemid: "vrsc",
        proofprotocol: 3,
        nativecurrencyid: {
          type: 9,
          address: formData.contractAddress
        },
        initialsupply: 0
      };

      // Step 1: Define currency
      const chainParam = getChainParam(connectionState?.selectedChain);
      console.log('EthereumMappedModal: Defining currency on chain:', connectionState?.selectedChain, 'param:', chainParam);
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

      console.log('Ethereum-mapped currency created successfully:', sendResult);
      
      if (onSuccess) {
        onSuccess();
      }
      onClose();

    } catch (err) {
      console.error('Ethereum-mapped currency creation failed:', err);
      error = typeof err === 'string' ? err : 'Currency creation failed. Please try again.';
    } finally {
      isSubmitting = false;
    }
  }
</script>

<Modal {isOpen} onclose={onClose} title="Create Ethereum-Mapped Currency" size="lg">
  <div class="p-6">
    <!-- Ethereum Mapping Info -->
    <div class="mb-6 p-4 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
      <p class="text-blue-700 dark:text-blue-300 text-sm mb-2">
        🌉 <strong>Ethereum-Mapped Currency:</strong> Create a 1:1 mapping of an existing ERC-20, ERC-721, or ERC-1155 token to Verus.
      </p>
      <ul class="text-blue-700 dark:text-blue-300 text-xs space-y-1 ml-4">
        <li>• Tokens can be bridged between Ethereum and Verus</li>
        <li>• Non-custodial bridge with consensus verification</li>
        <li>• Takes advantage of Verus L1 features like conversions</li>
      </ul>
    </div>

    <!-- Currency Name -->
    <div class="mb-6">
      <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
        Currency Name *
      </label>
      <input 
        type="text" 
        value={formData.name}
        oninput={(e) => formData.name = e.target.value}
        placeholder="USDC or MyToken"
        required
        class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
      />
      <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
        Must match an identity you control (e.g., if you own "usdc@", enter "usdc")
      </p>
    </div>

    <!-- Contract Address -->
    <div class="mb-6">
      <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
        Ethereum Contract Address *
      </label>
      <input 
        type="text" 
        value={formData.contractAddress}
        oninput={(e) => formData.contractAddress = e.target.value}
        placeholder="0x1234567890123456789012345678901234567890"
        required
        class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white font-mono"
      />
      <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
        The Ethereum smart contract address for the token you want to map (must start with 0x)
      </p>
    </div>

    <!-- Fixed Parameters Display -->
    <div class="mb-6 p-4 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium rounded-lg">
      <h4 class="text-sm font-medium text-verusidx-stone-dark dark:text-white mb-3">
        Fixed Parameters (automatically set)
      </h4>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4 text-xs">
        <div>
          <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Options:</span>
          <span class="ml-2 font-mono text-verusidx-stone-dark dark:text-white">32</span>
        </div>
        <div>
          <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Proof Protocol:</span>
          <span class="ml-2 font-mono text-verusidx-stone-dark dark:text-white">3 (Ethereum)</span>
        </div>
        <div>
          <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">System ID:</span>
          <span class="ml-2 font-mono text-verusidx-stone-dark dark:text-white">veth</span>
        </div>
        <div>
          <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Parent:</span>
          <span class="ml-2 font-mono text-verusidx-stone-dark dark:text-white">vrsc</span>
        </div>
        <div>
          <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Launch System:</span>
          <span class="ml-2 font-mono text-verusidx-stone-dark dark:text-white">vrsc</span>
        </div>
        <div>
          <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Initial Supply:</span>
          <span class="ml-2 font-mono text-verusidx-stone-dark dark:text-white">0</span>
        </div>
        <div>
          <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Native Currency Type:</span>
          <span class="ml-2 font-mono text-verusidx-stone-dark dark:text-white">9</span>
        </div>
      </div>
    </div>

    <!-- Bridge Information -->
    <div class="mb-6 p-4 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg">
      <p class="text-yellow-700 dark:text-yellow-300 text-sm">
        ℹ️ <strong>Next Steps:</strong> After creation, you'll need to export your currency to Ethereum as an ERC-20 
        to complete the bridge setup. This creates the bidirectional mapping.
      </p>
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
        class="flex-1 px-6 py-3 bg-verusidx-forest-deep hover:bg-verusidx-turquoise-deep text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {isSubmitting ? 'Creating Mapping...' : 'Create Ethereum Mapping'}
      </button>
    </div>
  </div>
</Modal>