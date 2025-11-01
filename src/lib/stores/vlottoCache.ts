/**
 * vlotto Lottery Data Cache
 * In-memory cache for vlotto ledger and ticket data
 * Cache is cleared when app closes (no persistence)
 */

import { writable } from 'svelte/store';

export interface VlottoLedgerData {
	ledgerVersion?: string;
	lastUpdated?: string;
	currentPhase: string;
	phaseStatus: Record<string, any>;
	drawingId?: string;
	drawingResults: {
		drawingHash: string;
		drawingTimestamp?: string;
		winnerStatus?: string;
		verificationStatus?: string;
		drawingMethod?: string;
		topTicketAuthentic?: boolean;
		topWinningTicket?: any;
	};
	lotteryParameters: {
		mainVerusID: string;
		drawingBlock: number;
		startBlock?: number;
		targetDrawingBlock?: number;
		requiredMatches: number;
		rAddressForTickets: string;
		ticketPrice?: number;
		ticketMultiplier?: number;
		jackpotMinimum?: number;
		jackpotCeilingCap?: number;
		gracePeriod?: number;
		confirmations?: number;
		payoutOfferExpiry?: number;
		claimedTicketsAddress?: string;
		offerExpiryOffset?: number;
		nextJackpotPercent?: number;
		operationsPercent?: number;
	};
	ticketSummary: {
		planned: number;
		generated?: number;
		registered?: number;
		dataUpdated?: number;
		dataFailed?: number;
		onMarketplace?: number;
		sold?: number;
		verified?: number;
		verificationResults?: {
			totalProcessed?: number;
			passedVerification?: number;
			failedVerification?: number;
			fraudulentTickets?: number;
		};
	};
	timelockStatus?: {
		applied?: boolean;
		lockHeight?: number;
		unlockHeight?: number;
		currentlyLocked?: boolean;
	};
	marketplaceStatus?: {
		ticketsListed?: number;
	};
	payoutSummary?: {
		winnerWasSold?: boolean;
		payoutRequired?: boolean;
		buybackTxID?: string;
	};
	distributionSummary?: {
		totalDistributed?: number;
		distributionTxID?: string;
	};
	operationalMetrics?: {
		phaseProcessingTimes?: Record<string, string>;
		retryOperations?: {
			totalRetries?: number;
			phaseRetries?: Record<string, number>;
		};
		utxoOperations?: {
			utxosCreated?: number;
			utxosConsumed?: number;
			utxoBalance?: number;
		};
	};
	securityMetrics?: {
		fraudDetectionChecks?: number;
		timelockStatus?: any;
	};
	utilities?: {
		utilityIds: string[];
		basketInfo?: Record<string, number>;
		totalBasket?: number;
		basketHeight?: number;
		graveyard?: {
			rAddress: string;
		};
	};
	financialSummary?: {
		jackpotStart?: number;
		jackpotCurrent?: number;
	};
}

export interface VlottoTicketData {
	name: string;
	playingNumber: string;
	registrationTxid: string;
	sold: boolean;
	discarded: boolean;
	claimed: boolean;
	matches: number;
	score: number;
	index: number;
	ticketValidation?: {
		signedByTicketSignature: string;
		signedByTicketHash: string;
	};
	proofguardAcknowledgement?: {
		signedByProofguardSignature: string;
		signedByProofguardHash: string;
	};
}

export interface VlottoCacheState {
	ledger: {
		data: VlottoLedgerData | null;
		rawIdentity: any;
		lastFetched: number;
		blockHeight: number;
		vlottoParent: string | null;
	};
	tickets: Map<string, VlottoTicketData>;
	ticketScores: VlottoTicketData[];
	lastPhase: string | null;
	lastDrawingBlock: number | null;
	isLoading: boolean;
	error: string | null;
}

function createVlottoCache() {
	const { subscribe, set, update } = writable<VlottoCacheState>({
		ledger: {
			data: null,
			rawIdentity: null,
			lastFetched: 0,
			blockHeight: 0,
			vlottoParent: null
		},
		tickets: new Map(),
		ticketScores: [],
		lastPhase: null,
		lastDrawingBlock: null,
		isLoading: false,
		error: null
	});

	return {
		subscribe,
		set,
		update,

		// Update ledger data
		setLedger: (data: VlottoLedgerData, rawIdentity: any, blockHeight: number, vlottoParent: string | null) =>
			update((state) => ({
				...state,
				ledger: {
					data,
					rawIdentity,
					lastFetched: Date.now(),
					blockHeight,
					vlottoParent
				},
				lastPhase: data.currentPhase,
				lastDrawingBlock: data.lotteryParameters.drawingBlock,
				error: null
			})),

		// Add or update ticket
		setTicket: (ticket: VlottoTicketData) =>
			update((state) => {
				const tickets = new Map(state.tickets);
				tickets.set(ticket.name, ticket);
				return { ...state, tickets };
			}),

		// Set all tickets at once
		setTickets: (tickets: VlottoTicketData[]) =>
			update((state) => {
				const ticketMap = new Map<string, VlottoTicketData>();
				tickets.forEach((t) => ticketMap.set(t.name, t));
				return { ...state, tickets: ticketMap };
			}),

		// Update ticket scores (sorted)
		setTicketScores: (scores: VlottoTicketData[]) =>
			update((state) => ({
				...state,
				ticketScores: scores
			})),

		// Invalidate ticket cache (on phase change)
		invalidateTickets: () =>
			update((state) => ({
				...state,
				tickets: new Map(),
				ticketScores: []
			})),

		// Set loading state
		setLoading: (isLoading: boolean) =>
			update((state) => ({
				...state,
				isLoading
			})),

		// Set error
		setError: (error: string | null) =>
			update((state) => ({
				...state,
				error
			})),

		// Clear all cache
		clear: () =>
			set({
				ledger: {
					data: null,
					rawIdentity: null,
					lastFetched: 0,
					blockHeight: 0,
					vlottoParent: null
				},
				tickets: new Map(),
				ticketScores: [],
				lastPhase: null,
				lastDrawingBlock: null,
				isLoading: false,
				error: null
			})
	};
}

export const vlottoCache = createVlottoCache();
