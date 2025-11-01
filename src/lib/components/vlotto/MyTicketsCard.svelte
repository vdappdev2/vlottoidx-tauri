<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import type { VlottoLedgerData } from '$lib/stores/vlottoCache';
	import { connectionStore, getChainParam } from '$lib/stores/connection';
	import Modal from '$lib/components/cards/Modal.svelte';

	interface Props {
		ledgerData: VlottoLedgerData;
		vlottoParent: string | null;
	}

	let { ledgerData, vlottoParent }: Props = $props();

	let connState = $state($connectionStore);
	connectionStore.subscribe((state) => {
		connState = state;
	});

	// Ticket data structures
	interface MyTicket {
		name: string;
		identityAddress: string; // i-address for RPC calls
		index: string; // e.g., "6of9"
		drawingBlock: number;
		identity: any; // Full identity object
	}

	let isExpanded = $state(false);
	let isLoading = $state(false);
	let error = $state<string | null>(null);
	let allMyTickets = $state<MyTicket[]>([]);
	let thisLotteryTickets = $state<MyTicket[]>([]);
	let previousLotteryTickets = $state<MyTicket[]>([]);
	let thisLotteryExpanded = $state(true);
	let previousLotteryExpanded = $state(false);

	// Pagination for Previous Lottery
	let prevLotteryCurrentPage = $state(1);
	let prevLotteryItemsPerPage = $state(10);
	let prevLotteryPaginated = $state<MyTicket[]>([]);
	let prevLotteryTotalPages = $state(0);

	// Graveyard modal state
	let showGraveyardModal = $state(false);
	let selectedTicket = $state<MyTicket | null>(null);
	let sourceOfFunds = $state('');
	let isSending = $state(false);
	let sendError = $state<string | null>(null);
	let sendSuccess = $state(false);

	// Effect to re-filter tickets when drawing block changes
	$effect(() => {
		if (allMyTickets.length === 0) return;

		const currentDrawingBlock = ledgerData.lotteryParameters.drawingBlock;

		// Re-split tickets based on current drawing block
		thisLotteryTickets = allMyTickets
			.filter((t) => t.drawingBlock === currentDrawingBlock)
			.sort((a, b) => {
				const aNum = parseInt(a.index.split('of')[0]);
				const bNum = parseInt(b.index.split('of')[0]);
				return aNum - bNum;
			});

		previousLotteryTickets = allMyTickets
			.filter((t) => t.drawingBlock !== currentDrawingBlock)
			.sort((a, b) => {
				if (a.drawingBlock !== b.drawingBlock) {
					return b.drawingBlock - a.drawingBlock;
				}
				const aNum = parseInt(a.index.split('of')[0]);
				const bNum = parseInt(b.index.split('of')[0]);
				return aNum - bNum;
			});
	});

	// Effect to handle Previous Lottery pagination
	$effect(() => {
		if (!previousLotteryTickets || previousLotteryTickets.length === 0) {
			prevLotteryPaginated = [];
			prevLotteryTotalPages = 0;
			return;
		}

		const newTotalPages = Math.ceil(previousLotteryTickets.length / prevLotteryItemsPerPage);
		const validCurrentPage = Math.max(1, Math.min(prevLotteryCurrentPage, newTotalPages || 1));

		const startIndex = (validCurrentPage - 1) * prevLotteryItemsPerPage;
		const endIndex = startIndex + prevLotteryItemsPerPage;

		prevLotteryPaginated = previousLotteryTickets.slice(startIndex, endIndex);
		prevLotteryTotalPages = newTotalPages;

		if (prevLotteryCurrentPage !== validCurrentPage) {
			prevLotteryCurrentPage = validCurrentPage;
		}
	});

	async function loadMyTickets() {
		if (!vlottoParent) {
			error = 'vlotto parent i-address not available';
			return;
		}

		isLoading = true;
		error = null;

		try {
			const chain = getChainParam(connState.selectedChain) || undefined;

			// Fetch all identities
			const identities = await invoke('list_identities', { chain: chain?.toLowerCase() });

			if (!Array.isArray(identities)) {
				throw new Error('Invalid response from list_identities');
			}

			// Filter for vlotto tickets (matching parent i-address)
			const vlottoTickets: MyTicket[] = [];

			for (const identity of identities) {
				const parent = identity?.identity?.parent;
				const name = identity?.identity?.name || '';

				if (parent === vlottoParent && name.includes('_') && name.includes('of')) {
					// Extract drawing block and index from ticket name
					// Pattern: <drawingBlock>_<idx>of<planned>.<mainID>@
					// Example: 773160_6of9.vlotto@
					const match = name.match(/^(\d+)_(\d+of\d+)/);
					if (match) {
						const drawingBlock = parseInt(match[1]);
						const index = match[2];
						const identityAddress = identity?.identity?.identityaddress || '';

						vlottoTickets.push({
							name,
							identityAddress,
							index,
							drawingBlock,
							identity
						});
					}
				}
			}

			allMyTickets = vlottoTickets;

			// Split tickets into current lottery vs previous
			const currentDrawingBlock = ledgerData.lotteryParameters.drawingBlock;
			thisLotteryTickets = vlottoTickets
				.filter((t) => t.drawingBlock === currentDrawingBlock)
				.sort((a, b) => {
					// Sort by index number (extract number from "6of9" -> 6)
					const aNum = parseInt(a.index.split('of')[0]);
					const bNum = parseInt(b.index.split('of')[0]);
					return aNum - bNum;
				});

			previousLotteryTickets = vlottoTickets
				.filter((t) => t.drawingBlock !== currentDrawingBlock)
				.sort((a, b) => {
					// Sort by drawing block (most recent first), then by index
					if (a.drawingBlock !== b.drawingBlock) {
						return b.drawingBlock - a.drawingBlock;
					}
					const aNum = parseInt(a.index.split('of')[0]);
					const bNum = parseInt(b.index.split('of')[0]);
					return aNum - bNum;
				});

			isExpanded = true;
		} catch (err: any) {
			error = err?.toString() || 'Failed to load tickets';
		} finally {
			isLoading = false;
		}
	}

	function toggleExpanded() {
		if (!isExpanded && allMyTickets.length === 0) {
			loadMyTickets();
		} else {
			isExpanded = !isExpanded;
		}
	}

	function handleRefresh() {
		allMyTickets = [];
		thisLotteryTickets = [];
		previousLotteryTickets = [];
		loadMyTickets();
	}

	function handleSendToGraveyard(ticket: MyTicket) {
		selectedTicket = ticket;
		sourceOfFunds = '';
		sendError = null;
		sendSuccess = false;
		showGraveyardModal = true;
	}

	async function confirmSendToGraveyard() {
		if (!selectedTicket) return;

		isSending = true;
		sendError = null;
		sendSuccess = false;

		try {
			const chain = getChainParam(connState.selectedChain) || undefined;

			// Call Tauri command to send ticket to graveyard
			const result = await invoke('send_ticket_to_graveyard', {
				ticketIdentityAddress: selectedTicket.identityAddress,
				sourceOfFunds: sourceOfFunds.trim() || null,
				chain: chain?.toLowerCase()
			});

			sendSuccess = true;
		} catch (err: any) {
			sendError = err?.toString() || 'Failed to send ticket to graveyard';
		} finally {
			isSending = false;
		}
	}

	function closeGraveyardModal() {
		if (!isSending) {
			showGraveyardModal = false;
			selectedTicket = null;
			sourceOfFunds = '';
			sendError = null;
			sendSuccess = false;
			// Refresh tickets if we just sent one successfully
			if (sendSuccess) {
				handleRefresh();
			}
		}
	}

	// Check if phase 10 is complete
	function isPhase10Complete(): boolean {
		return ledgerData.phaseStatus?.['phase10_distribution']?.completed === true;
	}

	// Check if ticket is the top winner
	function isTopWinner(ticketName: string): boolean {
		return ledgerData.drawingResults.topWinningTicket?.name === ticketName;
	}

	function prevLotteryGoToPage(page: number) {
		if (page >= 1 && page <= prevLotteryTotalPages) {
			prevLotteryCurrentPage = page;
		}
	}

	function prevLotteryNextPage() {
		if (prevLotteryCurrentPage < prevLotteryTotalPages) {
			prevLotteryCurrentPage++;
		}
	}

	function prevLotteryPrevPage() {
		if (prevLotteryCurrentPage > 1) {
			prevLotteryCurrentPage--;
		}
	}
</script>

<div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl p-6">
	<div class="flex justify-between items-center mb-4">
		<h2 class="text-xl font-bold text-verusidx-stone-dark dark:text-white">My Tickets</h2>
		<div class="flex gap-2">
			{#if isExpanded && !isLoading}
				<button
					onclick={handleRefresh}
					class="px-4 py-2 rounded-lg transition-colors bg-verusidx-mountain-blue text-white hover:bg-verusidx-lake-blue"
				>
					Refresh
				</button>
			{/if}
			<button
				onclick={toggleExpanded}
				class="px-4 py-2 rounded-lg transition-colors
					{isLoading
					? 'bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-mountain-grey cursor-not-allowed'
					: 'bg-verusidx-turquoise-deep text-white hover:bg-verusidx-turquoise-bright'}"
				disabled={isLoading}
			>
				{#if isLoading}
					Loading...
				{:else if isExpanded}
					Collapse
				{:else}
					Load My Tickets
				{/if}
			</button>
		</div>
	</div>

	{#if isLoading}
		<div class="text-center py-8">
			<div
				class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-verusidx-turquoise-deep mb-4"
			></div>
			<p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
				Loading your tickets...
			</p>
		</div>
	{:else if error}
		<div class="text-center py-8">
			<div
				class="bg-red-100 dark:bg-red-900/20 border border-red-400 dark:border-red-500 rounded-lg p-4"
			>
				<h3 class="text-red-700 dark:text-red-400 font-semibold mb-2">Error Loading Tickets</h3>
				<p class="text-verusidx-stone-dark dark:text-verusidx-mountain-mist">{error}</p>
				<button
					onclick={handleRefresh}
					class="mt-4 px-4 py-2 bg-verusidx-turquoise-deep text-white rounded-lg hover:bg-verusidx-turquoise-bright transition-colors"
				>
					Retry
				</button>
			</div>
		</div>
	{:else if isExpanded}
		{#if allMyTickets.length === 0}
			<div class="text-center py-8">
				<p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
					No vlotto tickets found in your wallet
				</p>
			</div>
		{:else}
			<div class="space-y-4">
				<!-- This Lottery Section -->
				{#if thisLotteryTickets.length > 0}
					<div class="border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg">
						<button
							onclick={() => (thisLotteryExpanded = !thisLotteryExpanded)}
							class="w-full p-4 flex justify-between items-center hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors rounded-t-lg"
						>
							<h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">
								Current Drawing ({thisLotteryTickets.length})
							</h3>
							<span
								class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist transform transition-transform {thisLotteryExpanded
									? 'rotate-180'
									: ''}"
							>
								▼
							</span>
						</button>

						{#if thisLotteryExpanded}
							<div class="p-4 space-y-3 border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
								{#each thisLotteryTickets as ticket}
									{@const phase10Done = isPhase10Complete()}
									{@const isWinner = isTopWinner(ticket.name)}
									{@const ticketNumber = ticket.index.split('of')[0]}
									<div
										class="p-4 rounded-lg bg-verusidx-sky-soft dark:bg-verusidx-stone-medium flex items-center gap-4"
									>
										<!-- Ticket number (left) -->
										<div class="flex-shrink-0 flex items-center justify-center min-w-20">
											<p class="text-5xl font-bold text-verusidx-mountain-blue dark:text-verusidx-turquoise-deep dark:[text-shadow:_-1px_-1px_0_#b8c8d4,_1px_-1px_0_#b8c8d4,_-1px_1px_0_#b8c8d4,_1px_1px_0_#b8c8d4]">
												{ticketNumber}
											</p>
										</div>

										<!-- Ticket info (middle) -->
										<div class="flex-1 min-w-0 pl-32">
											<div class="flex items-center gap-2">
												<p class="font-medium text-verusidx-stone-dark dark:text-white truncate">
													{ticket.name}
												</p>
												{#if isWinner}
													<span
														class="px-2 py-0.5 text-xs rounded bg-verusidx-turquoise-deep text-white font-bold flex-shrink-0"
													>
														TOP WINNER
													</span>
												{/if}
											</div>
											<p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
												Ticket #{ticket.index} • Drawing Block: {ticket.drawingBlock}
											</p>
										</div>

										<!-- Action button (right) -->
										<div class="flex-shrink-0">
											{#if phase10Done}
												{#if isWinner}
													<a
														href="/marketplace"
														class="px-3 py-1 text-sm bg-yellow-600 dark:bg-yellow-500 text-white rounded hover:bg-yellow-700 dark:hover:bg-yellow-600 transition-colors font-semibold"
													>
														Claim Jackpot
													</a>
												{:else}
													<button
														onclick={() => handleSendToGraveyard(ticket)}
														class="px-3 py-1 text-sm bg-red-700 dark:bg-red-800 text-white rounded hover:bg-red-800 dark:hover:bg-red-900 transition-colors"
													>
														Discard Ticket
													</button>
												{/if}
											{/if}
										</div>
									</div>
								{/each}
							</div>
						{/if}
					</div>
				{/if}

				<!-- Previous Lottery Section -->
				{#if previousLotteryTickets.length > 0}
					<div class="border border-verusidx-mountain-mist dark:border-verusidx-stone-medium rounded-lg">
						<button
							onclick={() => (previousLotteryExpanded = !previousLotteryExpanded)}
							class="w-full p-4 flex justify-between items-center hover:bg-verusidx-sky-soft dark:hover:bg-verusidx-stone-medium transition-colors rounded-t-lg"
						>
							<h3 class="text-lg font-semibold text-verusidx-stone-dark dark:text-white">
								Previous Drawings ({previousLotteryTickets.length})
							</h3>
							<span
								class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist transform transition-transform {previousLotteryExpanded
									? 'rotate-180'
									: ''}"
							>
								▼
							</span>
						</button>

						{#if previousLotteryExpanded}
							<div class="p-4 border-t border-verusidx-mountain-mist dark:border-verusidx-stone-medium">
								<!-- Pagination Info -->
								{#if previousLotteryTickets.length > prevLotteryItemsPerPage}
									<div class="flex justify-between items-center text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-4">
										<span
											>Showing {((prevLotteryCurrentPage - 1) * prevLotteryItemsPerPage) + 1} to {Math.min(
												prevLotteryCurrentPage * prevLotteryItemsPerPage,
												previousLotteryTickets.length
											)} of {previousLotteryTickets.length} tickets</span
										>
										<span>Page {prevLotteryCurrentPage} of {prevLotteryTotalPages}</span>
									</div>
								{/if}

								<div class="space-y-3">
									{#each prevLotteryPaginated as ticket}
										{@const ticketNumber = ticket.index.split('of')[0]}
										<div
											class="p-4 rounded-lg bg-verusidx-sky-soft dark:bg-verusidx-stone-medium flex items-center gap-4"
										>
											<!-- Ticket number (left) -->
											<div class="flex-shrink-0 flex items-center justify-center min-w-20">
												<p class="text-5xl font-bold text-verusidx-mountain-blue dark:text-verusidx-turquoise-deep dark:[text-shadow:_-1px_-1px_0_#b8c8d4,_1px_-1px_0_#b8c8d4,_-1px_1px_0_#b8c8d4,_1px_1px_0_#b8c8d4]">
													{ticketNumber}
												</p>
											</div>

											<!-- Ticket info (middle) -->
											<div class="flex-1 min-w-0 pl-32">
												<p class="font-medium text-verusidx-stone-dark dark:text-white truncate">
													{ticket.name}
												</p>
												<p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
													Ticket #{ticket.index} • Drawing Block: {ticket.drawingBlock}
												</p>
											</div>

											<!-- Action button (right) -->
											<div class="flex-shrink-0">
												<button
													onclick={() => handleSendToGraveyard(ticket)}
													class="px-3 py-1 text-sm bg-red-700 dark:bg-red-800 text-white rounded hover:bg-red-800 dark:hover:bg-red-900 transition-colors"
												>
													Discard Ticket
												</button>
											</div>
										</div>
									{/each}
								</div>

								<!-- Pagination Controls -->
								{#if prevLotteryTotalPages > 1}
									<div class="flex justify-center mt-6">
										<div class="flex items-center space-x-2">
											<button
												onclick={prevLotteryPrevPage}
												disabled={prevLotteryCurrentPage === 1}
												class="px-3 py-1 text-sm bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-stone-dark dark:text-white rounded hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-dark disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
											>
												Previous
											</button>

											<!-- Page numbers -->
											<div class="flex items-center space-x-1">
												{#each Array(Math.min(5, prevLotteryTotalPages)) as _, i}
													{@const pageNum = Math.max(
														1,
														Math.min(prevLotteryTotalPages - 4, prevLotteryCurrentPage - 2)
													) + i}
													{#if pageNum <= prevLotteryTotalPages}
														<button
															onclick={() => prevLotteryGoToPage(pageNum)}
															class="px-3 py-1 text-sm rounded transition-colors {prevLotteryCurrentPage ===
															pageNum
																? 'bg-verusidx-mountain-blue text-white'
																: 'bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-stone-dark dark:text-white hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-dark'}"
														>
															{pageNum}
														</button>
													{/if}
												{/each}
											</div>

											<button
												onclick={prevLotteryNextPage}
												disabled={prevLotteryCurrentPage === prevLotteryTotalPages}
												class="px-3 py-1 text-sm bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-stone-dark dark:text-white rounded hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-dark disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
											>
												Next
											</button>
										</div>
									</div>
								{/if}
							</div>
						{/if}
					</div>
				{/if}
			</div>
		{/if}
	{:else}
		<div class="text-center py-8">
			<p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
				Click "Load My Tickets" to view your vlotto tickets
			</p>
		</div>
	{/if}
</div>

<!-- Discard Ticket Confirmation Modal -->
<Modal isOpen={showGraveyardModal} title="Discard Ticket" onclose={closeGraveyardModal}>
	{#if selectedTicket}
		<div class="space-y-4">
			{#if !sendSuccess}
				<div class="space-y-3">
					<p class="text-verusidx-stone-dark dark:text-white">
						Discard <span class="font-semibold">{selectedTicket.name}</span>
						(<span class="font-semibold">{selectedTicket.index}</span>)?
					</p>
					<div
						class="bg-red-100 dark:bg-red-900/20 border border-red-400 dark:border-red-500 rounded-lg p-3"
					>
						<p class="text-sm text-red-700 dark:text-red-400 font-semibold">
							⚠️ This action cannot be undone
						</p>
						<p class="text-xs text-red-600 dark:text-red-300 mt-1">
							The ticket will be permanently discarded and removed from your wallet.
						</p>
					</div>

					<div>
						<label
							for="sourceOfFunds"
							class="block text-sm font-medium text-verusidx-stone-dark dark:text-white mb-2"
						>
							Source of Funds (Optional)
						</label>
						<input
							id="sourceOfFunds"
							type="text"
							bind:value={sourceOfFunds}
							placeholder="Enter R-address, z-address, or ID@"
							disabled={isSending}
							class="w-full px-3 py-2 bg-white dark:bg-verusidx-stone-medium border border-verusidx-mountain-mist dark:border-verusidx-stone-dark rounded-lg text-verusidx-stone-dark dark:text-white placeholder-verusidx-mountain-grey focus:outline-none focus:ring-2 focus:ring-verusidx-mountain-blue disabled:opacity-50"
						/>
						<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
							Specify which identity/address pays the transaction fee, or leave blank for automatic
							selection
						</p>
					</div>

					{#if sendError}
						<div
							class="bg-red-100 dark:bg-red-900/20 border border-red-400 dark:border-red-500 rounded-lg p-3"
						>
							<p class="text-sm text-red-700 dark:text-red-400 font-semibold">Error</p>
							<p class="text-xs text-red-600 dark:text-red-300 mt-1">{sendError}</p>
						</div>
					{/if}
				</div>

				<div class="flex gap-3 pt-2">
					<button
						onclick={closeGraveyardModal}
						disabled={isSending}
						class="flex-1 px-4 py-2 bg-verusidx-mountain-mist dark:bg-verusidx-stone-medium text-verusidx-stone-dark dark:text-white rounded-lg hover:bg-verusidx-mountain-grey dark:hover:bg-verusidx-stone-dark transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
					>
						Cancel
					</button>
					<button
						onclick={confirmSendToGraveyard}
						disabled={isSending}
						class="flex-1 px-4 py-2 bg-red-700 dark:bg-red-800 text-white rounded-lg hover:bg-red-800 dark:hover:bg-red-900 transition-colors disabled:opacity-50 disabled:cursor-not-allowed font-semibold"
					>
						{isSending ? 'Discarding...' : 'Confirm Discard'}
					</button>
				</div>
			{:else}
				<div class="text-center py-6">
					<div class="mb-4">
						<svg
							class="mx-auto h-12 w-12 text-green-600 dark:text-green-400"
							fill="none"
							stroke="currentColor"
							viewBox="0 0 24 24"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M5 13l4 4L19 7"
							></path>
						</svg>
					</div>
					<p class="text-lg font-semibold text-green-700 dark:text-green-400 mb-2">
						Ticket Sent Successfully!
					</p>
					<p class="text-sm text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-6">
						Ticket will disappear next block
					</p>
					<button
						onclick={closeGraveyardModal}
						class="px-6 py-2 bg-verusidx-mountain-blue text-white rounded-lg hover:bg-verusidx-mountain-blue-dark transition-colors font-semibold"
					>
						Close
					</button>
				</div>
			{/if}
		</div>
	{/if}
</Modal>
