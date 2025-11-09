<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { connectionStore, getChainParam } from "$lib/stores/connection";
  import { Modal } from '../cards';
  import PreallocationManager from './PreallocationManager.svelte';
  import { showSuccess, showError } from '$lib/services/notifications';

  interface Props {
    isOpen: boolean;
    onClose: () => void;
    onSuccess?: () => void;
  }

  let { isOpen = false, onClose, onSuccess }: Props = $props();

  // Form state - simplified for ID Control Token
  let formData = $state({
    name: '',
    preallocations: [{ address: '', amount: 0.00000001 }] // Must be exactly one satoshi
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
      preallocations: [{ address: '', amount: 0.00000001 }]
    };
    error = null;
    isSubmitting = false;
  }

  function handlePreallocationsUpdate(preallocations: {address: string, amount: number}[]) {
    // For ID Control Tokens, ensure we always have exactly one entry with 0.00000001
    if (preallocations.length === 0) {
      formData.preallocations = [{ address: '', amount: 0.00000001 }];
    } else {
      // Ensure amount is always 0.00000001 for ID Control Tokens
      formData.preallocations = preallocations.map(p => ({
        address: p.address,
        amount: 0.00000001
      }));
    }
  }

  // Validation
  function validateForm(): boolean {
    if (!formData.name.trim()) {
      error = 'Currency name is required';
      return false;
    }

    if (formData.preallocations.length !== 1) {
      error = 'ID Control Tokens must have exactly one recipient';
      return false;
    }

    if (!formData.preallocations[0].address.trim()) {
      error = 'Recipient address is required';
      return false;
    }

    if (formData.preallocations[0].amount !== 0.00000001) {
      error = 'ID Control Tokens must allocate exactly 0.00000001 (1 satoshi)';
      return false;
    }

    return true;
  }

  async function handleSubmit() {
    if (!validateForm()) return;

    isSubmitting = true;
    error = null;

    try {
      // Build currency definition for ID Control Token
      const currencyDefinition: any = {
        name: formData.name,
        options: 2080, // Fixed for ID Control Tokens
        proofprotocol: 1, // Decentralized
        maxpreconversion: [0], // Fixed: no preconversion allowed
        preallocations: [{
          [formData.preallocations[0].address]: 0.00000001
        }]
      };

      // Debug logging
      console.log('ID Control Token - Building currency definition:');
      console.log('formData.name:', formData.name);
      console.log('formData.preallocations:', formData.preallocations);
      console.log('Final currencyDefinition:', JSON.stringify(currencyDefinition, null, 2));

      // Step 1: Define currency
      const chainParam = getChainParam(connectionState?.selectedChain);
      console.log('IdControlTokenModal: Defining currency on chain:', connectionState?.selectedChain, 'param:', chainParam);
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

      console.log('ID Control Token created successfully:', sendResult);

      showSuccess('ID Control Token created successfully', { txid: sendResult as string });

      if (onSuccess) {
        onSuccess();
      }
      onClose();

    } catch (err) {
      console.error('ID Control Token creation failed:', err);
      error = typeof err === 'string' ? err : 'Currency creation failed. Please try again.';
      showError(error);
    } finally {
      isSubmitting = false;
    }
  }
</script>

<Modal {isOpen} onclose={onClose} title="Create ID Control Token" size="lg">
  <div class="p-6">
    <!-- ID Control Token Info -->
    <div class="mb-6 p-4 bg-purple-50 dark:bg-purple-900/20 border border-purple-200 dark:border-purple-800 rounded-lg">
      <p class="text-purple-700 dark:text-purple-300 text-sm mb-2">
        👑 <strong>ID Control Token:</strong> A special single-satoshi token that grants primary authority over the currency's ID@.
      </p>
      <ul class="text-purple-700 dark:text-purple-300 text-xs space-y-1 ml-4">
        <li>• Exactly 0.00000001 tokens (1 satoshi) minted to one recipient</li>
        <li>• Holder gains control over the identity's update/revocation/recovery</li>
        <li>• Token can be transferred to delegate control</li>
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
        placeholder="MyDAOControl"
        required
        class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
      />
      <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
        Must match an identity you control (e.g., if you own "mydao@", enter "mydao")
      </p>
    </div>

    <!-- Recipient Selection -->
    <div class="mb-6">
      <PreallocationManager
        preallocations={formData.preallocations}
        onUpdate={handlePreallocationsUpdate}
        isIdControlToken={true}
      />
    </div>

    <!-- Fixed Parameters Display -->
    <div class="mb-6 p-4 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium rounded-lg">
      <h4 class="text-sm font-medium text-verusidx-stone-dark dark:text-white mb-3">
        Fixed Parameters (automatically set)
      </h4>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4 text-xs">
        <div>
          <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Options:</span>
          <span class="ml-2 font-mono text-verusidx-stone-dark dark:text-white">2080</span>
        </div>
        <div>
          <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Proof Protocol:</span>
          <span class="ml-2 font-mono text-verusidx-stone-dark dark:text-white">1 (default)</span>
        </div>
        <div>
          <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Max Preconversion:</span>
          <span class="ml-2 font-mono text-verusidx-stone-dark dark:text-white">[0]</span>
        </div>
        <div>
          <span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Total Supply:</span>
          <span class="ml-2 font-mono text-verusidx-stone-dark dark:text-white">0.00000001</span>
        </div>
      </div>
    </div>

    <!-- Authority Transfer Warning -->
    <div class="mb-6 p-4 bg-orange-50 dark:bg-orange-900/20 border border-orange-200 dark:border-orange-800 rounded-lg">
      <p class="text-orange-700 dark:text-orange-300 text-sm">
        ⚠️ <strong>Important:</strong> The recipient will gain primary authority over your currency's namespace identity. 
        Choose the recipient address carefully, as this grants significant control.
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
        class="flex-1 px-6 py-3 bg-purple-900 hover:bg-purple-800 text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {isSubmitting ? 'Creating Control Token...' : 'Create ID Control Token'}
      </button>
    </div>
  </div>
</Modal>