<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { connectionStore } from "$lib/stores/connection";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { BlockHeightHeader, DefiOperationModal, OperationCard } from "$lib/components";
  import ExportIdentityWarningModal from "$lib/components/ExportIdentityWarningModal.svelte";
  
  type OperationType = 
    | 'send-transparent'
    | 'send-private' 
    | 'send-crosschain'
    | 'convert-reserve-to-basket'
    | 'convert-basket-to-reserve'
    | 'convert-reserve'
    | 'convert-crosschain'
    | 'preconvert'
    | 'estimate-conversion'
    | 'export-identity'
    | 'export-definition'
    | 'advanced';

  // Subscribe to connection state
  connectionStore.subscribe(state => {
    if (!state.current?.isConnected) {
      goto("/");
    }
  });

  // Operation types with their configurations (4x3 grid layout)
  const operationTypes = [
    // Row 1: Send Operations
    {
      id: "send-transparent",
      title: "Send from Transparent",
      description: "Send currency from R-address",
      icon: "💸",
      color: "bg-verusidx-mountain-blue hover:bg-verusidx-lake-blue"
    },
    {
      id: "send-private", 
      title: "Send from Private",
      description: "Send currency from z-address",
      icon: "🔒",
      color: "bg-verusidx-turquoise-deep hover:bg-verusidx-turquoise-bright"
    },
    {
      id: "send-crosschain", 
      title: "Send Cross-Chain",
      description: "Send currency to another chain",
      icon: "🌉",
      color: "bg-verusidx-mountain-blue hover:bg-verusidx-lake-blue"
    },
    // Row 2: Convert Operations  
    {
      id: "convert-reserve-to-basket",
      title: "Convert Reserve to Basket",
      description: "Convert reserve currency to basket currency", 
      icon: "🔄",
      color: "bg-verusidx-mountain-blue hover:bg-verusidx-lake-blue"
    },
    {
      id: "convert-basket-to-reserve",
      title: "Convert Basket to Reserve",
      description: "Convert basket currency to reserve currency", 
      icon: "🔄",
      color: "bg-verusidx-turquoise-deep hover:bg-verusidx-turquoise-bright"
    },
    {
      id: "convert-reserve",
      title: "Convert Reserve→Reserve", 
      description: "Convert between reserve currencies",
      icon: "🔄",
      color: "bg-verusidx-forest-deep hover:bg-verusidx-turquoise-deep"
    },
    // Row 3: Cross-Chain & Special Operations
    {
      id: "convert-crosschain",
      title: "Convert Cross-Chain", 
      description: "Convert and export to another chain",
      icon: "🌉", 
      color: "bg-verusidx-turquoise-deep hover:bg-verusidx-turquoise-bright"
    },
    {
      id: "preconvert",
      title: "Preconvert to Launch",
      description: "Convert during launch period",
      icon: "🚀", 
      color: "bg-verusidx-turquoise-deep hover:bg-verusidx-turquoise-bright"
    },
    {
      id: "estimate-conversion",
      title: "Estimate Conversion",
      description: "Calculate expected conversion output",
      icon: "🧮",
      color: "bg-verusidx-turquoise-deep hover:bg-verusidx-turquoise-bright"
    },
    // Row 4: Export & Advanced Operations
    {
      id: "export-identity",
      title: "Export Identity",
      description: "Export identity to another chain", 
      icon: "👤",
      color: "bg-verusidx-forest-deep hover:bg-verusidx-turquoise-deep"
    },
    {
      id: "export-definition",
      title: "Export Currency Definition",
      description: "Export currency to another chain",
      icon: "📋",
      color: "bg-verusidx-forest-deep hover:bg-verusidx-turquoise-deep"
    },
    {
      id: "advanced",
      title: "Advanced Mode",
      description: "Full sendcurrency interface with all parameters",
      icon: "🔧",
      color: "bg-verusidx-stone-dark hover:bg-verusidx-mountain-grey"
    }
  ];

  // State for modal management
  let isModalOpen = $state(false);
  let currentOperation = $state<OperationType | null>(null);
  let isExportWarningModalOpen = $state(false);

  // Check if user has seen the export identity warning this session
  const EXPORT_IDENTITY_WARNING_KEY = 'exportIdentityWarningShown';
  let hasSeenExportIdentityWarning = $state(false);

  // Load from sessionStorage on mount
  $effect(() => {
    if (typeof window !== 'undefined') {
      hasSeenExportIdentityWarning = sessionStorage.getItem(EXPORT_IDENTITY_WARNING_KEY) === 'true';
    }
  });

  // State for operations tracking
  let operations = $state<any[]>([]);

  async function loadOperations() {
    try {
      const pendingOps = await invoke("z_get_operation_status", {
        operation_ids: null,
        chain: null
      });
      operations = pendingOps as any[];
    } catch (error) {
      console.error("Failed to load operations:", error);
      operations = [];
    }
  }

  function openModal(operationId: string) {
    // Check if this is export-identity and if user has seen warning
    if (operationId === 'export-identity' && !hasSeenExportIdentityWarning) {
      // Show warning modal first
      currentOperation = operationId as OperationType;
      isExportWarningModalOpen = true;
      console.log("Opening export identity warning modal");
    } else {
      // Directly open operation modal
      currentOperation = operationId as OperationType;
      isModalOpen = true;
      console.log("Opening modal for:", operationId);
    }
  }

  function closeModal() {
    isModalOpen = false;
    currentOperation = null;
    // Reload operations after closing modal to show any new operations
    loadOperations();
  }

  function handleExportWarningProceed() {
    // Mark as seen for this session
    if (typeof window !== 'undefined') {
      sessionStorage.setItem(EXPORT_IDENTITY_WARNING_KEY, 'true');
      hasSeenExportIdentityWarning = true;
    }
    // Close warning modal and open operation modal
    isExportWarningModalOpen = false;
    isModalOpen = true;
  }

  function handleExportWarningCancel() {
    isExportWarningModalOpen = false;
    currentOperation = null;
  }

  // Load operations when component mounts
  $effect(() => {
    loadOperations();
  });

  // Check for URL parameters to auto-open modal
  $effect(() => {
    const operationParam = $page.url.searchParams.get('operation');
    if (operationParam && !isModalOpen) {
      // Validate that the operation is valid
      const validOperations: OperationType[] = [
        'send-transparent', 'send-private', 'send-crosschain',
        'convert-reserve-to-basket', 'convert-basket-to-reserve', 'convert-reserve',
        'convert-crosschain', 'preconvert', 'estimate-conversion',
        'export-identity', 'export-definition', 'advanced'
      ];
      
      if (validOperations.includes(operationParam as OperationType)) {
        console.log("Auto-opening modal for operation:", operationParam);
        openModal(operationParam);
        
        // Clear the URL parameter after opening modal to prevent re-opening
        const url = new URL($page.url);
        url.searchParams.delete('operation');
        goto(url.pathname, { replaceState: true });
      }
    }
  });

</script>

<svelte:head>
  <title>DeFi & Cross-Chain Operations - VerusIDX</title>
  <meta name="description" content="Currency operations, conversions, and cross-chain transfers" />
</svelte:head>


<div class="bg-verusidx-sky-soft dark:bg-verusidx-lake-deep min-h-screen max-h-screen overflow-y-auto">
  <!-- Block Height Header -->
  <BlockHeightHeader />

  <div class="max-w-7xl mx-auto p-8 pb-16 space-y-8">
    <!-- Page Header -->
    <div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl p-6 mb-8">
      <div class="flex justify-between items-center">
        <div>
          <h1 class="text-3xl font-bold text-verusidx-stone-dark dark:text-white">DeFi & Cross-Chain Operations</h1>
          <p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
            Send, convert, and export currencies across chains using sendcurrency
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

    <!-- Operation Buttons Grid -->
    <div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl p-6">
      <h2 class="text-xl font-bold text-verusidx-stone-dark dark:text-white mb-6">Choose Operation</h2>
      
      <!-- Operations grid (4x3) -->
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {#each operationTypes as operation}
          <button
            onclick={() => openModal(operation.id)}
            class="group p-6 {operation.color} text-white rounded-lg shadow-lg hover:shadow-xl transition-all duration-200 transform hover:scale-105"
          >
            <div class="text-3xl mb-3 group-hover:scale-110 transition-transform">
              {operation.icon}
            </div>
            <h3 class="text-lg font-semibold mb-2">{operation.title}</h3>
            <p class="text-sm opacity-90">{operation.description}</p>
          </button>
        {/each}
      </div>
    </div>

    <!-- Operations Status/History -->
    <OperationCard
      {operations}
      cardClass="bg-white dark:bg-verusidx-stone-dark text-verusidx-stone-dark dark:text-white"
      onRefresh={loadOperations}
    />

  </div>
</div>

<!-- Export Identity Warning Modal -->
<ExportIdentityWarningModal
  isOpen={isExportWarningModalOpen}
  onProceed={handleExportWarningProceed}
  onCancel={handleExportWarningCancel}
/>

<!-- DeFi Operation Modal -->
<DefiOperationModal
  isOpen={isModalOpen}
  operationType={currentOperation}
  onClose={closeModal}
/>