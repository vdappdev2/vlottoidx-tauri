<script lang="ts">
	import type { VlottoTicketData, VlottoLedgerData } from '$lib/stores/vlottoCache';
	import { vlottoCache } from '$lib/stores/vlottoCache';
	import { enumerateAndFetchTickets } from '$lib/services/vlottoService';
	import { highlightMatches } from '$lib/utils/vlottoScoring';
	import { verifyTicketAuthenticity, type VerificationResult } from '$lib/utils/vlottoVerification';
	import { connectionStore, getChainParam } from '$lib/stores/connection';

	interface Props {
		ledgerData: VlottoLedgerData;
	}

	let { ledgerData }: Props = $props();

	let connState = $state($connectionStore);
	connectionStore.subscribe((state) => {
		connState = state;
	});

	let isExpanded = $state(false);
	let isLoadingTickets = $state(false);
	let fetchProgress = $state({ current: 0, total: 0 });
	let phase7RefreshInterval: ReturnType<typeof setInterval> | null = null;

	// Verification state
	let verifyingTicket = $state<string | null>(null);
	let verificationResults = $state<Map<string, VerificationResult>>(new Map());

	// Subscribe to cache for ticket scores
	let ticketScores: VlottoTicketData[] = $state([]);
	vlottoCache.subscribe((state) => {
		ticketScores = state.ticketScores;
	});

	// Pagination
	let currentPage = $state(1);
	let itemsPerPage = $state(10);
	let paginatedTickets = $state<VlottoTicketData[]>([]);
	let totalPages = $state(0);

	// Effect to handle pagination
	$effect(() => {
		if (!ticketScores || ticketScores.length === 0) {
			paginatedTickets = [];
			totalPages = 0;
			return;
		}

		const newTotalPages = Math.ceil(ticketScores.length / itemsPerPage);
		const validCurrentPage = Math.max(1, Math.min(currentPage, newTotalPages || 1));

		const startIndex = (validCurrentPage - 1) * itemsPerPage;
		const endIndex = startIndex + itemsPerPage;

		paginatedTickets = ticketScores.slice(startIndex, endIndex);
		totalPages = newTotalPages;

		if (currentPage !== validCurrentPage) {
			currentPage = validCurrentPage;
		}
	});

	// Auto-refresh when Phase 7 (Sales) completes
	let lastPhase7Status = $state<boolean | undefined>(undefined);
	$effect(() => {
		const phase7Complete = ledgerData.phaseStatus?.phase7_sales?.completed === true;

		// Check if Phase 7 just completed (transition from false/undefined to true)
		if (phase7Complete && lastPhase7Status === false && ticketScores.length > 0) {
			loadTickets(true);
		}

		lastPhase7Status = phase7Complete;
	});

	// Auto-refresh when Phase 8 (Drawing) completes
	let lastPhase8Status = $state<boolean | undefined>(undefined);
	$effect(() => {
		const phase8Complete = ledgerData.phaseStatus?.phase8_drawing?.completed === true;

		// Check if Phase 8 just completed (transition from false/undefined to true)
		if (phase8Complete && lastPhase8Status === false && ticketScores.length > 0) {
			loadTickets(true);
		}

		lastPhase8Status = phase8Complete;
	});

	// Periodic refresh during Phase 7 (Sales) when expanded
	$effect(() => {
		const phase7Active = ledgerData.phaseStatus?.phase7_sales?.completed === false;
		const shouldRefresh = phase7Active && isExpanded && ticketScores.length > 0;

		// Clear any existing interval
		if (phase7RefreshInterval) {
			clearInterval(phase7RefreshInterval);
			phase7RefreshInterval = null;
		}

		// Start new interval if conditions met
		if (shouldRefresh) {
			phase7RefreshInterval = setInterval(() => {
				loadTickets(true);
			}, 90000); // 90 seconds
		}

		// Cleanup function
		return () => {
			if (phase7RefreshInterval) {
				clearInterval(phase7RefreshInterval);
				phase7RefreshInterval = null;
			}
		};
	});

	async function loadTickets(forceRefresh: boolean = false) {
		if (ticketScores.length > 0 && !forceRefresh) {
			// Already loaded and not forcing refresh
			isExpanded = true;
			return;
		}

		isLoadingTickets = true;
		fetchProgress = { current: 0, total: ledgerData.ticketSummary.planned };

		try {
			// Clear cache if force refreshing
			if (forceRefresh) {
				vlottoCache.invalidateTickets();
			}

			const chain = getChainParam(connState.selectedChain) || undefined;
			await enumerateAndFetchTickets(chain, (current, total) => {
				fetchProgress = { current, total };
			});
			isExpanded = true;
		} catch (error: any) {
			alert(`Failed to load tickets: ${error.toString()}`);
		} finally {
			isLoadingTickets = false;
		}
	}

	function toggleExpanded() {
		if (!isExpanded && ticketScores.length === 0) {
			loadTickets();
		} else {
			isExpanded = !isExpanded;
		}
	}

	async function verifyTicket(ticket: VlottoTicketData) {
		verifyingTicket = ticket.name;
		try {
			const chain = getChainParam(connState.selectedChain) || undefined;

			const result = await verifyTicketAuthenticity(
				ticket,
				ledgerData.lotteryParameters.mainVerusID,
				chain
			);

			verificationResults = new Map(verificationResults).set(ticket.name, result);
		} catch (error: any) {
			alert(`Verification failed: ${error.toString()}`);
		} finally {
			verifyingTicket = null;
		}
	}

	function goToPage(page: number) {
		if (page >= 1 && page <= totalPages) {
			currentPage = page;
		}
	}

	function nextPage() {
		if (currentPage < totalPages) {
			currentPage++;
		}
	}

	function prevPage() {
		if (currentPage > 1) {
			currentPage--;
		}
	}
</script>

<div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl p-6">
	<div class="flex justify-between items-center mb-4">
		<h2 class="text-xl font-bold text-verusidx-stone-dark dark:text-white">Ticket List</h2>
		<div class="flex gap-2">
			{#if isExpanded && ticketScores.length > 0}
				<button
					onclick={() => loadTickets(true)}
					class="px-4 py-2 rounded-lg transition-colors
						{isLoadingTickets
						? 'bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-mountain-grey cursor-not-allowed'
						: 'bg-verusidx-mountain-blue text-white hover:bg-verusidx-lake-blue'}"
					disabled={isLoadingTickets}
				>
					{isLoadingTickets ? 'Refreshing...' : 'Refresh'}
				</button>
			{/if}
			<button
				onclick={toggleExpanded}
				class="px-4 py-2 rounded-lg transition-colors
					{isLoadingTickets
					? 'bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-mountain-grey cursor-not-allowed'
					: 'bg-verusidx-turquoise-deep text-white hover:bg-verusidx-turquoise-bright'}"
				disabled={isLoadingTickets}
			>
				{#if isLoadingTickets}
					Loading {fetchProgress.current}/{fetchProgress.total}...
				{:else if isExpanded}
					Collapse
				{:else}
					Load Tickets ({ledgerData.ticketSummary.planned})
				{/if}
			</button>
		</div>
	</div>

	{#if isLoadingTickets}
		<div class="text-center py-8">
			<div
				class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-verusidx-turquoise-deep mb-4"
			></div>
			<p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
				Fetching tickets... {fetchProgress.current} of {fetchProgress.total}
			</p>
		</div>
	{:else if isExpanded && ticketScores.length > 0}
		<!-- Pagination Info -->
		<div class="flex justify-between items-center text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-4">
			<span>Showing {((currentPage - 1) * itemsPerPage) + 1} to {Math.min(currentPage * itemsPerPage, ticketScores.length)} of {ticketScores.length} tickets</span>
			<span>Page {currentPage} of {totalPages}</span>
		</div>

		<div class="space-y-3">
			{#each paginatedTickets as ticket, i}
				{@const actualIndex = (currentPage - 1) * itemsPerPage + i}
				{@const isTopWinner = actualIndex === 0 && ticket.matches >= ledgerData.lotteryParameters.requiredMatches}
				{@const isQualified = ticket.matches >= ledgerData.lotteryParameters.requiredMatches}
				{@const highlighted = ledgerData.drawingResults.drawingHash && ledgerData.drawingResults.drawingHash.trim() !== ''
					? highlightMatches(ticket.playingNumber, ledgerData.drawingResults.drawingHash)
					: ticket.playingNumber.split('').map(char => ({ char, matches: false, isLeadingZero: false }))}

				<div
					class="p-4 rounded-lg border
					{isTopWinner
						? 'bg-verusidx-turquoise-light/10 border-verusidx-turquoise-deep'
						: isQualified
							? 'bg-verusidx-sky-soft/50 dark:bg-verusidx-stone-medium/50 border-verusidx-mountain-blue'
							: 'bg-verusidx-sky-soft dark:bg-verusidx-stone-medium border-transparent'}"
				>
					<div class="flex justify-between items-start mb-2">
						<div>
							<p class="text-base font-medium text-verusidx-stone-dark dark:text-white">
								{ticket.name}
							</p>
							<div class="flex gap-2 mt-1">
								<span
									class="px-2 py-0.5 text-xs rounded
									{ticket.claimed
										? 'bg-orange-500 dark:bg-orange-400 text-white dark:text-orange-900 font-semibold'
										: ticket.discarded
											? 'bg-gray-200 dark:bg-gray-900/30 text-gray-700 dark:text-gray-400'
											: ticket.sold
												? 'bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400'
												: 'bg-amber-100 dark:bg-amber-900/30 text-amber-700 dark:text-amber-400'}"
								>
									{ticket.claimed ? 'Claimed' : ticket.discarded ? 'Discarded' : ticket.sold ? 'Sold' : 'Unsold'}
								</span>
								{#if isTopWinner}
									<span
										class="px-2 py-0.5 text-xs rounded bg-verusidx-turquoise-deep text-white font-bold"
									>
										TOP WINNER
									</span>
								{:else if isQualified}
									<span
										class="px-2 py-0.5 text-xs rounded bg-verusidx-mountain-blue text-white"
									>
										Qualified
									</span>
								{/if}
							</div>
						</div>

						<div class="text-right">
							<p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
								Matches: <span class="font-bold text-verusidx-stone-dark dark:text-white"
									>{ticket.matches}</span
								>
							</p>
							<p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
								Score: <span class="font-bold text-verusidx-stone-dark dark:text-white"
									>{ticket.score}</span
								>
							</p>
						</div>
					</div>

					<!-- Playing Number with matches highlighted -->
					<div class="mt-2">
						<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-1">
							Playing Number:
						</p>
						<div class="font-mono text-xs leading-relaxed">
							{#each highlighted as { char, matches, isLeadingZero }}
								<span
									class="
									{matches
										? 'bg-verusidx-turquoise-deep text-white px-0.5 rounded'
										: isLeadingZero
											? 'text-verusidx-mountain-grey/50'
											: 'text-verusidx-stone-dark dark:text-white'}"
								>
									{char}
								</span>
							{/each}
						</div>
					</div>

					<!-- Verification Section (for top winner or qualified tickets) -->
					{#if (isTopWinner || isQualified) && ticket.ticketValidation}
						{@const verification = verificationResults.get(ticket.name)}
						<div class="mt-3 pt-3 border-t border-verusidx-mountain-mist/20">
							<div class="flex items-center justify-between">
								<p class="text-xs font-semibold text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
									Cryptographic Verification
								</p>

								{#if !verification}
									<button
										onclick={() => verifyTicket(ticket)}
										disabled={verifyingTicket === ticket.name}
										class="px-3 py-1 text-xs rounded transition-colors
											{verifyingTicket === ticket.name
												? 'bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-mountain-grey cursor-not-allowed'
												: 'bg-verusidx-turquoise-deep text-white hover:bg-verusidx-turquoise-bright'}"
									>
										{verifyingTicket === ticket.name ? 'Verifying...' : 'Verify'}
									</button>
								{:else}
									<span
										class="px-2 py-1 text-xs rounded font-medium
											{verification.success
												? 'bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400'
												: 'bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-400'}"
									>
										{verification.success ? '✓ Verified' : '✗ Failed'}
									</span>
								{/if}
							</div>

							<!-- Detailed verification results -->
							{#if verification}
								<div class="mt-2 space-y-1">
									<div class="flex items-center gap-2 text-xs">
										<span class={verification.checks.ticketSignedRegistration ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}>
											{verification.checks.ticketSignedRegistration ? '✓' : '✗'}
										</span>
										<span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
											Ticket signed registration
										</span>
									</div>
									<div class="flex items-center gap-2 text-xs">
										<span class={verification.checks.ticketSignedHash ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}>
											{verification.checks.ticketSignedHash ? '✓' : '✗'}
										</span>
										<span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
											Ticket signed content hash
										</span>
									</div>
									<div class="flex items-center gap-2 text-xs">
										<span class={verification.checks.proofguardSignedTicketSig ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}>
											{verification.checks.proofguardSignedTicketSig ? '✓' : '✗'}
										</span>
										<span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
											Proofguard acknowledged ticket
										</span>
									</div>
									<div class="flex items-center gap-2 text-xs">
										<span class={verification.checks.proofguardSignedHash ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}>
											{verification.checks.proofguardSignedHash ? '✓' : '✗'}
										</span>
										<span class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
											Proofguard signed content hash
										</span>
									</div>

									{#if verification.errors.length > 0}
										<div class="mt-2 p-2 bg-red-50 dark:bg-red-900/10 rounded">
											<p class="text-xs text-red-700 dark:text-red-400 font-medium mb-1">Errors:</p>
											{#each verification.errors as error}
												<p class="text-xs text-red-600 dark:text-red-500">{error}</p>
											{/each}
										</div>
									{/if}
								</div>
							{/if}
						</div>
					{/if}
				</div>
			{/each}
		</div>

		<!-- Pagination Controls -->
		{#if totalPages > 1}
			<div class="flex justify-center mt-6">
				<div class="flex items-center space-x-2">
					<button
						onclick={prevPage}
						disabled={currentPage === 1}
						class="px-3 py-1 text-sm bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-stone-dark dark:text-white rounded hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-dark disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
					>
						Previous
					</button>

					<!-- Page numbers -->
					<div class="flex items-center space-x-1">
						{#each Array(Math.min(5, totalPages)) as _, i}
							{@const pageNum = Math.max(1, Math.min(totalPages - 4, currentPage - 2)) + i}
							{#if pageNum <= totalPages}
								<button
									onclick={() => goToPage(pageNum)}
									class="px-3 py-1 text-sm rounded transition-colors {currentPage === pageNum
										? 'bg-verusidx-mountain-blue text-white'
										: 'bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-stone-dark dark:text-white hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-dark'}"
								>
									{pageNum}
								</button>
							{/if}
						{/each}
					</div>

					<button
						onclick={nextPage}
						disabled={currentPage === totalPages}
						class="px-3 py-1 text-sm bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-stone-dark dark:text-white rounded hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-dark disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
					>
						Next
					</button>
				</div>
			</div>
		{/if}
	{:else if isExpanded}
		<div class="text-center py-8">
			<p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
				No tickets loaded yet
			</p>
		</div>
	{:else}
		<div class="text-center py-8">
			<p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
				Click "Load Tickets" to enumerate and score all tickets
			</p>
			<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-2">
				This will fetch {ledgerData.ticketSummary.planned} tickets from the blockchain
			</p>
		</div>
	{/if}
</div>
