<script lang="ts">
	import type { VlottoLedgerData } from '$lib/stores/vlottoCache';

	interface Props {
		ledgerData: VlottoLedgerData;
	}

	let { ledgerData }: Props = $props();

	// Extract phase status (10 phases typical for vlotto)
	const phaseStatus = $derived(ledgerData.phaseStatus || {});

	const phases = [
		{ number: 1, name: 'Initialization', key: 'phase1_initialization' },
		{ number: 2, name: 'Funding', key: 'phase2_funding' },
		{ number: 3, name: 'Timelock', key: 'phase3_timelock' },
		{ number: 4, name: 'Generation', key: 'phase4_generation' },
		{ number: 5, name: 'Data', key: 'phase5_data' },
		{ number: 6, name: 'Marketplace', key: 'phase6_marketplace' },
		{ number: 7, name: 'Sales', key: 'phase7_sales' },
		{ number: 8, name: 'Drawing', key: 'phase8_drawing' },
		{ number: 9, name: 'Payout', key: 'phase9_payout' },
		{ number: 10, name: 'Distribution', key: 'phase10_distribution' }
	];

	// Find the highest completed phase number
	const highestCompletedPhase = $derived(() => {
		let highest = 0;
		for (const phase of phases) {
			if (phaseStatus[phase.key]?.completed === true) {
				highest = phase.number;
			}
		}
		return highest;
	});

	// In-progress phase is the one after the last completed phase
	const inProgressPhaseNumber = $derived(highestCompletedPhase() + 1);
</script>

<div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl p-6">
	<h2 class="text-xl font-bold text-verusidx-stone-dark dark:text-white mb-4">Phase Timeline</h2>

	<div class="space-y-3">
		{#each phases as phase}
			{@const isComplete = phaseStatus[phase.key]?.completed === true}
			{@const isInProgress = phase.number === inProgressPhaseNumber && !isComplete}

			<div class="flex items-center gap-3">
				<!-- Phase number circle -->
				<div
					class="flex-shrink-0 w-10 h-10 rounded-full flex items-center justify-center font-bold text-sm
					{isComplete
						? 'bg-verusidx-turquoise-deep text-white'
						: isInProgress
							? 'bg-blue-600 dark:bg-blue-500 text-white'
							: 'bg-verusidx-sky-soft dark:bg-verusidx-stone-medium text-verusidx-mountain-grey dark:text-verusidx-mountain-mist'}"
				>
					{#if isComplete}
						✓
					{:else}
						{phase.number}
					{/if}
				</div>

				<!-- Phase info -->
				<div class="flex-1">
					<p
						class="font-medium
						{isInProgress
							? 'text-blue-600 dark:text-blue-400'
							: 'text-verusidx-stone-dark dark:text-white'}"
					>
						{phase.name}
					</p>
				</div>

				<!-- Status indicator -->
				<div class="flex-shrink-0">
					{#if isComplete}
						<span
							class="px-2 py-1 text-xs rounded bg-verusidx-turquoise-light/20 text-verusidx-turquoise-deep dark:text-verusidx-turquoise-light"
						>
							Complete
						</span>
					{:else if isInProgress}
						<span
							class="px-2 py-1 text-xs rounded bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400 font-medium"
						>
							In Progress
						</span>
					{:else}
						<span
							class="px-2 py-1 text-xs rounded bg-verusidx-sky-soft dark:bg-verusidx-stone-medium text-verusidx-mountain-grey dark:text-verusidx-mountain-mist"
						>
							Pending
						</span>
					{/if}
				</div>
			</div>
		{/each}
	</div>
</div>
