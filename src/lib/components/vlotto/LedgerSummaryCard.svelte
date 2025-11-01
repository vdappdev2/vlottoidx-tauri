<script lang="ts">
	import type { VlottoLedgerData } from '$lib/stores/vlottoCache';
	import { ExpandableCard } from '$lib/components/cards';
	import { formatRelativeTime } from '$lib/utils/vlottoParser';

	interface Props {
		ledgerData: VlottoLedgerData;
	}

	let { ledgerData }: Props = $props();

	// Helper to get phase badge color
	function getPhaseColor(phase: string): string {
		if (phase.includes('complete')) return 'bg-verusidx-turquoise-deep text-white';
		if (phase.includes('phase10') || phase.includes('phase9')) return 'bg-verusidx-mountain-blue text-white';
		return 'bg-verusidx-lake-blue text-white';
	}

	// Helper to get winner status badge color
	function getWinnerStatusColor(status: string | undefined): string {
		if (!status) return 'bg-verusidx-mountain-mist text-verusidx-stone-dark';
		if (status === 'winner') return 'bg-verusidx-turquoise-deep text-white';
		if (status === 'no_winner') return 'bg-verusidx-mountain-grey text-white';
		return 'bg-verusidx-mountain-blue text-white'; // pending
	}

	// Helper to format winner status text
	function formatWinnerStatus(status: string | undefined): string {
		if (!status) return 'Pending';
		if (status === 'winner') return 'Winner';
		if (status === 'no_winner') return 'No Winner';
		return 'Pending';
	}

	// Check if Phase 8 (Drawing) is complete
	const isPhase8Complete = $derived(ledgerData.phaseStatus?.phase8_drawing?.completed === true);
</script>

<ExpandableCard title="Ledger Summary" modalSize="xl">
	<!-- Preview Slot -->
	<div slot="preview">
		<div class="grid grid-cols-2 md:grid-cols-3 gap-4">
			<!-- Current Phase -->
			<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-4 rounded-lg">
				<p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist text-sm mb-2">
					Current Phase
				</p>
				<span class="{getPhaseColor(ledgerData.currentPhase)} px-3 py-1 rounded-full text-sm font-semibold inline-block">
					{ledgerData.currentPhase.replace('phase', 'Phase ').replace('_', ' - ')}
				</span>
			</div>

			<!-- Drawing Block -->
			<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-4 rounded-lg">
				<p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist text-sm mb-1">
					Drawing Block
				</p>
				<p class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">
					{ledgerData.lotteryParameters.drawingBlock.toLocaleString()}
				</p>
			</div>

			<!-- Ticket Price -->
			<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-4 rounded-lg">
				<p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist text-sm mb-1">
					Ticket Price
				</p>
				<p class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">
					{ledgerData.lotteryParameters.ticketPrice !== undefined ? ledgerData.lotteryParameters.ticketPrice : 'N/A'}
				</p>
			</div>

			<!-- Jackpot Current (if available) -->
			{#if ledgerData.financialSummary?.jackpotCurrent !== undefined}
				<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-4 rounded-lg">
					<p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist text-sm mb-1">
						Current Jackpot
					</p>
					<p class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">
						{ledgerData.financialSummary.jackpotCurrent.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 8 })}
					</p>
				</div>
			{/if}

			<!-- Tickets Sold -->
			<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-4 rounded-lg">
				<p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist text-sm mb-1">
					Tickets Sold
				</p>
				<p class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">
					{ledgerData.ticketSummary.sold !== undefined ? ledgerData.ticketSummary.sold : 'N/A'} of {ledgerData.ticketSummary.planned}
				</p>
			</div>

			<!-- Required Matches -->
			<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-4 rounded-lg">
				<p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist text-sm mb-1">
					Required Matches
				</p>
				<p class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">
					{ledgerData.lotteryParameters.requiredMatches}
				</p>
			</div>

			<!-- Winner Status (if Phase 8+ complete) -->
			{#if isPhase8Complete}
				<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-4 rounded-lg">
					<p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist text-sm mb-2">
						Winner Status
					</p>
					<span class="{getWinnerStatusColor(ledgerData.drawingResults.winnerStatus)} px-3 py-1 rounded-full text-sm font-semibold inline-block">
						{formatWinnerStatus(ledgerData.drawingResults.winnerStatus)}
					</span>
				</div>
			{/if}

			<!-- Main ID -->
			<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-4 rounded-lg">
				<p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist text-sm mb-1">
					Main ID
				</p>
				<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white break-all">
					{ledgerData.lotteryParameters.mainVerusID}
				</p>
			</div>

			<!-- Last Updated -->
			{#if ledgerData.lastUpdated}
				<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-4 rounded-lg">
					<p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist text-sm mb-1">
						Last Updated
					</p>
					<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white">
						{formatRelativeTime(ledgerData.lastUpdated)}
					</p>
				</div>
			{/if}
		</div>
	</div>

	<!-- Expanded Slot -->
	<div slot="expanded">
		<div class="space-y-4 max-h-[70vh] overflow-y-auto pr-2">
			<!-- Drawing Parameters Section -->
			<details class="border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg">
				<summary class="p-3 cursor-pointer text-verusidx-stone-dark dark:text-white font-medium hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors">
					Drawing Parameters
				</summary>
				<div class="p-3 space-y-3 border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
				<div class="grid grid-cols-1 md:grid-cols-2 gap-3">
					<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
						<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Ticket Price</p>
						<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white">
							{ledgerData.lotteryParameters.ticketPrice !== undefined ? ledgerData.lotteryParameters.ticketPrice : 'N/A'}
						</p>
					</div>
					<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
						<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Ticket Multiplier</p>
						<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white">
							{ledgerData.lotteryParameters.ticketMultiplier !== undefined ? ledgerData.lotteryParameters.ticketMultiplier : 'N/A'}
						</p>
					</div>
					<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
						<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Jackpot Minimum</p>
						<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white">
							{ledgerData.lotteryParameters.jackpotMinimum !== undefined ? ledgerData.lotteryParameters.jackpotMinimum : 'N/A'}
						</p>
					</div>
					<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
						<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Jackpot Ceiling Cap</p>
						<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white">
							{ledgerData.lotteryParameters.jackpotCeilingCap !== undefined ? ledgerData.lotteryParameters.jackpotCeilingCap.toLocaleString() : 'N/A'}
						</p>
					</div>
					<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
						<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Start Block</p>
						<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white">
							{ledgerData.lotteryParameters.startBlock !== undefined ? ledgerData.lotteryParameters.startBlock.toLocaleString() : 'N/A'}
						</p>
					</div>
					<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
						<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Target Drawing Block</p>
						<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white">
							{ledgerData.lotteryParameters.targetDrawingBlock !== undefined ? ledgerData.lotteryParameters.targetDrawingBlock.toLocaleString() : 'N/A'}
						</p>
					</div>
					<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
						<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Grace Period (blocks)</p>
						<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white">
							{ledgerData.lotteryParameters.gracePeriod !== undefined ? ledgerData.lotteryParameters.gracePeriod.toLocaleString() : 'N/A'}
						</p>
					</div>
					<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
						<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Confirmations</p>
						<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white">
							{ledgerData.lotteryParameters.confirmations !== undefined ? ledgerData.lotteryParameters.confirmations : 'N/A'}
						</p>
					</div>
					<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
						<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Payout Offer Expiry (blocks)</p>
						<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white">
							{ledgerData.lotteryParameters.payoutOfferExpiry !== undefined ? ledgerData.lotteryParameters.payoutOfferExpiry.toLocaleString() : 'N/A'}
						</p>
					</div>
					<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
						<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Next Jackpot %</p>
						<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white">
							{ledgerData.lotteryParameters.nextJackpotPercent !== undefined ? ledgerData.lotteryParameters.nextJackpotPercent + '%' : 'N/A'}
						</p>
					</div>
					<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
						<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Operations %</p>
						<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white">
							{ledgerData.lotteryParameters.operationsPercent !== undefined ? ledgerData.lotteryParameters.operationsPercent + '%' : 'N/A'}
						</p>
					</div>
				</div>
				</div>
			</details>

			<!-- Ticket Summary Section -->
			<details class="border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg">
				<summary class="p-3 cursor-pointer text-verusidx-stone-dark dark:text-white font-medium hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors">
					Ticket Summary
				</summary>
				<div class="p-3 space-y-3 border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
				<div class="grid grid-cols-2 md:grid-cols-4 gap-3">
					<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
						<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Planned</p>
						<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white">{ledgerData.ticketSummary.planned}</p>
					</div>
					<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
						<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Generated</p>
						<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white">
							{ledgerData.ticketSummary.generated !== undefined ? ledgerData.ticketSummary.generated : 'N/A'}
						</p>
					</div>
					<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
						<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Registered</p>
						<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white">
							{ledgerData.ticketSummary.registered !== undefined ? ledgerData.ticketSummary.registered : 'N/A'}
						</p>
					</div>
					<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
						<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Data Updated</p>
						<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white">
							{ledgerData.ticketSummary.dataUpdated !== undefined ? ledgerData.ticketSummary.dataUpdated : 'N/A'}
						</p>
					</div>
					<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
						<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">On Marketplace</p>
						<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white">
							{ledgerData.ticketSummary.onMarketplace !== undefined ? ledgerData.ticketSummary.onMarketplace : 'N/A'}
						</p>
					</div>
					<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
						<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Sold</p>
						<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white">
							{ledgerData.ticketSummary.sold !== undefined ? ledgerData.ticketSummary.sold : 'N/A'}
						</p>
					</div>
					<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
						<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Verified</p>
						<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white">
							{ledgerData.ticketSummary.verified !== undefined ? ledgerData.ticketSummary.verified : 'N/A'}
						</p>
					</div>
				</div>

				{#if ledgerData.ticketSummary.verificationResults}
					<div class="mt-3 p-3 bg-verusidx-turquoise-light/10 dark:bg-verusidx-turquoise-deep/20 rounded border border-verusidx-turquoise-light">
						<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white mb-2">Verification Results</p>
						<div class="grid grid-cols-2 md:grid-cols-4 gap-2 text-xs">
							<div>
								<p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Total Processed</p>
								<p class="font-semibold text-verusidx-stone-dark dark:text-white">
									{ledgerData.ticketSummary.verificationResults.totalProcessed ?? 'N/A'}
								</p>
							</div>
							<div>
								<p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Passed</p>
								<p class="font-semibold text-verusidx-turquoise-deep dark:text-verusidx-turquoise-light">
									{ledgerData.ticketSummary.verificationResults.passedVerification ?? 'N/A'}
								</p>
							</div>
							<div>
								<p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Failed</p>
								<p class="font-semibold text-verusidx-stone-dark dark:text-white">
									{ledgerData.ticketSummary.verificationResults.failedVerification ?? 'N/A'}
								</p>
							</div>
							<div>
								<p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Fraudulent</p>
								<p class="font-semibold {ledgerData.ticketSummary.verificationResults.fraudulentTickets ? 'text-red-600 dark:text-red-400' : 'text-verusidx-stone-dark dark:text-white'}">
									{ledgerData.ticketSummary.verificationResults.fraudulentTickets ?? '0'}
								</p>
							</div>
						</div>
					</div>
				{/if}
				</div>
			</details>

			<!-- Drawing Results Section (if Phase 8+ complete) -->
			{#if isPhase8Complete}
			<details class="border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg">
				<summary class="p-3 cursor-pointer text-verusidx-stone-dark dark:text-white font-medium hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors">
					Drawing Results
				</summary>
				<div class="p-3 space-y-3 border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
					<div class="grid grid-cols-1 md:grid-cols-2 gap-3">
						<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
							<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Winner Status</p>
							<span class="{getWinnerStatusColor(ledgerData.drawingResults.winnerStatus)} px-3 py-1 rounded-full text-sm font-semibold inline-block mt-1">
								{formatWinnerStatus(ledgerData.drawingResults.winnerStatus)}
							</span>
						</div>
						<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
							<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Verification Status</p>
							<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white mt-1">
								{ledgerData.drawingResults.verificationStatus || 'N/A'}
							</p>
						</div>
						<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
							<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Drawing Method</p>
							<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white mt-1">
								{ledgerData.drawingResults.drawingMethod || 'N/A'}
							</p>
						</div>
						<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
							<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Top Ticket Authentic</p>
							<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white mt-1">
								{ledgerData.drawingResults.topTicketAuthentic !== undefined ? (ledgerData.drawingResults.topTicketAuthentic ? 'Yes' : 'No') : 'N/A'}
							</p>
						</div>
						{#if ledgerData.drawingResults.drawingTimestamp}
							<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded col-span-2">
								<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Drawing Timestamp</p>
								<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white mt-1">
									{new Date(ledgerData.drawingResults.drawingTimestamp).toLocaleString()}
								</p>
							</div>
						{/if}
					</div>
				</div>
			</details>
			{/if}

			<!-- Financial Summary Section -->
			{#if ledgerData.financialSummary}
			<details class="border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg">
				<summary class="p-3 cursor-pointer text-verusidx-stone-dark dark:text-white font-medium hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors">
					Financial Summary
				</summary>
				<div class="p-3 space-y-3 border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
					<div class="grid grid-cols-1 md:grid-cols-2 gap-3">
						{#if ledgerData.financialSummary.jackpotStart !== undefined}
							<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
								<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Starting Jackpot</p>
								<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white">
									{ledgerData.financialSummary.jackpotStart.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 8 })}
								</p>
							</div>
						{/if}
						{#if ledgerData.financialSummary.jackpotCurrent !== undefined}
							<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
								<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Current Jackpot</p>
								<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white">
									{ledgerData.financialSummary.jackpotCurrent.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 8 })}
								</p>
							</div>
						{/if}
					</div>
				</div>
			</details>
			{/if}

			<!-- Timelock Status Section -->
			{#if ledgerData.timelockStatus}
			<details class="border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg">
				<summary class="p-3 cursor-pointer text-verusidx-stone-dark dark:text-white font-medium hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors">
					Timelock Status
				</summary>
				<div class="p-3 space-y-3 border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
					<div class="grid grid-cols-2 md:grid-cols-4 gap-3">
						<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
							<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Applied</p>
							<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white">
								{ledgerData.timelockStatus.applied !== undefined ? (ledgerData.timelockStatus.applied ? 'Yes' : 'No') : 'N/A'}
							</p>
						</div>
						<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
							<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Lock Height</p>
							<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white">
								{ledgerData.timelockStatus.lockHeight !== undefined ? ledgerData.timelockStatus.lockHeight.toLocaleString() : 'N/A'}
							</p>
						</div>
						<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
							<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Unlock Height</p>
							<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white">
								{ledgerData.timelockStatus.unlockHeight !== undefined ? ledgerData.timelockStatus.unlockHeight.toLocaleString() : 'N/A'}
							</p>
						</div>
						<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
							<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Currently Locked</p>
							<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white">
								{ledgerData.timelockStatus.currentlyLocked !== undefined ? (ledgerData.timelockStatus.currentlyLocked ? 'Yes' : 'No') : 'N/A'}
							</p>
						</div>
					</div>
				</div>
			</details>
			{/if}

			<!-- Marketplace Status Section -->
			{#if ledgerData.marketplaceStatus}
			<details class="border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg">
				<summary class="p-3 cursor-pointer text-verusidx-stone-dark dark:text-white font-medium hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors">
					Marketplace Status
				</summary>
				<div class="p-3 space-y-3 border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
					<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded inline-block">
						<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Tickets Listed</p>
						<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white">
							{ledgerData.marketplaceStatus.ticketsListed !== undefined ? ledgerData.marketplaceStatus.ticketsListed : 'N/A'}
						</p>
					</div>
				</div>
			</details>
			{/if}

			<!-- Payout & Distribution Section -->
			{#if ledgerData.payoutSummary || ledgerData.distributionSummary}
			<details class="border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg">
				<summary class="p-3 cursor-pointer text-verusidx-stone-dark dark:text-white font-medium hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors">
					Payout & Distribution
				</summary>
				<div class="p-3 space-y-3 border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
					<div class="grid grid-cols-1 md:grid-cols-2 gap-3">
						{#if ledgerData.payoutSummary}
							<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
								<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-2">Payout Summary</p>
								<div class="space-y-1 text-xs">
									<div class="flex justify-between">
										<span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Winner Was Sold:</span>
										<span class="font-semibold text-verusidx-stone-dark dark:text-white">
											{ledgerData.payoutSummary.winnerWasSold !== undefined ? (ledgerData.payoutSummary.winnerWasSold ? 'Yes' : 'No') : 'N/A'}
										</span>
									</div>
									<div class="flex justify-between">
										<span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Payout Required:</span>
										<span class="font-semibold text-verusidx-stone-dark dark:text-white">
											{ledgerData.payoutSummary.payoutRequired !== undefined ? (ledgerData.payoutSummary.payoutRequired ? 'Yes' : 'No') : 'N/A'}
										</span>
									</div>
									{#if ledgerData.payoutSummary.buybackTxID}
										<div class="mt-2 pt-2 border-t border-verusidx-mountain-mist/20">
											<p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Buyback TxID:</p>
											<p class="font-mono text-xs text-verusidx-stone-dark dark:text-white break-all">
												{ledgerData.payoutSummary.buybackTxID}
											</p>
										</div>
									{/if}
								</div>
							</div>
						{/if}
						{#if ledgerData.distributionSummary}
							<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
								<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-2">Distribution Summary</p>
								<div class="space-y-1 text-xs">
									{#if ledgerData.distributionSummary.totalDistributed !== undefined}
										<div class="flex justify-between">
											<span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Total Distributed:</span>
											<span class="font-semibold text-verusidx-stone-dark dark:text-white">
												{ledgerData.distributionSummary.totalDistributed.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 8 })}
											</span>
										</div>
									{/if}
									{#if ledgerData.distributionSummary.distributionTxID}
										<div class="mt-2 pt-2 border-t border-verusidx-mountain-mist/20">
											<p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Distribution TxID:</p>
											<p class="font-mono text-xs text-verusidx-stone-dark dark:text-white break-all">
												{ledgerData.distributionSummary.distributionTxID}
											</p>
										</div>
									{/if}
								</div>
							</div>
						{/if}
					</div>
				</div>
			</details>
			{/if}

			<!-- Security & Operations Section -->
			{#if ledgerData.securityMetrics || ledgerData.operationalMetrics}
			<details class="border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg">
				<summary class="p-3 cursor-pointer text-verusidx-stone-dark dark:text-white font-medium hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors">
					Security & Operations
				</summary>
				<div class="p-3 space-y-3 border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
					<div class="grid grid-cols-1 md:grid-cols-2 gap-3">
						{#if ledgerData.securityMetrics}
							<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
								<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-2">Security Metrics</p>
								<div class="text-xs">
									<div class="flex justify-between">
										<span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Fraud Detection Checks:</span>
										<span class="font-semibold text-verusidx-stone-dark dark:text-white">
											{ledgerData.securityMetrics.fraudDetectionChecks !== undefined ? ledgerData.securityMetrics.fraudDetectionChecks : 'N/A'}
										</span>
									</div>
								</div>
							</div>
						{/if}
						{#if ledgerData.operationalMetrics}
							<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
								<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-2">Operational Metrics</p>
								<div class="text-xs space-y-1">
									{#if ledgerData.operationalMetrics.retryOperations}
										<div class="flex justify-between">
											<span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Total Retries:</span>
											<span class="font-semibold text-verusidx-stone-dark dark:text-white">
												{ledgerData.operationalMetrics.retryOperations.totalRetries ?? 'N/A'}
											</span>
										</div>
									{/if}
									{#if ledgerData.operationalMetrics.utxoOperations}
										<div class="flex justify-between">
											<span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">UTXO Balance:</span>
											<span class="font-semibold text-verusidx-stone-dark dark:text-white">
												{ledgerData.operationalMetrics.utxoOperations.utxoBalance ?? 'N/A'}
											</span>
										</div>
									{/if}
								</div>
							</div>
						{/if}
					</div>
				</div>
			</details>
			{/if}

			<!-- Ledger Metadata Section -->
			<details class="border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg">
				<summary class="p-3 cursor-pointer text-verusidx-stone-dark dark:text-white font-medium hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors">
					Ledger Metadata
				</summary>
				<div class="p-3 space-y-3 border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
				<div class="grid grid-cols-1 md:grid-cols-2 gap-3">
					{#if ledgerData.ledgerVersion}
						<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
							<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Ledger Version</p>
							<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white">{ledgerData.ledgerVersion}</p>
						</div>
					{/if}
					{#if ledgerData.lastUpdated}
						<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded">
							<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Last Updated</p>
							<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white">
								{new Date(ledgerData.lastUpdated).toLocaleString()}
							</p>
						</div>
					{/if}
					{#if ledgerData.drawingId}
						<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded col-span-2">
							<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">Drawing ID</p>
							<p class="text-sm font-semibold text-verusidx-stone-dark dark:text-white font-mono">{ledgerData.drawingId}</p>
						</div>
					{/if}
				</div>
				</div>
			</details>
		</div>
	</div>
</ExpandableCard>
