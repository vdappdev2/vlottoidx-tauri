<script lang="ts">
	import type { VlottoLedgerData } from '$lib/stores/vlottoCache';
	import { formatHexForDisplay } from '$lib/utils/vlottoScoring';

	interface Props {
		ledgerData: VlottoLedgerData;
	}

	let { ledgerData }: Props = $props();

	const drawingHash = $derived(ledgerData.drawingResults.drawingHash);
	const topWinningTicket = $derived(ledgerData.drawingResults.topWinningTicket);
	const hasDrawing = $derived(drawingHash && drawingHash.trim().length > 0);

	// Parse ticket number from index (e.g., "1of1" -> "1")
	const ticketNumber = $derived(() => {
		if (!topWinningTicket?.index) return 'N/A';
		const indexStr = String(topWinningTicket.index);
		const beforeOf = indexStr.split('of')[0];
		return beforeOf || indexStr;
	});
</script>

<div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl p-6">
	<h2 class="text-xl font-bold text-verusidx-stone-dark dark:text-white mb-4">Drawing Results</h2>

	<!-- Drawing ID -->
	{#if ledgerData.drawingId}
		<div class="mb-4 pb-4 border-b border-verusidx-mountain-mist/20">
			<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-1">
				Drawing ID
			</p>
			<p class="font-mono text-sm text-verusidx-stone-dark dark:text-white">
				{ledgerData.drawingId}
			</p>
		</div>
	{/if}

	{#if hasDrawing}
		<div class="space-y-4">
			<!-- Drawing Hash -->
			<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-4 rounded-lg">
				<p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist text-sm mb-2">
					Drawing Hash
				</p>
				<p
					class="font-mono text-sm text-verusidx-stone-dark dark:text-white break-all leading-relaxed"
				>
					{formatHexForDisplay(drawingHash, 16)}
				</p>
			</div>

			<!-- Top Winning Ticket (if available) -->
			{#if topWinningTicket}
				<div class="bg-verusidx-turquoise-light/10 dark:bg-verusidx-turquoise-deep/20 p-4 rounded-lg border border-verusidx-turquoise-light">
					<h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white mb-3">
						Top Winning Ticket
					</h3>

					<div class="grid grid-cols-2 gap-3">
						<!-- Row 1, Column 1: Ticket Name -->
						<div>
							<p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist text-sm">
								Ticket Name
							</p>
							<p class="text-lg font-bold text-verusidx-stone-dark dark:text-white">
								{topWinningTicket.name || topWinningTicket.ticketName || 'N/A'}
							</p>
						</div>

						<!-- Row 1, Column 2: Ticket Number -->
						<div>
							<p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist text-sm">
								Ticket Number
							</p>
							<p class="text-lg font-bold text-verusidx-stone-dark dark:text-white">
								{ticketNumber()}
							</p>
						</div>

						<!-- Row 2, Column 1: Matches -->
						<div>
							<p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist text-sm">
								Matches
							</p>
							<p class="text-lg font-bold text-verusidx-stone-dark dark:text-white">
								{topWinningTicket.matches || 'N/A'}
							</p>
						</div>

						<!-- Row 2, Column 2: Score -->
						<div>
							<p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist text-sm">
								Score
							</p>
							<p class="text-lg font-bold text-verusidx-stone-dark dark:text-white">
								{topWinningTicket.score || 'N/A'}
							</p>
						</div>
					</div>
				</div>
			{/if}
		</div>
	{:else}
		<div class="text-center py-8">
			<p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
				Drawing has not been completed yet
			</p>
		</div>
	{/if}
</div>
