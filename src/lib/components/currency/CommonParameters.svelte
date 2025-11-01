<script lang="ts">
  interface Props {
    formData: {
      idRegistrationFees?: number | undefined;
      idReferralLevels?: number | undefined;
      startBlock?: string;
      endBlock?: string; // Only for centralized currencies
      minPreconversion?: number[];
      maxPreconversion?: number[];
    };
    onUpdate: (field: string, value: any) => void;
    showEndBlock?: boolean; // For centralized currencies
    currencies?: string[]; // For preconversion limits
  }

  let { formData, onUpdate, showEndBlock = false, currencies = [] }: Props = $props();

  function updateMinPreconversion(index: number, value: number) {
    const currentArray = formData.minPreconversion || [];
    const newArray = [...currentArray];
    newArray[index] = value || 0;
    onUpdate('minPreconversion', newArray);
  }

  function updateMaxPreconversion(index: number, value: number) {
    const currentArray = formData.maxPreconversion || [];
    const newArray = [...currentArray];
    newArray[index] = value || 0;
    onUpdate('maxPreconversion', newArray);
  }

  // Ensure preconversion arrays match currencies length
  $effect(() => {
    if (currencies.length > 0) {
      // Initialize arrays if they don't exist
      if (!formData.minPreconversion) {
        const newMinArray = new Array(currencies.length).fill(0);
        onUpdate('minPreconversion', newMinArray);
      }
      if (!formData.maxPreconversion) {
        const newMaxArray = new Array(currencies.length).fill(0);
        onUpdate('maxPreconversion', newMaxArray);
      }

      // Resize arrays to match currencies
      if (formData.minPreconversion && formData.minPreconversion.length !== currencies.length) {
        const newMinArray = new Array(currencies.length).fill(0);
        // Copy existing values up to the min of old and new length
        const copyLength = Math.min(formData.minPreconversion.length, currencies.length);
        for (let i = 0; i < copyLength; i++) {
          newMinArray[i] = formData.minPreconversion[i];
        }
        onUpdate('minPreconversion', newMinArray);
      }
      if (formData.maxPreconversion && formData.maxPreconversion.length !== currencies.length) {
        const newMaxArray = new Array(currencies.length).fill(0);
        // Copy existing values up to the min of old and new length
        const copyLength = Math.min(formData.maxPreconversion.length, currencies.length);
        for (let i = 0; i < copyLength; i++) {
          newMaxArray[i] = formData.maxPreconversion[i];
        }
        onUpdate('maxPreconversion', newMaxArray);
      }
    }
  });
</script>

<div class="space-y-6">
  <!-- Sub-Identity Settings -->
  <div class="border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium pt-4">
    <h4 class="text-sm font-medium text-verusidx-stone-dark dark:text-white mb-4">
      Sub-Identity Settings
    </h4>
    
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div>
        <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
          Registration Fee
        </label>
        <input 
          type="number" 
          value={formData.idRegistrationFees ?? ''}
          oninput={(e) => {
            const value = parseFloat(e.target.value);
            onUpdate('idRegistrationFees', isNaN(value) ? undefined : value);
          }}
          min="0.000000001"
          max="999999999"
          step="0.00000001"
          placeholder="Optional - leave blank to use default"
          class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
        />
        <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
          Cost in your currency to register sub-identities
        </p>
      </div>

      <div>
        <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
          Referral Levels (0-5)
        </label>
        <input 
          type="number" 
          value={formData.idReferralLevels ?? ''}
          oninput={(e) => {
            const value = parseInt(e.target.value);
            onUpdate('idReferralLevels', isNaN(value) ? undefined : value);
          }}
          min="0"
          max="5"
          placeholder="Optional - leave blank to use default"
          class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
        />
        <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
          Number of referral levels for sub-identity registration rewards
        </p>
      </div>
    </div>
  </div>

  <!-- Launch Timing -->
  <div class="border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium pt-4">
    <h4 class="text-sm font-medium text-verusidx-stone-dark dark:text-white mb-4">
      Launch Timing
    </h4>
    
    <div class="grid grid-cols-1 {showEndBlock ? 'md:grid-cols-2' : ''} gap-4">
      <div>
        <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
          Start Block (Optional)
        </label>
        <input 
          type="number" 
          value={formData.startBlock}
          oninput={(e) => onUpdate('startBlock', e.target.value)}
          min="0"
          placeholder="Auto (current + 20 blocks)"
          class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
        />
        <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
          Block height to start the currency (default: current + 20)
        </p>
      </div>

      {#if showEndBlock}
        <div>
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
            End Block (Optional)
          </label>
          <input 
            type="number" 
            value={formData.endBlock}
            oninput={(e) => onUpdate('endBlock', e.target.value)}
            min="0"
            placeholder="No end block"
            class="w-full p-3 border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
          />
          <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
            After this block, no more supply can be minted (centralized currencies only)
          </p>
        </div>
      {/if}
    </div>
  </div>

  <!-- Preconversion Limits -->
  {#if currencies.length > 0}
    <div class="border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium pt-4">
      <h4 class="text-sm font-medium text-verusidx-stone-dark dark:text-white mb-4">
        Preconversion Limits (Optional)
      </h4>
      
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
            Minimum Preconversion
          </label>
          <div class="grid gap-2" style="grid-template-columns: repeat({currencies.length}, 1fr);">
            {#each currencies as currency, index}
              <div>
                <label class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-1 block">
                  {currency || `Currency ${index + 1}`}
                </label>
                <input 
                  type="number" 
                  value={formData.minPreconversion?.[index] || 0}
                  oninput={(e) => updateMinPreconversion(index, parseFloat(e.target.value) || 0)}
                  min="0"
                  step="0.00000001"
                  placeholder="0"
                  class="w-full p-2 text-sm border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
                />
              </div>
            {/each}
          </div>
          <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
            Minimum amounts needed for launch (currency won't launch if not met)
          </p>
        </div>

        <div>
          <label class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2">
            Maximum Preconversion
          </label>
          <div class="grid gap-2" style="grid-template-columns: repeat({currencies.length}, 1fr);">
            {#each currencies as currency, index}
              <div>
                <label class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-1 block">
                  {currency || `Currency ${index + 1}`}
                </label>
                <input 
                  type="number" 
                  value={formData.maxPreconversion?.[index] || 0}
                  oninput={(e) => updateMaxPreconversion(index, parseFloat(e.target.value) || 0)}
                  min="0"
                  step="0.00000001"
                  placeholder="0 (no limit)"
                  class="w-full p-2 text-sm border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
                />
              </div>
            {/each}
          </div>
          <p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
            Maximum amounts allowed (excess will be refunded after launch)
          </p>
        </div>
      </div>
    </div>
  {/if}
</div>