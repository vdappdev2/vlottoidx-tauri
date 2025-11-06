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
  title="🚥 Identity Export - Important Information"
>
  <div class="space-y-4">
    <!-- Educational content -->
    <div class="p-4 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg">
      <h3 class="text-sm font-semibold text-yellow-900 dark:text-yellow-200 mb-3">
        Understanding Control Tokens and Identity Exports
      </h3>

      <ul class="space-y-3 text-sm text-yellow-800 dark:text-yellow-300">
        <li class="flex items-start">
          <span class="mr-2 mt-0.5">🌐</span>
          <span>
            <strong>One control token gets defined:</strong> An ID control token can only be defined on the chain where the identity was originally registered. There will only be one control token that works across all chains and all instances of an exported ID.
          </span>
        </li>

        <li class="flex items-start">
          <span class="mr-2 mt-0.5">⌚</span>
          <span>
            <strong>The order matters:</strong> Once an identity is exported without a control token, you cannot retroactively add token control to it. You can still define a control token even if you've already exported the ID to other chains, but any previously exported IDs won't be controlled by it.
          </span>
        </li>

        <li class="flex items-start">
          <span class="mr-2 mt-0.5">🎫</span>
          <span>
            <strong>A difference in flags:</strong> A flag is set in an ID when it gets a control token definition. Only afterward can the ID be exported with that flag and be controllable by the token.
          </span>
        </li>
      </ul>
    </div>

    <!-- Understanding checkbox -->
    <label class="flex items-start space-x-3 cursor-pointer p-3 bg-gray-50 dark:bg-gray-800/50 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors">
      <input
        type="checkbox"
        bind:checked={understood}
        class="mt-1 w-4 h-4 text-pink-600 border-gray-300 dark:border-gray-600 rounded focus:ring-pink-600"
      />
      <span class="text-sm text-gray-700 dark:text-gray-300 font-medium">
        I understand how control tokens work with identity exports
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
        class="px-4 py-2 bg-pink-600 text-white rounded-lg hover:bg-pink-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      >
        Proceed
      </button>
    </div>
  </div>
</Modal>
