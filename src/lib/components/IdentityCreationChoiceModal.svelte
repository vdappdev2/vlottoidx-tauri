<script lang="ts">
  import { Modal } from './cards';

  interface Props {
    isOpen: boolean;
    onClose: () => void;
    onChooseNew: () => void;
    onChooseExisting: () => void;
  }

  let { isOpen = false, onClose, onChooseNew, onChooseExisting }: Props = $props();
</script>

<Modal {isOpen} onclose={onClose} title="Create Identity" size="md">
  <div class="p-6">
    <p class="text-verusidx-stone-dark dark:text-verusidx-mountain-mist mb-6">
      Choose how you want to proceed with identity creation:
    </p>

    <div class="space-y-4">
      <!-- Option 1: Start fresh -->
      <button
        onclick={onChooseNew}
        class="w-full p-6 border-2 border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark hover:border-verusidx-turquoise-deep dark:hover:border-verusidx-turquoise-bright hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-all text-left"
      >
        <div class="flex items-start space-x-4">
          <span class="text-3xl">📝</span>
          <div>
            <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-1">
              I don't have a name commitment yet
            </h3>
            <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
              Start the full identity creation process from name reservation through registration
            </p>
          </div>
        </div>
      </button>

      <!-- Option 2: Use existing commitment -->
      <button
        onclick={onChooseExisting}
        class="w-full p-6 border-2 border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark hover:border-verusidx-turquoise-deep dark:hover:border-verusidx-turquoise-bright hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-all text-left"
      >
        <div class="flex items-start space-x-4">
          <span class="text-3xl">📋</span>
          <div>
            <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-1">
              I have a name commitment already
            </h3>
            <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
              Resume registration with an existing name commitment JSON
            </p>
          </div>
        </div>
      </button>
    </div>

    <!-- Cancel button -->
    <div class="mt-6 pt-6 border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
      <button
        onclick={onClose}
        class="w-full px-6 py-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium text-verusidx-stone-dark dark:text-white rounded-lg hover:bg-verusidx-mountain-mist dark:hover:bg-verusidx-stone-medium transition-colors"
      >
        Cancel
      </button>
    </div>
  </div>
</Modal>
