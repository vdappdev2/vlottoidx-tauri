<script lang="ts">
  import { connectionStore } from "$lib/stores/connection";
  import { goto } from "$app/navigation";
  import { BlockHeightHeader } from "$lib/components";
  import IdentitySearchCard from "$lib/components/IdentitySearchCard.svelte";
  import IdentityListCard from "$lib/components/IdentityListCard.svelte";
  import IdentityCreationModal from "$lib/components/IdentityCreationModal.svelte";
  import RevokeIdentityModal from "$lib/components/RevokeIdentityModal.svelte";
  import RecoverIdentityModal from "$lib/components/RecoverIdentityModal.svelte";
  import UpdateIdentityModal from "$lib/components/UpdateIdentityModal.svelte";
  import TimelockIdentityModal from "$lib/components/TimelockIdentityModal.svelte";
  import TimelockWarningModal from "$lib/components/TimelockWarningModal.svelte";

  let isCreationModalOpen = $state(false);
  let isRevokeModalOpen = $state(false);
  let isRecoverModalOpen = $state(false);
  let isUpdateModalOpen = $state(false);
  let isTimelockWarningModalOpen = $state(false);
  let isTimelockModalOpen = $state(false);

  // Check if user has seen the timelock warning this session
  const TIMELOCK_WARNING_KEY = 'timelockWarningShown';
  let hasSeenTimelockWarning = $state(false);

  // Load from sessionStorage on mount
  $effect(() => {
    if (typeof window !== 'undefined') {
      hasSeenTimelockWarning = sessionStorage.getItem(TIMELOCK_WARNING_KEY) === 'true';
    }
  });

  connectionStore.subscribe(state => {
    if (!state.current?.isConnected) {
      goto("/");
    }
  });

  function disconnect() {
    connectionStore.update(state => ({ ...state, current: null }));
    goto("/");
  }

  function openCreationModal() {
    isCreationModalOpen = true;
  }

  function closeCreationModal() {
    isCreationModalOpen = false;
  }

  function handleCreationSuccess() {
    // Refresh the identity list after successful creation
    // The IdentityListCard component should handle refreshing itself
    closeCreationModal();
  }

  function openRevokeModal() {
    isRevokeModalOpen = true;
  }

  function closeRevokeModal() {
    isRevokeModalOpen = false;
  }

  function openRecoverModal() {
    isRecoverModalOpen = true;
  }

  function closeRecoverModal() {
    isRecoverModalOpen = false;
  }

  function openUpdateModal() {
    isUpdateModalOpen = true;
  }

  function closeUpdateModal() {
    isUpdateModalOpen = false;
  }

  function openTimelockModal() {
    // Check if user has seen warning this session
    if (hasSeenTimelockWarning) {
      // Directly open timelock modal
      isTimelockModalOpen = true;
    } else {
      // Show warning modal first
      isTimelockWarningModalOpen = true;
    }
  }

  function closeTimelockModal() {
    isTimelockModalOpen = false;
  }

  function handleTimelockWarningProceed() {
    // Mark as seen for this session
    if (typeof window !== 'undefined') {
      sessionStorage.setItem(TIMELOCK_WARNING_KEY, 'true');
      hasSeenTimelockWarning = true;
    }
    // Close warning modal and open timelock modal
    isTimelockWarningModalOpen = false;
    isTimelockModalOpen = true;
  }

  function handleTimelockWarningCancel() {
    isTimelockWarningModalOpen = false;
  }
</script>

<div class="bg-verusidx-sky-soft dark:bg-verusidx-lake-deep min-h-screen max-h-screen overflow-y-auto">
  <!-- Block Height Header -->
  <BlockHeightHeader />

  <div class="max-w-7xl mx-auto p-8">
    <!-- Header -->
    <div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl p-6 mb-8">
      <div class="flex justify-between items-center">
        <div>
          <h1 class="text-3xl font-bold text-verusidx-stone-dark dark:text-white">Identity Management</h1>
          <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
            Search network identities and manage your own
          </p>
        </div>
        
        <a 
          href="/dashboard"
          class="px-4 py-2 bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-stone-medium dark:text-verusidx-mountain-mist rounded-lg hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-dark transition-colors"
        >
          Back to Dashboard
        </a>
      </div>
    </div>

    <!-- Identity Management Section -->
    <div class="mb-8">
      <div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl p-6">
        <div class="flex items-center justify-between mb-4">
          <div>
            <h2 class="text-xl font-semibold text-verusidx-stone-dark dark:text-white">ID Actions</h2>
            <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
              Create & Manage IDs
            </p>
          </div>
          <div class="flex space-x-3">
            <button
              onclick={openCreationModal}
              class="px-4 py-2 bg-verusidx-forest-deep text-white rounded-lg hover:bg-verusidx-turquoise-deep transition-colors"
            >
              Create Identity
            </button>
            <button
              onclick={openRevokeModal}
              class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
            >
              Revoke Identity
            </button>
            <button
              onclick={openRecoverModal}
              class="px-4 py-2 bg-verusidx-turquoise-deep text-white rounded-lg hover:bg-verusidx-turquoise-bright transition-colors"
            >
              Recover Identity
            </button>
            <button
              onclick={openUpdateModal}
              class="px-4 py-2 bg-verusidx-mountain-blue text-white rounded-lg hover:bg-verusidx-lake-blue transition-colors"
            >
              Update Identity
            </button>
            <button
              onclick={openTimelockModal}
              class="px-4 py-2 bg-purple-900 text-white rounded-lg hover:bg-purple-800 transition-colors"
            >
              Timelock Identity
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Main Content Grid -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
      <!-- Identity Search Section -->
      <div>
        <IdentitySearchCard />
      </div>

      <!-- Your Identities Section -->
      <div>
        <IdentityListCard />
      </div>
    </div>

  </div>
</div>

<!-- Identity Creation Modal -->
<IdentityCreationModal 
  isOpen={isCreationModalOpen}
  onClose={closeCreationModal}
  onSuccess={handleCreationSuccess}
/>

<!-- Revoke Identity Modal -->
<RevokeIdentityModal
  isOpen={isRevokeModalOpen}
  onClose={closeRevokeModal}
/>

<!-- Recover Identity Modal -->
<RecoverIdentityModal
  isOpen={isRecoverModalOpen}
  onClose={closeRecoverModal}
/>

<!-- Update Identity Modal -->
<UpdateIdentityModal
  isOpen={isUpdateModalOpen}
  onClose={closeUpdateModal}
/>

<!-- Timelock Warning Modal -->
<TimelockWarningModal
  isOpen={isTimelockWarningModalOpen}
  onProceed={handleTimelockWarningProceed}
  onCancel={handleTimelockWarningCancel}
/>

<!-- Timelock Identity Modal -->
<TimelockIdentityModal
  isOpen={isTimelockModalOpen}
  onClose={closeTimelockModal}
/>