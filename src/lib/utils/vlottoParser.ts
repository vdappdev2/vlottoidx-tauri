/**
 * vlotto ContentMultimap Parser
 * Utilities for parsing contentmultimap data from vlotto identities
 */

import type { VlottoLedgerData } from '$lib/stores/vlottoCache';

// VDXF Keys
export const VDXF_KEYS = {
	LEDGER_DATA: 'iFVPmjN213NmfaiBhAkxAJWWGtcDEoXJcU', // vlotto.ledger.data
	TICKET_FINALIZED_DATA: 'iMzWvy5j4ciiMSBsEEVzfy66awLQ85b4GN', // vlotto.ticket.finalizeddata
	DATA_DESCRIPTOR: 'i4GC1YGEVD21afWudGoFJVdnfjJ5XWnCQv' // data.type.object.datadescriptor
};

// Graveyard addresses for unsold tickets
export const GRAVEYARD_ADDRESSES = {
	VRSCTEST: 'RMzd5vMptsxxz1tWH2FeSdUgRSNgS4G52w',
	VRSC: 'RAXCjm9Z4RJWEmsNgo83B8JevTcJRt6Tj5'
};

/**
 * Get graveyard address for the specified chain
 */
export function getGraveyardAddress(chain: string | undefined): string {
	const network = (!chain || chain === 'vrsctest') ? 'VRSCTEST' : 'VRSC';
	return GRAVEYARD_ADDRESSES[network];
}

/**
 * Extract message from contentmultimap latest entry
 * Pattern: contentmultimap → entries[] → value[0] → entries[] → value.objectdata.message
 */
export function extractContentMultimapMessage(identity: any): string | null {
	try {
		const contentmultimap = identity?.identity?.contentmultimap;
		if (!contentmultimap) return null;

		// Get all entries
		const entries = Object.entries(contentmultimap);
		if (entries.length === 0) return null;

		// Get the latest entry (first one)
		const [_, valueArray] = entries[0] as [string, any[]];
		if (!Array.isArray(valueArray) || valueArray.length === 0) return null;

		// Get first item in array
		const latestEntry = valueArray[0];
		if (!latestEntry) return null;

		// Navigate through nested structure
		const nestedEntries = Object.entries(latestEntry);
		if (nestedEntries.length === 0) return null;

		const [__, nestedValue] = nestedEntries[0] as [string, any];

		// Extract message
		const message = nestedValue?.objectdata?.message;
		if (typeof message !== 'string') return null;

		return message;
	} catch (error) {
		return null;
	}
}

/**
 * Parse ledger message from JSON string
 * Handles both snake_case and PascalCase field names
 */
export function parseLedgerMessage(messageStr: string): VlottoLedgerData | null {
	try {
		const data = JSON.parse(messageStr);

		// Helper to get field with fallback for different casings
		const getField = (obj: any, ...names: string[]): any => {
			for (const name of names) {
				if (obj[name] !== undefined) return obj[name];
			}
			return undefined;
		};

		// Extract common objects
		const lotteryParams = data.lotteryParameters || data.LotteryParameters || data.lottery_parameters || {};
		const ticketSum = data.ticketSummary || data.TicketSummary || data.ticket_summary || {};
		const drawingRes = data.drawingResults || data.DrawingResults || data.drawing_results || {};
		const timelockStat = data.timelockStatus || data.TimelockStatus || data.timelock_status || {};
		const marketStat = data.marketplaceStatus || data.MarketplaceStatus || data.marketplace_status || {};
		const payoutSum = data.payoutSummary || data.PayoutSummary || data.payout_summary || {};
		const distSum = data.distributionSummary || data.DistributionSummary || data.distribution_summary || {};
		const opMetrics = data.operationalMetrics || data.OperationalMetrics || data.operational_metrics || {};
		const secMetrics = data.securityMetrics || data.SecurityMetrics || data.security_metrics || {};

		return {
			ledgerVersion: getField(data, 'ledgerVersion', 'LedgerVersion', 'ledger_version'),
			lastUpdated: getField(data, 'lastUpdated', 'LastUpdated', 'last_updated'),
			currentPhase: getField(data, 'currentPhase', 'CurrentPhase', 'current_phase') || 'Unknown',
			phaseStatus: getField(data, 'phaseStatus', 'PhaseStatus', 'phase_status') || {},
			drawingId: getField(data, 'drawingId', 'DrawingId', 'drawing_id'),
			drawingResults: {
				drawingHash: getField(drawingRes, 'drawingHash', 'DrawingHash', 'drawing_hash') || '',
				drawingTimestamp: getField(drawingRes, 'drawingTimestamp', 'DrawingTimestamp', 'drawing_timestamp'),
				winnerStatus: getField(drawingRes, 'winnerStatus', 'WinnerStatus', 'winner_status'),
				verificationStatus: getField(drawingRes, 'verificationStatus', 'VerificationStatus', 'verification_status'),
				drawingMethod: getField(drawingRes, 'drawingMethod', 'DrawingMethod', 'drawing_method'),
				topTicketAuthentic: getField(drawingRes, 'topTicketAuthentic', 'TopTicketAuthentic', 'top_ticket_authentic'),
				topWinningTicket: (() => {
					const ticketName = getField(drawingRes, 'topWinningTicket', 'TopWinningTicket', 'top_winning_ticket');
					const matches = getField(drawingRes, 'topTicketMatches', 'TopTicketMatches', 'top_ticket_matches');
					const score = getField(drawingRes, 'topTicketScore', 'TopTicketScore', 'top_ticket_score');

					if (ticketName) {
						// Parse index from ticket name pattern: <drawingBlock>_<idx>of<planned>
						// Example: "773160_6of9" or "773160_6of9.vlotto@" -> "6of9"
						let index = null;
						const indexMatch = ticketName.match(/\d+_(\d+of\d+)/);
						if (indexMatch) {
							index = indexMatch[1];
						}

						return {
							name: ticketName,
							matches: matches,
							score: score,
							index: index
						};
					}
					return null;
				})()
			},
			lotteryParameters: {
				mainVerusID: getField(lotteryParams, 'mainVerusID', 'MainVerusID', 'main_verus_id', 'mainLotteryID', 'MainLotteryID') || '',
				drawingBlock: getField(lotteryParams, 'drawingBlock', 'DrawingBlock', 'drawing_block') || 0,
				startBlock: getField(lotteryParams, 'startBlock', 'StartBlock', 'start_block'),
				targetDrawingBlock: getField(lotteryParams, 'targetDrawingBlock', 'TargetDrawingBlock', 'target_drawing_block'),
				requiredMatches: getField(lotteryParams, 'requiredMatches', 'RequiredMatches', 'required_matches') || 0,
				rAddressForTickets: getField(lotteryParams, 'rAddressForTickets', 'RAddressForTickets', 'r_address_for_tickets') || '',
				ticketPrice: getField(lotteryParams, 'ticketPrice', 'TicketPrice', 'ticket_price'),
				ticketMultiplier: getField(lotteryParams, 'ticketMultiplier', 'TicketMultiplier', 'ticket_multiplier'),
				jackpotMinimum: getField(lotteryParams, 'jackpotMinimum', 'JackpotMinimum', 'jackpot_minimum'),
				jackpotCeilingCap: getField(lotteryParams, 'jackpotCeilingCap', 'JackpotCeilingCap', 'jackpot_ceiling_cap'),
				gracePeriod: getField(lotteryParams, 'gracePeriod', 'GracePeriod', 'grace_period'),
				confirmations: getField(lotteryParams, 'confirmations', 'Confirmations'),
				payoutOfferExpiry: getField(lotteryParams, 'payoutOfferExpiry', 'PayoutOfferExpiry', 'payout_offer_expiry'),
				claimedTicketsAddress: getField(lotteryParams, 'claimedTicketsAddress', 'ClaimedTicketsAddress', 'claimed_tickets_address'),
				offerExpiryOffset: getField(lotteryParams, 'offerExpiryOffset', 'OfferExpiryOffset', 'offer_expiry_offset'),
				nextJackpotPercent: getField(lotteryParams, 'nextJackpotPercent', 'NextJackpotPercent', 'next_jackpot_percent'),
				operationsPercent: getField(lotteryParams, 'operationsPercent', 'OperationsPercent', 'operations_percent')
			},
			ticketSummary: {
				planned: getField(ticketSum, 'planned', 'Planned') || 0,
				generated: getField(ticketSum, 'generated', 'Generated'),
				registered: getField(ticketSum, 'registered', 'Registered'),
				dataUpdated: getField(ticketSum, 'dataUpdated', 'DataUpdated', 'data_updated'),
				dataFailed: getField(ticketSum, 'dataFailed', 'DataFailed', 'data_failed'),
				onMarketplace: getField(ticketSum, 'onMarketplace', 'OnMarketplace', 'on_marketplace'),
				sold: getField(ticketSum, 'sold', 'Sold'),
				verified: getField(ticketSum, 'verified', 'Verified'),
				verificationResults: getField(ticketSum, 'verificationResults', 'VerificationResults', 'verification_results')
			},
			timelockStatus: {
				applied: getField(timelockStat, 'applied', 'Applied'),
				lockHeight: getField(timelockStat, 'lockHeight', 'LockHeight', 'lock_height'),
				unlockHeight: getField(timelockStat, 'unlockHeight', 'UnlockHeight', 'unlock_height'),
				currentlyLocked: getField(timelockStat, 'currentlyLocked', 'CurrentlyLocked', 'currently_locked')
			},
			marketplaceStatus: {
				ticketsListed: getField(marketStat, 'ticketsListed', 'TicketsListed', 'tickets_listed')
			},
			payoutSummary: {
				winnerWasSold: getField(payoutSum, 'winnerWasSold', 'WinnerWasSold', 'winner_was_sold'),
				payoutRequired: getField(payoutSum, 'payoutRequired', 'PayoutRequired', 'payout_required'),
				buybackTxID: getField(payoutSum, 'buybackTxID', 'BuybackTxID', 'buyback_txid')
			},
			distributionSummary: {
				totalDistributed: getField(distSum, 'totalDistributed', 'TotalDistributed', 'total_distributed'),
				distributionTxID: getField(distSum, 'distributionTxID', 'DistributionTxID', 'distribution_txid')
			},
			operationalMetrics: {
				phaseProcessingTimes: getField(opMetrics, 'phaseProcessingTimes', 'PhaseProcessingTimes', 'phase_processing_times'),
				retryOperations: getField(opMetrics, 'retryOperations', 'RetryOperations', 'retry_operations'),
				utxoOperations: getField(opMetrics, 'utxoOperations', 'UTXOOperations', 'utxo_operations')
			},
			securityMetrics: {
				fraudDetectionChecks: getField(secMetrics, 'fraudDetectionChecks', 'FraudDetectionChecks', 'fraud_detection_checks'),
				timelockStatus: getField(secMetrics, 'timelockStatus', 'TimelockStatus', 'timelock_status')
			},
			utilities: data.utilities || data.Utilities,
			financialSummary: getField(data, 'financialSummary', 'FinancialSummary', 'financial_summary')
		};
	} catch (error) {
		return null;
	}
}

/**
 * Parse complete ledger data from identity response
 */
export function parseLedgerData(identity: any): VlottoLedgerData | null {
	const messageStr = extractContentMultimapMessage(identity);
	if (!messageStr) return null;

	return parseLedgerMessage(messageStr);
}

/**
 * Parse ticket message from contentmultimap
 * Returns ticket-specific data including playing number and signatures
 */
export function parseTicketMessage(messageStr: string): any {
	try {
		const data = JSON.parse(messageStr);

		// Helper to get field with fallback for different casings
		const getField = (obj: any, ...names: string[]): any => {
			for (const name of names) {
				if (obj[name] !== undefined) return obj[name];
			}
			return undefined;
		};

		return {
			playingNumber: getField(data, 'playing_number', 'PlayingNumber', 'playingnumber') || '',
			registrationTxid:
				getField(data, 'registration_txid', 'RegistrationTxID', 'registrationTxid') || '',
			ticketValidation: {
				signedByTicketSignature: getField(
					data.ticket_validation || data.TicketValidation || {},
					'signed_by_ticket_signature',
					'SignedByTicketSignature'
				),
				signedByTicketHash: getField(
					data.ticket_validation || data.TicketValidation || {},
					'signed_by_ticket_hash',
					'SignedByTicketHash'
				)
			},
			proofguardAcknowledgement: {
				signedByProofguardSignature: getField(
					data.proofguard_acknowledgement || data.ProofguardAcknowledgement || {},
					'signed_by_proofguard_signature',
					'SignedByProofguardSignature'
				),
				signedByProofguardHash: getField(
					data.proofguard_acknowledgement || data.ProofguardAcknowledgement || {},
					'signed_by_proofguard_hash',
					'SignedByProofguardHash'
				)
			}
		};
	} catch (error) {
		return null;
	}
}

/**
 * Generate deterministic ticket name
 * Pattern: <drawingBlock>_<idx>of<planned>.<mainID-trimmed>@
 */
export function generateTicketName(
	drawingBlock: number,
	index: number,
	planned: number,
	mainVerusID: string
): string {
	// Trim trailing @ from mainVerusID
	const trimmedMain = mainVerusID.replace(/@$/, '');
	return `${drawingBlock}_${index}of${planned}.${trimmedMain}@`;
}

/**
 * Check if ticket is discarded (sent to graveyard)
 * Ticket is discarded if current primary address equals graveyard address
 */
export function isTicketDiscarded(
	ticketIdentity: any,
	network: 'VRSCTEST' | 'VRSC' = 'VRSCTEST'
): boolean {
	const primaryAddress = ticketIdentity?.identity?.primaryaddresses?.[0];
	if (!primaryAddress) return false;

	const graveyardAddress = GRAVEYARD_ADDRESSES[network];
	return primaryAddress === graveyardAddress;
}

/**
 * Check if ticket is claimed (winning ticket sent to claim address)
 * Ticket is claimed if current primary address equals claimedTicketsAddress
 */
export function isTicketClaimed(
	ticketIdentity: any,
	claimedTicketsAddress?: string
): boolean {
	if (!claimedTicketsAddress) return false;

	const primaryAddress = ticketIdentity?.identity?.primaryaddresses?.[0];
	if (!primaryAddress) return false;

	return primaryAddress === claimedTicketsAddress;
}

/**
 * Check if ticket is sold
 * Ticket is sold if current primary address differs from rAddressForTickets
 */
export function isTicketSold(
	ticketIdentity: any,
	rAddressForTickets: string,
	network: 'VRSCTEST' | 'VRSC' = 'VRSCTEST'
): boolean {
	const primaryAddress = ticketIdentity?.identity?.primaryaddresses?.[0];
	if (!primaryAddress) return false;

	const graveyardAddress = GRAVEYARD_ADDRESSES[network];

	// If address is graveyard, ticket is unsold
	if (primaryAddress === graveyardAddress) return false;

	// If address differs from rAddressForTickets, ticket is sold
	return primaryAddress !== rAddressForTickets;
}

/**
 * Format UTC ISO timestamp as relative time
 * @param isoTimestamp ISO 8601 timestamp (e.g., "2025-10-30T12:23:34Z")
 * @returns Relative time string (e.g., "2 minutes ago", "3 hours ago", "5 days ago")
 */
export function formatRelativeTime(isoTimestamp: string): string {
	const date = new Date(isoTimestamp);
	const now = new Date();
	const diffMs = now.getTime() - date.getTime();
	const diffSec = Math.floor(diffMs / 1000);
	const diffMin = Math.floor(diffSec / 60);
	const diffHour = Math.floor(diffMin / 60);
	const diffDay = Math.floor(diffHour / 24);

	if (diffSec < 60) return 'just now';
	if (diffMin < 60) return `${diffMin} minute${diffMin > 1 ? 's' : ''} ago`;
	if (diffHour < 24) return `${diffHour} hour${diffHour > 1 ? 's' : ''} ago`;
	if (diffDay < 7) return `${diffDay} day${diffDay > 1 ? 's' : ''} ago`;

	// Fallback to localized date for > 7 days
	return date.toLocaleDateString();
}
