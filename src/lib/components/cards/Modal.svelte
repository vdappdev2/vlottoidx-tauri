<script lang="ts">
  interface Props {
    isOpen: boolean;
    title: string;
    size?: 'sm' | 'md' | 'lg' | 'xl' | 'full';
    zIndex?: string;
    onclose?: () => void;
    preventBackdropClose?: boolean;
  }

  let { isOpen = false, title, size = 'md', zIndex = 'z-50', onclose, preventBackdropClose = false }: Props = $props();

  function closeModal() {
    if (onclose) {
      onclose();
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && isOpen && !preventBackdropClose) {
      closeModal();
    }
  }

  function handleBackdropClick() {
    if (!preventBackdropClose) {
      closeModal();
    }
  }

  const sizeClasses = $derived({
    sm: 'max-w-md',
    md: 'max-w-lg',
    lg: 'max-w-2xl',
    xl: 'max-w-4xl',
    full: 'max-w-[95vw] max-h-[95vh]'
  });
</script>

<svelte:window on:keydown={handleKeydown} />

{#if isOpen}
  <div 
    class="fixed inset-0 {zIndex} overflow-y-auto"
    aria-labelledby="modal-title" 
    role="dialog" 
    aria-modal="true"
  >
    <!-- Backdrop -->
    <div 
      class="flex items-center justify-center min-h-screen pt-4 px-4 pb-20 text-center sm:block sm:p-0"
      onclick={handleBackdropClick}
      onkeydown={(e) => e.key === 'Escape' && closeModal()}
      role="presentation"
    >
      <div 
        class="fixed inset-0 bg-transparent"
        aria-hidden="true"
      ></div>

      <!-- Center modal -->
      <span class="hidden sm:inline-block sm:align-middle sm:h-screen" aria-hidden="true">&#8203;</span>

      <!-- Modal panel -->
      <div 
        class="inline-block align-middle bg-white dark:bg-verusidx-stone-dark rounded-lg px-4 pt-5 pb-4 text-left overflow-hidden shadow-xl transform transition-all sm:my-8 {sizeClasses[size]} sm:w-full sm:p-6"
        role="dialog"
        tabindex="0"
        onclick={(e) => e.stopPropagation()}
        onkeydown={(e) => e.key === 'Escape' && closeModal()}
      >
        <!-- Header -->
        <div class="flex items-center justify-between pb-4 border-b border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
          <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white" id="modal-title">
            {title}
          </h3>
          <button
            type="button"
            class="rounded-md text-verusidx-mountain-mist hover:text-verusidx-mountain-grey dark:hover:text-verusidx-mountain-mist focus:outline-none focus:ring-2 focus:ring-verusidx-mountain-blue"
            onclick={closeModal}
          >
            <span class="sr-only">Close</span>
            <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <!-- Content -->
        <div class="mt-4">
          <slot />
        </div>
      </div>
    </div>
  </div>
{/if}