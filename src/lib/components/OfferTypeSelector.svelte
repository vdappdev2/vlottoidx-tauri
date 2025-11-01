<script lang="ts">
  // Props
  interface Props {
    selectedType: 'id-for-id' | 'id-for-currency' | 'currency-for-id' | 'currency-for-currency' | null;
    onTypeSelect: (type: 'id-for-id' | 'id-for-currency' | 'currency-for-id' | 'currency-for-currency') => void;
    disabled?: boolean;
    mode?: 'make' | 'take';
  }

  let { selectedType = null, onTypeSelect, disabled = false, mode = 'make' }: Props = $props();

  // Offer type definitions
  const offerTypes = [
    {
      id: 'id-for-id' as const,
      title: 'Identity → Identity',
      description: `${mode === 'make' ? 'Trade' : 'Accept'} an identity for another identity`,
      icon: '👤↔️👤',
      examples: 'Example: myname@ → yourname@',
      color: 'bg-verusidx-mountain-blue'
    },
    {
      id: 'id-for-currency' as const,
      title: 'Identity → Currency',
      description: `${mode === 'make' ? 'Trade' : 'Accept'} an identity for currency`,
      icon: '👤💰',
      examples: 'Example: myname@ → 100 VRSC',
      color: 'bg-verusidx-turquoise-deep'
    },
    {
      id: 'currency-for-id' as const,
      title: 'Currency → Identity',
      description: `${mode === 'make' ? 'Trade' : 'Accept'} currency for an identity`,
      icon: '💰👤',
      examples: 'Example: 100 VRSC → yourname@',
      color: 'bg-verusidx-forest-deep'
    },
    {
      id: 'currency-for-currency' as const,
      title: 'Currency → Currency',
      description: `${mode === 'make' ? 'Trade' : 'Accept'} one currency for another`,
      icon: '💰↔️💰',
      examples: 'Example: 100 VRSC → 50 DAI.vETH',
      color: 'bg-verusidx-lake-blue'
    }
  ];

  function handleTypeSelect(typeId: typeof offerTypes[0]['id']) {
    if (!disabled) {
      onTypeSelect(typeId);
    }
  }
</script>

<div class="space-y-4">
  <div class="text-center mb-6">
    <h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-2">
      Select {mode === 'make' ? 'Offer' : 'Acceptance'} Type
    </h3>
    <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
      Choose what you want to {mode === 'make' ? 'offer and what you want in return' : 'accept and what you will provide'}
    </p>
  </div>

  <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
    {#each offerTypes as offerType}
      <button
        onclick={() => handleTypeSelect(offerType.id)}
        disabled={disabled}
        class="relative p-6 rounded-lg border-2 transition-all duration-200 text-left group {selectedType === offerType.id ? 
          'border-verusidx-turquoise-deep bg-verusidx-turquoise-light/20 dark:bg-verusidx-turquoise-deep/20' : 
          'border-verusidx-mountain-mist dark:border-verusidx-stone-medium hover:border-verusidx-turquoise-light dark:hover:border-verusidx-turquoise-deep hover:bg-verusidx-snow-ice dark:hover:bg-verusidx-stone-medium'
        } disabled:opacity-50 disabled:cursor-not-allowed"
      >
        <!-- Selection Indicator -->
        {#if selectedType === offerType.id}
          <div class="absolute top-3 right-3 w-6 h-6 bg-verusidx-turquoise-deep rounded-full flex items-center justify-center">
            <svg class="w-4 h-4 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
            </svg>
          </div>
        {/if}

        <!-- Icon -->
        <div class="text-3xl mb-3 group-hover:scale-110 transition-transform">
          {offerType.icon}
        </div>

        <!-- Title -->
        <h4 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-2 group-hover:text-verusidx-turquoise-deep dark:group-hover:text-verusidx-turquoise-bright transition-colors">
          {offerType.title}
        </h4>

        <!-- Description -->
        <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-3">
          {offerType.description}
        </p>

        <!-- Examples -->
        <div class="text-xs text-verusidx-turquoise-deep dark:text-verusidx-turquoise-bright bg-verusidx-turquoise-light/10 dark:bg-verusidx-turquoise-deep/10 rounded px-2 py-1">
          {offerType.examples}
        </div>

        <!-- Hover Effect Overlay -->
        <div class="absolute inset-0 rounded-lg {offerType.color} opacity-0 group-hover:opacity-5 transition-opacity"></div>
      </button>
    {/each}
  </div>

  <!-- Selected Type Summary -->
  {#if selectedType}
    {@const selected = offerTypes.find(t => t.id === selectedType)}
    <div class="mt-6 p-4 bg-verusidx-lake-deep/10 dark:bg-verusidx-turquoise-deep/20 border border-verusidx-turquoise-light dark:border-verusidx-turquoise-deep rounded-lg">
      <div class="flex items-center space-x-3">
        <span class="text-2xl">{selected?.icon}</span>
        <div>
          <h4 class="font-medium text-verusidx-stone-dark dark:text-white">
            Selected: {selected?.title}
          </h4>
          <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
            {selected?.description}
          </p>
        </div>
      </div>
    </div>
  {/if}

  <!-- Help Text -->
  <div class="mt-6 p-4 bg-verusidx-snow-ice dark:bg-verusidx-stone-medium rounded-lg">
    <h4 class="font-medium text-verusidx-stone-dark dark:text-white mb-2">💡 Need Help?</h4>
    <ul class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist space-y-1">
      <li>• <strong>Identity trades</strong> involve transferring ownership of VerusID names</li>
      <li>• <strong>Currency trades</strong> involve exchanging amounts of different tokens</li>
      <li>• <strong>Mixed trades</strong> combine identity and currency exchanges</li>
      <li>• All trades are atomic - either both sides succeed or the trade fails safely</li>
    </ul>
  </div>
</div>