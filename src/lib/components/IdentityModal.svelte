<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { connectionStore, getChainParam } from "$lib/stores/connection";
  import Modal from "./cards/Modal.svelte";
  import IdentityDetailsView from "./IdentityDetailsView.svelte";

  interface Props {
    isOpen: boolean;
    identityAddress: string;
    onclose: () => void;
  }

  let { isOpen = false, identityAddress, onclose }: Props = $props();

  let identityData = $state<any>(null);
  let isLoading = $state(false);
  let error = $state<string | null>(null);
  let connectionState = $state<any>(null);

  connectionStore.subscribe(state => {
    connectionState = state;
  });

  // Effect to fetch identity data when modal opens
  $effect(() => {
    if (isOpen && identityAddress) {
      fetchIdentityData();
    } else if (!isOpen) {
      // Reset state when modal closes
      identityData = null;
      error = null;
      isLoading = false;
    }
  });

  async function fetchIdentityData() {
    if (!identityAddress) return;

    isLoading = true;
    error = null;
    identityData = null;

    try {
      const chainParam = getChainParam(connectionState?.selectedChain);
      console.log("IdentityModal: Fetching identity data for:", identityAddress, "on chain:", chainParam);
      
      const result = await invoke("get_identity", { 
        name: identityAddress,
        chain: chainParam
      });

      console.log("IdentityModal: get_identity result:", result);
      identityData = result;
    } catch (err) {
      console.error("IdentityModal: Failed to fetch identity data:", err);
      error = typeof err === 'string' ? err : "Failed to load identity data. Please try again.";
    } finally {
      isLoading = false;
    }
  }

  function handleClose() {
    onclose();
  }
</script>

<Modal 
  {isOpen} 
  title="Identity Details - {identityAddress}"
  size="xl"
  zIndex="z-[100]"
  onclose={handleClose}
>
  {#if isLoading}
    <div class="flex items-center justify-center py-8">
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-verusidx-mountain-blue"></div>
      <span class="ml-3 text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Loading identity data...</span>
    </div>
  {:else if error}
    <div class="text-center py-8">
      <div class="text-red-600 dark:text-red-400 mb-2">
        <svg class="h-12 w-12 mx-auto mb-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.732-.833-2.5 0L4.268 16.5c-.77.833.192 2.5 1.732 2.5z" />
        </svg>
      </div>
      <p class="text-red-600 dark:text-red-400 font-medium">{error}</p>
      <p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-2">
        Please check the identity address and try again
      </p>
    </div>
  {:else if identityData}
    <IdentityDetailsView {identityData} />
  {:else}
    <div class="text-center py-8">
      <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">No identity data available</p>
    </div>
  {/if}
</Modal>