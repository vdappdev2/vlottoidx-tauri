/**
 * vlotto Ticket Verification
 * Implements 4-step cryptographic verification per vlotto third-party integration guide
 */

import { invoke } from '@tauri-apps/api/core';
import type { VlottoTicketData } from '$lib/stores/vlottoCache';

export interface VerificationResult {
	success: boolean;
	checks: {
		ticketSignedRegistration: boolean;
		ticketSignedHash: boolean;
		proofguardSignedTicketSig: boolean;
		proofguardSignedHash: boolean;
	};
	errors: string[];
}

/**
 * Verify ticket authenticity using 4-step verification
 *
 * Per vlotto integration guide Section 7:
 * 1. Ticket signed registration txid: verifymessage(T, sig1, regtx)
 * 2. Ticket signed hash1: verifyhash(T, sig1, hash1)
 * 3. Proofguard signed sig1: verifymessage(proofguard.main@, sig2, sig1)
 * 4. Proofguard signed hash2: verifyhash(proofguard.main@, sig2, hash2)
 *
 * @param ticket - Ticket data with signatures and hashes
 * @param mainVerusID - Main lottery ID (e.g., "vlotto@")
 * @param chain - Network chain (VRSCTEST or VRSC)
 */
export async function verifyTicketAuthenticity(
	ticket: VlottoTicketData,
	mainVerusID: string,
	chain?: string
): Promise<VerificationResult> {
	const result: VerificationResult = {
		success: false,
		checks: {
			ticketSignedRegistration: false,
			ticketSignedHash: false,
			proofguardSignedTicketSig: false,
			proofguardSignedHash: false
		},
		errors: []
	};

	// Validate required fields
	if (!ticket.ticketValidation || !ticket.proofguardAcknowledgement) {
		result.errors.push('Missing signature data');
		return result;
	}

	const {
		signedByTicketSignature: sig1,
		signedByTicketHash: hash1
	} = ticket.ticketValidation;

	const {
		signedByProofguardSignature: sig2,
		signedByProofguardHash: hash2
	} = ticket.proofguardAcknowledgement;

	if (!sig1 || !hash1 || !sig2 || !hash2) {
		result.errors.push('Incomplete signature data');
		return result;
	}

	if (!ticket.registrationTxid) {
		result.errors.push('Missing registration txid');
		return result;
	}

	// Construct proofguard identity name
	// Trim trailing @ from mainVerusID if present
	const trimmedMain = mainVerusID.replace(/@$/, '');
	const proofguardIdentity = `proofguard.${trimmedMain}@`;

	try {
		// Check 1: Ticket signed the registration txid
		try {
			result.checks.ticketSignedRegistration = await invoke<boolean>('verify_message', {
				taddrOrIdentity: ticket.name,
				signature: sig1,
				message: ticket.registrationTxid,
				checklatest: false,
				chain: chain?.toLowerCase()
			});
		} catch (error: any) {
			result.errors.push(`Check 1 failed: ${error.toString()}`);
		}

		// Check 2: Ticket signed hash1
		try {
			result.checks.ticketSignedHash = await invoke<boolean>('verify_hash', {
				taddrOrIdentity: ticket.name,
				signature: sig1,
				hexhash: hash1,
				checklatest: false,
				chain: chain?.toLowerCase()
			});
		} catch (error: any) {
			result.errors.push(`Check 2 failed: ${error.toString()}`);
		}

		// Check 3: Proofguard signed sig1 (the ticket signature)
		try {
			result.checks.proofguardSignedTicketSig = await invoke<boolean>('verify_message', {
				taddrOrIdentity: proofguardIdentity,
				signature: sig2,
				message: sig1,
				checklatest: false,
				chain: chain?.toLowerCase()
			});
		} catch (error: any) {
			result.errors.push(`Check 3 failed: ${error.toString()}`);
		}

		// Check 4: Proofguard signed hash2
		try {
			result.checks.proofguardSignedHash = await invoke<boolean>('verify_hash', {
				taddrOrIdentity: proofguardIdentity,
				signature: sig2,
				hexhash: hash2,
				checklatest: false,
				chain: chain?.toLowerCase()
			});
		} catch (error: any) {
			result.errors.push(`Check 4 failed: ${error.toString()}`);
		}

		// All checks must pass for success
		result.success =
			result.checks.ticketSignedRegistration &&
			result.checks.ticketSignedHash &&
			result.checks.proofguardSignedTicketSig &&
			result.checks.proofguardSignedHash;
	} catch (error: any) {
		result.errors.push(`Verification error: ${error.toString()}`);
	}

	return result;
}

/**
 * Quick verification check (just returns true/false)
 */
export async function isTicketVerified(
	ticket: VlottoTicketData,
	mainVerusID: string,
	chain?: string
): Promise<boolean> {
	const result = await verifyTicketAuthenticity(ticket, mainVerusID, chain);
	return result.success;
}
