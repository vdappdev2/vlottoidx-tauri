<script lang="ts">
  interface Props {
    baseOptions: number; // 32 for Simple Token, 33 for Basket Currency
    selectedModifiers: {
      onlyIdCanCreateSubIds: boolean;
      enableReferrals: boolean;
      referralsRequired: boolean;
    };
    onUpdate: (newOptions: number) => void;
    onModifiersUpdate: (newModifiers: {
      onlyIdCanCreateSubIds: boolean;
      enableReferrals: boolean;
      referralsRequired: boolean;
    }) => void;
  }

  let { baseOptions, selectedModifiers, onUpdate, onModifiersUpdate }: Props = $props();

  // Calculate final options value
  let finalOptions = $derived(() => {
    let options = baseOptions;
    if (selectedModifiers.onlyIdCanCreateSubIds) options += 2;
    if (selectedModifiers.enableReferrals) options += 8;
    if (selectedModifiers.referralsRequired) options += 16;
    return options;
  });

  // Update parent when options change
  $effect(() => {
    onUpdate(finalOptions());
  });

  function handleModifierChange(field: keyof typeof selectedModifiers, value: boolean) {
    const newModifiers = {
      ...selectedModifiers,
      [field]: value
    };
    onModifiersUpdate(newModifiers);
  }
</script>

<div class="space-y-3">
  <div class="border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium pt-4">
    <h4 class="text-sm font-medium text-verusidx-stone-dark dark:text-white mb-3">
      Sub-Identity Control Options
    </h4>
    
    <div class="space-y-2">
      <label class="flex items-center space-x-2">
        <input 
          type="checkbox" 
          checked={selectedModifiers.onlyIdCanCreateSubIds}
          onchange={(e) => handleModifierChange('onlyIdCanCreateSubIds', e.target.checked)}
          class="text-verusidx-forest-deep focus:ring-verusidx-forest-deep"
        />
        <span class="text-sm text-verusidx-stone-dark dark:text-white">
          Only the currency's ID@ can create sub-identities (+2)
        </span>
      </label>
      
      <label class="flex items-center space-x-2">
        <input 
          type="checkbox" 
          checked={selectedModifiers.enableReferrals}
          onchange={(e) => handleModifierChange('enableReferrals', e.target.checked)}
          class="text-verusidx-forest-deep focus:ring-verusidx-forest-deep"
        />
        <span class="text-sm text-verusidx-stone-dark dark:text-white">
          Enable referrals & discounts for sub-identity creation (+8)
        </span>
      </label>
      
      <label class="flex items-center space-x-2">
        <input 
          type="checkbox" 
          checked={selectedModifiers.referralsRequired}
          onchange={(e) => handleModifierChange('referralsRequired', e.target.checked)}
          disabled={!selectedModifiers.enableReferrals}
          class="text-verusidx-forest-deep focus:ring-verusidx-forest-deep disabled:opacity-50"
        />
        <span class="text-sm text-verusidx-stone-dark dark:text-white {!selectedModifiers.enableReferrals ? 'opacity-50' : ''}">
          Referrals are required for sub-identity creation (+16)
        </span>
      </label>
    </div>
  </div>
</div>