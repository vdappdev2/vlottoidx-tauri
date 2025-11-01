<script lang="ts">
  import { Modal } from './cards';

  // Props
  interface Props {
    isOpen: boolean;
    onProceed: () => void;
    onCancel: () => void;
  }

  let { isOpen = false, onProceed, onCancel }: Props = $props();

  // Checkbox state
  let understood = $state(false);

  // Reset checkbox when modal opens/closes
  $effect(() => {
    if (isOpen) {
      understood = false;
    }
  });
</script>

<Modal
  isOpen={isOpen}
  onclose={onCancel}
  title="⚠️ Identity Timelock - Important Information"
>
  <div class="space-y-4">
    <!-- Educational content -->
    <div class="p-4 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg">
      <h3 class="text-sm font-semibold text-yellow-900 dark:text-yellow-200 mb-3">
        Best Practices for Setting Identity Timelock
      </h3>

      <ul class="space-y-3 text-sm text-yellow-800 dark:text-yellow-300">
        <li class="flex items-start">
          <span class="mr-2 mt-0.5">🔒</span>
          <span>
            <strong>Absolute block height is permanent:</strong> Once set, you cannot change the unlock block height without going through the revoke/recover process.
          </span>
        </li>

        <li class="flex items-start">
          <span class="mr-2 mt-0.5">⏰</span>
          <span>
            <strong>Initiating unlock for delay timelock:</strong> To start the unlock countdown on a delay-based timelock, input <code class="px-1 py-0.5 bg-yellow-100 dark:bg-yellow-800 rounded">0</code> into the "absolute timelock unlock at block height" field.
          </span>
        </li>

        <li class="flex items-start">
          <span class="mr-2 mt-0.5">🔑</span>
          <span>
            <strong>Recovery is possible:</strong> You can revoke and recover a locked identity if you have the proper revocation and recovery authorities configured.
          </span>
        </li>
      </ul>
    </div>

    <!-- Understanding checkbox -->
    <label class="flex items-start space-x-3 cursor-pointer p-3 bg-gray-50 dark:bg-gray-800/50 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors">
      <input
        type="checkbox"
        bind:checked={understood}
        class="mt-1 w-4 h-4 text-purple-900 border-gray-300 dark:border-gray-600 rounded focus:ring-purple-900"
      />
      <span class="text-sm text-gray-700 dark:text-gray-300 font-medium">
        I understand the implications of setting an identity timelock
      </span>
    </label>

    <!-- Action buttons -->
    <div class="flex justify-end space-x-3 pt-2">
      <button
        type="button"
        onclick={onCancel}
        class="px-4 py-2 text-verusidx-mountain-grey dark:text-verusidx-mountain-mist hover:text-verusidx-stone-dark dark:hover:text-white transition-colors"
      >
        Cancel
      </button>
      <button
        type="button"
        onclick={onProceed}
        disabled={!understood}
        class="px-4 py-2 bg-purple-900 text-white rounded-lg hover:bg-purple-800 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      >
        Proceed
      </button>
    </div>
  </div>
</Modal>
