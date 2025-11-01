<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { BlockHeightHeader } from '$lib/components';
	import {
		LedgerSummaryCard,
		PhaseTimelineCard,
		DrawingResultsCard,
		FinancialSummaryCard,
		TicketListCard,
		MyTicketsCard
	} from '$lib/components/vlotto';
	import { vlottoCache } from '$lib/stores/vlottoCache';
	import { fetchLedger, startAutoRefresh, manualRefresh } from '$lib/services/vlottoService';
	import { connectionStore, getChainParam } from '$lib/stores/connection';

	let cacheState = $state($vlottoCache);
	vlottoCache.subscribe((state) => {
		cacheState = state;
	});

	let connState = $state($connectionStore);
	connectionStore.subscribe((state) => {
		connState = state;
	});

	let stopAutoRefresh: (() => void) | null = null;
	let showRawData = $state(false);

	onMount(async () => {
		// Initial load
		await loadData();

		// Start auto-refresh polling (every 60s)
		const chain = getChainParam(connState.selectedChain) || undefined;
		stopAutoRefresh = startAutoRefresh(chain, 60000);
	});

	onDestroy(() => {
		// Stop polling when component unmounts
		if (stopAutoRefresh) {
			stopAutoRefresh();
		}
	});

	async function loadData() {
		try {
			await fetchLedger();
		} catch (error) {
			// Error handled by vlottoCache state
		}
	}

	async function handleRefresh() {
		await manualRefresh();
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
					<h1 class="text-3xl font-bold text-verusidx-stone-dark dark:text-white">
						vLotto Viewer
					</h1>
					<p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
						View phases, drawing status, and ticket information
					</p>
				</div>

				<div class="flex gap-2">
					<a
						href="/defi"
						class="px-4 py-2 bg-verusidx-forest-deep text-white rounded-lg hover:bg-verusidx-turquoise-deep transition-colors"
					>
						Get vlotto
					</a>

					<a
						href="/marketplace"
						class="px-4 py-2 bg-purple-900 text-white rounded-lg hover:bg-purple-800 transition-colors"
					>
						Get Tickets
					</a>

					<button
						onclick={handleRefresh}
						class="px-4 py-2 bg-verusidx-mountain-blue text-white rounded-lg hover:bg-verusidx-lake-blue transition-colors disabled:opacity-50"
						disabled={cacheState.isLoading}
					>
						{cacheState.isLoading ? 'Refreshing...' : 'Refresh'}
					</button>

					<a
						href="/dashboard"
						class="px-4 py-2 bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-stone-medium dark:text-verusidx-mountain-mist rounded-lg hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-dark transition-colors"
					>
						Back to Dashboard
					</a>
				</div>
			</div>
		</div>

		{#if cacheState.isLoading && !cacheState.ledger.data}
			<!-- Initial Loading State -->
			<div class="flex items-center justify-center py-12">
				<div class="text-center">
					<div
						class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-verusidx-turquoise-deep mb-4"
					></div>
					<p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
						Loading ledger data...
					</p>
				</div>
			</div>
		{:else if cacheState.error}
			<!-- Error State -->
			<div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl p-6">
				<div
					class="bg-red-100 dark:bg-red-900/20 border border-red-400 dark:border-red-500 rounded-lg p-4"
				>
					<h3 class="text-red-700 dark:text-red-400 font-semibold mb-2">Error Loading Ledger</h3>
					<p class="text-verusidx-stone-dark dark:text-verusidx-mountain-mist">
						{cacheState.error}
					</p>
					<button
						onclick={handleRefresh}
						class="mt-4 px-4 py-2 bg-verusidx-turquoise-deep text-white rounded-lg hover:bg-verusidx-turquoise-bright transition-colors"
					>
						Retry
					</button>
				</div>
			</div>
		{:else if cacheState.ledger.data}
			<!-- Main Content -->
			<div class="space-y-6">
				<!-- Ledger Summary -->
				<LedgerSummaryCard ledgerData={cacheState.ledger.data} />

				<!-- Two-column layout for Phase Timeline and Drawing Results -->
				<div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
					<PhaseTimelineCard ledgerData={cacheState.ledger.data} />
					<DrawingResultsCard ledgerData={cacheState.ledger.data} />
				</div>

				<!-- Ticket List (lazy-loaded) -->
				<TicketListCard ledgerData={cacheState.ledger.data} />

				<!-- My Tickets (lazy-loaded) -->
				<MyTicketsCard
					ledgerData={cacheState.ledger.data}
					vlottoParent={cacheState.ledger.vlottoParent}
				/>

				<!-- Financial Summary -->
				{#if cacheState.ledger.data.utilities}
					<FinancialSummaryCard ledgerData={cacheState.ledger.data} />
				{/if}

				<!-- Raw Data Toggle (for debugging) -->
				<div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl p-6">
					<button
						onclick={() => (showRawData = !showRawData)}
						class="w-full text-left flex justify-between items-center"
					>
						<h3 class="text-lg font-bold text-verusidx-stone-dark dark:text-white">
							Raw Data
						</h3>
						<span
							class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist transform transition-transform {showRawData
								? 'rotate-180'
								: ''}"
						>
							▼
						</span>
					</button>

					{#if showRawData}
						<div
							class="mt-4 bg-verusidx-sky-soft dark:bg-verusidx-stone-medium rounded p-4 overflow-auto max-h-96"
						>
							<pre
								class="text-xs text-verusidx-stone-dark dark:text-verusidx-mountain-mist">{JSON.stringify(cacheState.ledger.rawIdentity, null, 2)}</pre>
						</div>
					{/if}
				</div>
			</div>
		{/if}
	</div>
</div>
