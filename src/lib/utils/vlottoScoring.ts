/**
 * vlotto Ticket Scoring Algorithm
 * Implements the scoring rules from the vlotto third-party integration guide
 */

import type { VlottoTicketData } from '$lib/stores/vlottoCache';

/**
 * Find the first non-zero nibble index in a hex string
 * Leading zeros are ignored per vlotto scoring rules
 */
export function findFirstNonZeroIndex(hexString: string): number {
	for (let i = 0; i < hexString.length; i++) {
		if (hexString[i] !== '0') {
			return i;
		}
	}
	return hexString.length; // All zeros
}

/**
 * Convert hex character to integer value
 */
export function hexToInt(hexChar: string): number {
	const value = parseInt(hexChar, 16);
	return isNaN(value) ? 0 : value;
}

/**
 * Compute matches and score for a ticket against drawing hash
 *
 * Scoring Rules:
 * 1. Ignore leading zeros in drawing hash
 * 2. For each position after first non-zero:
 *    - If ticket nibble === drawing nibble: increment matches, add hex value to score
 * 3. Rank by: matches desc, score desc, index asc
 *
 * @param playingNumber Ticket's 64-hex playing number
 * @param drawingHash Drawing's 64-hex hash
 * @returns Object with matches and score
 */
export function computeTicketScore(playingNumber: string, drawingHash: string): {
	matches: number;
	score: number;
} {
	let matches = 0;
	let score = 0;

	// Find first non-zero nibble in drawing hash
	const startIndex = findFirstNonZeroIndex(drawingHash);

	// Compare from startIndex to end of hash
	for (let i = startIndex; i < drawingHash.length && i < playingNumber.length; i++) {
		const drawingNibble = drawingHash[i];
		const ticketNibble = playingNumber[i];

		if (drawingNibble === ticketNibble) {
			matches++;
			score += hexToInt(drawingNibble);
		}
	}

	return { matches, score };
}

/**
 * Rank tickets by vlotto rules:
 * 1. matches (descending)
 * 2. score (descending)
 * 3. index (ascending - lower ticket numbers win ties)
 */
export function rankTickets(tickets: VlottoTicketData[]): VlottoTicketData[] {
	return [...tickets].sort((a, b) => {
		// First: matches (higher is better)
		if (b.matches !== a.matches) {
			return b.matches - a.matches;
		}

		// Second: score (higher is better)
		if (b.score !== a.score) {
			return b.score - a.score;
		}

		// Third: index (lower is better - tie-breaker)
		return a.index - b.index;
	});
}

/**
 * Filter tickets that meet winner requirements
 * @param tickets Ranked tickets
 * @param requiredMatches Minimum matches required to win
 * @param soldOnly Only consider sold tickets
 */
export function filterWinningTickets(
	tickets: VlottoTicketData[],
	requiredMatches: number,
	soldOnly: boolean = true
): VlottoTicketData[] {
	return tickets.filter((ticket) => {
		// Must meet minimum matches
		if (ticket.matches < requiredMatches) return false;

		// If soldOnly, must be sold
		if (soldOnly && !ticket.sold) return false;

		return true;
	});
}

/**
 * Get top winner from ranked tickets
 * Returns null if no ticket meets requirements
 */
export function getTopWinner(
	tickets: VlottoTicketData[],
	requiredMatches: number,
	soldOnly: boolean = true
): VlottoTicketData | null {
	const ranked = rankTickets(tickets);
	const winners = filterWinningTickets(ranked, requiredMatches, soldOnly);
	return winners.length > 0 ? winners[0] : null;
}

/**
 * Generate scoring summary statistics
 */
export function generateScoringSummary(tickets: VlottoTicketData[], requiredMatches: number): {
	totalTickets: number;
	soldTickets: number;
	unsoldTickets: number;
	highestMatches: number;
	highestScore: number;
	qualifiedWinners: number;
	soldQualifiedWinners: number;
} {
	const soldTickets = tickets.filter((t) => t.sold);
	const unsoldTickets = tickets.filter((t) => !t.sold);

	const highestMatches = Math.max(...tickets.map((t) => t.matches), 0);
	const highestScore = Math.max(...tickets.map((t) => t.score), 0);

	const qualifiedWinners = tickets.filter((t) => t.matches >= requiredMatches).length;
	const soldQualifiedWinners = soldTickets.filter((t) => t.matches >= requiredMatches).length;

	return {
		totalTickets: tickets.length,
		soldTickets: soldTickets.length,
		unsoldTickets: unsoldTickets.length,
		highestMatches,
		highestScore,
		qualifiedWinners,
		soldQualifiedWinners
	};
}

/**
 * Format hex string for display (split into groups of 8)
 */
export function formatHexForDisplay(hexString: string, groupSize: number = 8): string {
	const groups = [];
	for (let i = 0; i < hexString.length; i += groupSize) {
		groups.push(hexString.slice(i, i + groupSize));
	}
	return groups.join(' ');
}

/**
 * Highlight matching nibbles between ticket and drawing hash
 * Returns array of characters with match flag
 */
export function highlightMatches(
	playingNumber: string,
	drawingHash: string
): Array<{ char: string; matches: boolean; isLeadingZero: boolean }> {
	const startIndex = findFirstNonZeroIndex(drawingHash);
	const result = [];

	for (let i = 0; i < drawingHash.length; i++) {
		const isLeadingZero = i < startIndex;
		const matches = !isLeadingZero && i < playingNumber.length && playingNumber[i] === drawingHash[i];

		result.push({
			char: i < playingNumber.length ? playingNumber[i] : '?',
			matches,
			isLeadingZero
		});
	}

	return result;
}
