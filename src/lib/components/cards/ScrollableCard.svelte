<script lang="ts">
  interface Props {
    title: string;
    maxHeight?: string;
    cardClass?: string;
  }

  let { title, maxHeight = 'h-64', cardClass = '' }: Props = $props();
</script>

<div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl text-white {cardClass}">
  <div class="p-6 border-b border-gray-200 dark:border-verusidx-stone-medium">
    <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">{title}</h3>
  </div>
  
  <div class="p-6">
    <div class="space-y-3 {maxHeight} overflow-y-auto scrollbar-thin scrollbar-thumb-verusidx-stone-medium scrollbar-track-transparent hover:scrollbar-thumb-verusidx-stone-light">
      <slot />
    </div>
  </div>
</div>

<style>
  /* Custom scrollbar styles for better integration */
  .scrollbar-thin {
    scrollbar-width: thin;
  }
  
  .scrollbar-thin::-webkit-scrollbar {
    width: 6px;
  }
  
  .scrollbar-thin::-webkit-scrollbar-track {
    background: transparent;
  }
  
  .scrollbar-thin::-webkit-scrollbar-thumb {
    background-color: rgba(107, 114, 128, 0.3);
    border-radius: 3px;
  }
  
  .scrollbar-thin::-webkit-scrollbar-thumb:hover {
    background-color: rgba(107, 114, 128, 0.5);
  }
</style>