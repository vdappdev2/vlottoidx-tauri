<script lang="ts">
	import type { VlottoLedgerData } from '$lib/stores/vlottoCache';
	import { fetchUtilityIdentity, fetchUtilityCurrency } from '$lib/services/vlottoService';
	import Modal from '$lib/components/cards/Modal.svelte';
	import IdentityDetailsView from '$lib/components/IdentityDetailsView.svelte';
	import CurrencyDetailsView from '$lib/components/CurrencyDetailsView.svelte';

	interface Props {
		ledgerData: VlottoLedgerData;
	}

	let { ledgerData }: Props = $props();

	const utilities = ledgerData.utilities;
	const hasUtilities = utilities && Object.keys(utilities).length > 0;

	// Modal state
	let isViewingIdentity = $state(false);
	let isViewingCurrency = $state(false);
	let selectedUtilityData = $state<any>(null);
	let isLoadingDetails = $state(false);
	let detailsError = $state<string | null>(null);

	// Utility role descriptions (from Section 15 of vlotto third-party guide)
	const UTILITY_ROLES: Record<string, string> = {
		jackpot: 'Holds current jackpot balance',
		payout: 'Staging area for winner buyback offers',
		operations: 'Operator allocation pool',
		ledger: 'Public bulletin of drawing status',
		proofguard: 'Second-party signature attester',
		reserves: 'Treasury buffer for jackpot ceiling',
		cashier: 'Ticket sales revenue collection'
	};

	/**
	 * Get role description for a utility ID
	 */
	function getUtilityRole(utilityId: string): string {
		// Extract utility type from ID (e.g., "jackpot.vlotto@" -> "jackpot")
		const utilityType = utilityId.split('.')[0].toLowerCase();
		return UTILITY_ROLES[utilityType] || 'Utility identity';
	}

	/**
	 * View utility identity details
	 */
	async function viewUtilityIdentity(utilityId: string) {
		isViewingIdentity = true;
		isLoadingDetails = true;
		detailsError = null;
		selectedUtilityData = null;

		try {
			const identity = await fetchUtilityIdentity(utilityId);
			selectedUtilityData = identity;
		} catch (error: any) {
			detailsError = error?.message || 'Failed to load utility details';
		} finally {
			isLoadingDetails = false;
		}
	}

	/**
	 * View basket currency details
	 */
	async function viewBasketCurrency(currencyName: string) {
		isViewingCurrency = true;
		isLoadingDetails = true;
		detailsError = null;
		selectedUtilityData = null;

		try {
			const currency = await fetchUtilityCurrency(currencyName);
			selectedUtilityData = currency;
		} catch (error: any) {
			detailsError = error?.message || 'Failed to load currency details';
		} finally {
			isLoadingDetails = false;
		}
	}

	/**
	 * Close modals
	 */
	function closeDetailsModal() {
		isViewingIdentity = false;
		isViewingCurrency = false;
		selectedUtilityData = null;
		detailsError = null;
	}
</script>

<div class="bg-white dark:bg-verusidx-stone-dark rounded-lg shadow-xl p-6">
	<h2 class="text-xl font-bold text-verusidx-stone-dark dark:text-white mb-4">
		Financial Summary
	</h2>

	{#if hasUtilities}
		<div class="space-y-4">
			<!-- Total Basket Value -->
			{#if utilities.totalBasket !== undefined}
				<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-4 rounded-lg">
					<p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist text-sm mb-1">
						Total Basket Value
					</p>
					<p class="text-2xl font-bold text-verusidx-stone-dark dark:text-white">
						{utilities.totalBasket.toLocaleString(undefined, {
							minimumFractionDigits: 2,
							maximumFractionDigits: 8
						})}
					</p>
					{#if utilities.basketHeight}
						<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mt-1">
							At block {utilities.basketHeight.toLocaleString()}
						</p>
					{/if}
				</div>
			{/if}

			<!-- Basket Composition -->
			{#if utilities.basketInfo && Object.keys(utilities.basketInfo).length > 0}
				<div>
					<h3
						class="text-sm font-semibold text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-2"
					>
						Basket Composition
					</h3>
					<div class="grid grid-cols-2 gap-3">
						{#each Object.entries(utilities.basketInfo) as [currency, balance]}
							<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded-lg">
								<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-1">
									{currency}
								</p>
								<p class="font-mono text-sm text-verusidx-stone-dark dark:text-white">
									{typeof balance === 'number'
										? balance.toLocaleString(undefined, {
												minimumFractionDigits: 2,
												maximumFractionDigits: 8
											})
										: balance}
								</p>
							</div>
						{/each}
					</div>
				</div>
			{/if}

			<!-- Utility IDs -->
			{#if utilities.utilityIds && utilities.utilityIds.length > 0}
				<div>
					<h3
						class="text-sm font-semibold text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-2"
					>
						Utility IDs
					</h3>
					<div class="grid grid-cols-1 md:grid-cols-2 gap-3">
						{#each utilities.utilityIds as utilityId}
							<div
								class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-4 rounded-lg border border-verusidx-mountain-mist/20 dark:border-verusidx-stone-dark/50"
							>
								<div class="flex items-start justify-between mb-2">
									<div class="flex-1">
										<p class="font-mono text-sm font-semibold text-verusidx-stone-dark dark:text-white mb-1">
											{utilityId}
										</p>
										<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
											{getUtilityRole(utilityId)}
										</p>
									</div>
									<!-- Future: Balance slot reserved here -->
								</div>
								<button
									onclick={() => viewUtilityIdentity(utilityId)}
									class="mt-2 w-full px-3 py-1.5 text-xs bg-verusidx-turquoise-deep text-white rounded hover:bg-verusidx-turquoise-bright transition-colors"
								>
									View Details
								</button>
							</div>
						{/each}

						<!-- Basket Currency Card (if mainLotteryID exists) -->
						{#if ledgerData.lotteryParameters?.mainVerusID}
							<div
								class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-4 rounded-lg border border-verusidx-mountain-mist/20 dark:border-verusidx-stone-dark/50"
							>
								<div class="flex items-start justify-between mb-2">
									<div class="flex-1">
										<p class="font-mono text-sm font-semibold text-verusidx-stone-dark dark:text-white mb-1">
											{ledgerData.lotteryParameters.mainVerusID.replace(/@$/, '')} (currency)
										</p>
										<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
											Main currency basket (100% weighted VRSC reserves)
										</p>
									</div>
									<!-- Future: Balance slot reserved here -->
								</div>
								<button
									onclick={() => viewBasketCurrency(ledgerData.lotteryParameters.mainVerusID.replace(/@$/, ''))}
									class="mt-2 w-full px-3 py-1.5 text-xs bg-verusidx-mountain-blue text-white rounded hover:bg-verusidx-lake-blue transition-colors"
								>
									View Details
								</button>
							</div>
						{/if}
					</div>
				</div>
			{/if}

			<!-- Graveyard Info -->
			{#if utilities.graveyard?.rAddress}
				<div class="bg-verusidx-sky-soft dark:bg-verusidx-stone-medium p-3 rounded-lg">
					<p class="text-xs text-verusidx-mountain-grey dark:text-verusidx-mountain-mist mb-1">
						Graveyard Address (Unsold Tickets)
					</p>
					<p class="font-mono text-xs text-verusidx-stone-dark dark:text-white break-all">
						{utilities.graveyard.rAddress}
					</p>
				</div>
			{/if}
		</div>
	{:else}
		<div class="text-center py-8">
			<p class="text-verusidx-mountain-grey dark:text-verusidx-mountain-mist">
				No financial data available
			</p>
		</div>
	{/if}
</div>

<!-- Identity Details Modal -->
{#if isViewingIdentity}
	<Modal isOpen={isViewingIdentity} onclose={closeDetailsModal} title="Utility Identity Details">
		<IdentityDetailsView
			identityData={selectedUtilityData}
			isLoading={isLoadingDetails}
			error={detailsError}
		/>
	</Modal>
{/if}

<!-- Currency Details Modal -->
{#if isViewingCurrency}
	<Modal isOpen={isViewingCurrency} onclose={closeDetailsModal} title="Basket Currency Details">
		<CurrencyDetailsView
			currencyData={selectedUtilityData}
			isLoading={isLoadingDetails}
			error={detailsError}
		/>
	</Modal>
{/if}
