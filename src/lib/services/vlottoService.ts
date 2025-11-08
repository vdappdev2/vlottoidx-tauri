/**
 * vlotto Lottery Service
 * Data fetching, polling, and update detection for vlotto lottery
 */

import { invoke } from '@tauri-apps/api/core';
import { vlottoCache, type VlottoTicketData } from '$lib/stores/vlottoCache';
import {
	parseLedgerData,
	parseTicketMessage,
	extractContentMultimapMessage,
	generateTicketName,
	isTicketSold,
	isTicketDiscarded,
	isTicketClaimed,
	VDXF_KEYS
} from '$lib/utils/vlottoParser';
import { computeTicketScore, rankTickets } from '$lib/utils/vlottoScoring';
import { get } from 'svelte/store';
import { connectionStore, getChainParam } from '$lib/stores/connection';

const LEDGER_IDENTITY = 'ledger.vlotto@';
const POLL_INTERVAL_MS = 60000; // 60 seconds

/**
 * Fetch ledger.vlotto@ identity and parse data
 */
export async function fetchLedger(chain?: string): Promise<void> {
	// Get chain from connection store if not provided
	if (!chain) {
		const connState = get(connectionStore);
		chain = getChainParam(connState.selectedChain) || undefined;
	}
	try {
		vlottoCache.setLoading(true);
		vlottoCache.setError(null);

		// Fetch ledger identity
		const identity = await invoke('get_identity', {
			name: LEDGER_IDENTITY,
			chain: chain?.toLowerCase()
		});

		// Get current block height for cache tracking
		const blockHeight = await invoke<number>('get_block_count', {
			chain: chain?.toLowerCase()
		});

		// Parse ledger data
		const ledgerData = parseLedgerData(identity);
		if (!ledgerData) {
			throw new Error('Failed to parse ledger data');
		}

		// Extract vlotto parent i-address from ledger identity
		const vlottoParent = identity?.identity?.parent || null;

		// Update cache
		vlottoCache.setLedger(ledgerData, identity, blockHeight, vlottoParent);

		// Check if phase or drawing block changed - invalidate ticket cache if so
		const cacheState = get(vlottoCache);
		const phaseChanged = cacheState.lastPhase && cacheState.lastPhase !== ledgerData.currentPhase;
		const drawingBlockChanged = cacheState.lastDrawingBlock && cacheState.lastDrawingBlock !== ledgerData.lotteryParameters.drawingBlock;

		if (phaseChanged || drawingBlockChanged) {
			vlottoCache.invalidateTickets();
		}
	} catch (error: any) {
		vlottoCache.setError(error?.toString() || 'Failed to fetch ledger data');
		throw error;
	} finally {
		vlottoCache.setLoading(false);
	}
}

/**
 * Fetch single ticket data with contentmultimap
 * @param ticketName Full ticket identity name
 * @param chain Network chain (VRSCTEST or VRSC)
 * @param vdxfKey Optional VDXF key filter
 * @param mainVerusID Optional main ID for fallback ticket naming
 * @param index Optional ticket index for fallback naming
 * @param planned Optional total planned tickets for fallback naming
 * @param drawingBlock Optional drawing block for fallback naming
 */
export async function fetchTicket(
	ticketName: string,
	chain?: string,
	vdxfKey?: string,
	mainVerusID?: string,
	index?: number,
	planned?: number,
	drawingBlock?: number
): Promise<any> {
	// Get chain from connection store if not provided
	if (!chain) {
		const connState = get(connectionStore);
		chain = getChainParam(connState.selectedChain) || undefined;
	}
	// Try primary ticket name
	try {
		// Fetch ticket content (full history)
		const ticketContent = await invoke('get_identity_content', {
			name: ticketName,
			heightstart: null,
			heightend: null,
			txproofs: false,
			vdxfkey: vdxfKey || VDXF_KEYS.TICKET_FINALIZED_DATA,
			chain: chain?.toLowerCase()
		});

		// Extract and parse message
		const messageStr = extractContentMultimapMessage(ticketContent);
		if (!messageStr) {
			throw new Error(`No contentmultimap message found for ${ticketName}`);
		}

		const ticketData = parseTicketMessage(messageStr);

		// Also fetch current identity state to check sold status
		const ticketIdentity = await invoke('get_identity', {
			name: ticketName,
			chain: chain?.toLowerCase()
		});

		return { ticketContent, ticketData, ticketIdentity };
	} catch (error) {
		// Fallback: Try trimming digits from mainID
		if (mainVerusID && index !== undefined && planned !== undefined && drawingBlock !== undefined) {
			// Trim trailing @ and remove trailing digits
			let trimmedMain = mainVerusID.replace(/@$/, '');
			const fallbackAttempts = 3;

			for (let attempt = 1; attempt <= fallbackAttempts; attempt++) {
				// Remove last character (assumed to be a digit)
				trimmedMain = trimmedMain.slice(0, -1);
				const fallbackName = generateTicketName(drawingBlock, index, planned, trimmedMain + '@');

				try {
					const ticketContent = await invoke('get_identity_content', {
						name: fallbackName,
						heightstart: null,
						heightend: null,
						txproofs: false,
						vdxfkey: vdxfKey || VDXF_KEYS.TICKET_FINALIZED_DATA,
						chain: chain?.toLowerCase()
					});

					const messageStr = extractContentMultimapMessage(ticketContent);
					if (!messageStr) continue;

					const ticketData = parseTicketMessage(messageStr);
					const ticketIdentity = await invoke('get_identity', {
						name: fallbackName,
						chain: chain?.toLowerCase()
					});

					return { ticketContent, ticketData, ticketIdentity };
				} catch (fallbackError) {
					// Continue to next fallback attempt
					continue;
				}
			}
		}

		return null;
	}
}

/**
 * Enumerate and fetch all tickets for the current lottery
 * This is expensive - only call when needed (e.g., user expands ticket list)
 */
export async function enumerateAndFetchTickets(
	chain?: string,
	progressCallback?: (current: number, total: number) => void
): Promise<VlottoTicketData[]> {
	// Get chain from connection store if not provided
	if (!chain) {
		const connState = get(connectionStore);
		chain = getChainParam(connState.selectedChain) || undefined;
	}
	const cacheState = get(vlottoCache);
	const ledgerData = cacheState.ledger.data;

	if (!ledgerData) {
		throw new Error('Ledger data not loaded');
	}

	const { lotteryParameters, ticketSummary, drawingResults } = ledgerData;
	const { mainVerusID, drawingBlock, rAddressForTickets, claimedTicketsAddress } = lotteryParameters;
	const { planned } = ticketSummary;
	const { drawingHash } = drawingResults;

	// Drawing hash is optional - tickets can be enumerated before drawing
	const hasDrawingHash = drawingHash && drawingHash.trim() !== '';

	const tickets: VlottoTicketData[] = [];

	// Enumerate tickets
	for (let i = 1; i <= planned; i++) {
		if (progressCallback) {
			progressCallback(i, planned);
		}

		const ticketName = generateTicketName(drawingBlock, i, planned, mainVerusID);

		try {
			const result = await fetchTicket(ticketName, chain, undefined, mainVerusID, i, planned, drawingBlock);
			if (!result) continue;

			const { ticketData, ticketIdentity } = result;

			// Determine network type for graveyard address lookup
			const network = (chain && chain.toLowerCase() === 'vrsctest') ? 'VRSCTEST' : 'VRSC';

			// Check ticket status
			const claimed = isTicketClaimed(ticketIdentity, claimedTicketsAddress);
			const discarded = isTicketDiscarded(ticketIdentity, network);
			const sold = isTicketSold(ticketIdentity, rAddressForTickets, network);

			// Compute score (only if drawing hash available)
			let matches = 0;
			let score = 0;
			if (hasDrawingHash) {
				const scoreResult = computeTicketScore(ticketData.playingNumber, drawingHash);
				matches = scoreResult.matches;
				score = scoreResult.score;
			}

			// Create ticket data object
			const ticket: VlottoTicketData = {
				name: ticketName,
				playingNumber: ticketData.playingNumber,
				registrationTxid: ticketData.registrationTxid,
				sold,
				discarded,
				claimed,
				matches,
				score,
				index: i,
				ticketValidation: ticketData.ticketValidation,
				proofguardAcknowledgement: ticketData.proofguardAcknowledgement
			};

			tickets.push(ticket);
			vlottoCache.setTicket(ticket); // Cache individual ticket
		} catch (error) {
			// Ticket fetch failed, continue to next ticket
		}
	}

	// Rank tickets and update cache
	const rankedTickets = rankTickets(tickets);
	vlottoCache.setTicketScores(rankedTickets);

	return rankedTickets;
}

/**
 * Check for updates (poll block height)
 * Call this periodically to detect new data
 */
export async function checkForUpdates(chain?: string): Promise<boolean> {
	// Get chain from connection store if not provided
	if (!chain) {
		const connState = get(connectionStore);
		chain = getChainParam(connState.selectedChain) || undefined;
	}
	try {
		const cacheState = get(vlottoCache);
		const lastBlockHeight = cacheState.ledger.blockHeight;

		// Get current block height
		const currentBlockHeight = await invoke<number>('get_block_count', {
			chain: chain?.toLowerCase()
		});

		// If block height increased, refresh ledger
		if (currentBlockHeight > lastBlockHeight) {
			await fetchLedger(chain);
			return true;
		}

		return false;
	} catch (error) {
		return false;
	}
}

/**
 * Start auto-refresh polling
 * Returns cleanup function to stop polling
 */
export function startAutoRefresh(
	chain?: string,
	intervalMs: number = POLL_INTERVAL_MS
): () => void {
	const intervalId = setInterval(async () => {
		await checkForUpdates(chain);
	}, intervalMs);

	return () => clearInterval(intervalId);
}

/**
 * Manual refresh - refetch everything
 */
export async function manualRefresh(chain?: string): Promise<void> {
	await fetchLedger(chain);
	// Note: tickets are not automatically refetched - user must expand section
}

/**
 * Fetch utility identity details
 * Used for displaying utility ID information (jackpot, payout, operations, etc.)
 */
export async function fetchUtilityIdentity(utilityName: string, chain?: string): Promise<any> {
	// Get chain from connection store if not provided
	if (!chain) {
		const connState = get(connectionStore);
		chain = getChainParam(connState.selectedChain) || undefined;
	}

	try {
		const identity = await invoke('get_identity', {
			name: utilityName,
			chain: chain?.toLowerCase()
		});

		return identity;
	} catch (error: any) {
		throw new Error(`Failed to fetch utility identity ${utilityName}: ${error?.toString() || 'Unknown error'}`);
	}
}

/**
 * Fetch utility currency details
 * Used for displaying basket currency information
 */
export async function fetchUtilityCurrency(currencyName: string, chain?: string): Promise<any> {
	// Get chain from connection store if not provided
	if (!chain) {
		const connState = get(connectionStore);
		chain = getChainParam(connState.selectedChain) || undefined;
	}

	try {
		const currency = await invoke('get_currency', {
			currencyName: currencyName,
			height: null,
			chain: chain?.toLowerCase()
		});

		return currency;
	} catch (error: any) {
		throw new Error(`Failed to fetch utility currency ${currencyName}: ${error?.toString() || 'Unknown error'}`);
	}
}
