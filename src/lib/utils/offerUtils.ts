/**
 * Offer utilities for marketplace display
 */

export type OfferType = 'ID→ID' | 'ID→Currency' | 'Currency→ID' | 'Currency→Currency';

export interface ParsedOffer {
  type: OfferType;
  offering: {
    type: 'identity' | 'currency';
    name: string;
    amount?: number;
    identityAddress?: string;
    currencyAddress?: string;
  };
  accepting: {
    type: 'identity' | 'currency';
    name: string;
    amount?: number;
    identityAddress?: string;
    currencyAddress?: string;
  };
  price?: number;
  blockExpiry?: number;
  txId?: string;
  isActive?: boolean;
}

/**
 * Map daemon category strings to user-friendly offer types
 * Categories from daemon:
 * 1. id_XXX_for_currency_YYY → ID→Currency
 * 2. currency_XXX_for_id_YYY → Currency→ID  
 * 3. ids_for_id_XXX → ID→ID
 * 4. currency_XXX_for_ids → Currency→ID (multiple IDs accepting)
 * 5. currency_XXX_offers_in_currency_YYY → Currency→Currency
 * 6. id_XXX_for_ids → ID→ID (multiple IDs accepting)
 */
export function mapCategoryToOfferType(category: string): OfferType {
  // ID→Currency patterns
  if (category.startsWith('id_') && category.includes('_for_currency_')) {
    return 'ID→Currency';
  }
  
  // Currency→ID patterns
  if (category.startsWith('currency_') && (category.includes('_for_id_') || category.includes('_for_ids'))) {
    return 'Currency→ID';
  }
  
  // ID→ID patterns
  if ((category.startsWith('ids_for_id_') || category.startsWith('id_')) && category.includes('_for_ids')) {
    return 'ID→ID';
  }
  
  // Currency→Currency patterns
  if (category.startsWith('currency_') && category.includes('_offers_in_currency_')) {
    return 'Currency→Currency';
  }
  
  // Fallback - try to infer from structure
  console.warn('Unknown category pattern:', category);
  return 'Currency→Currency'; // Most common fallback
}

/**
 * Extract offer data from marketplace search results (getoffers)
 * Expects raw RPC structure with injected _category field
 */
export function extractOfferFromSearchResult(offerResult: any): ParsedOffer | null {
  console.log('Extracting offer from:', offerResult);
  
  if (!offerResult?._category || !offerResult?.offer) {
    console.warn('Invalid offer result structure. Expected _category and offer fields:', offerResult);
    return null;
  }

  const category = offerResult._category;
  const offer = offerResult.offer;
  const price = offerResult.price;
  const currencyid = offerResult.currencyid;
  const identityid = offerResult.identityid;

  const offerType = mapCategoryToOfferType(category);
  
  // Extract offering data (what the original maker is offering)
  const offering = extractOfferingData(offer.offer, category, currencyid, identityid);
  
  // Extract accepting data (what the original maker wants)
  const accepting = extractAcceptingData(offer.accept, category, currencyid, identityid);

  return {
    type: offerType,
    offering,
    accepting,
    price: price,
    blockExpiry: offer.blockexpiry,
    txId: offer.txid,
    isActive: !isOfferExpired(offer.blockexpiry)
  };
}

/**
 * Extract offering data from offer.offer object
 */
function extractOfferingData(offerObj: any, category: string, currencyid?: string, identityid?: string): ParsedOffer['offering'] {
  console.log('Extracting offering data:', { offerObj, category, currencyid, identityid });
  
  // Identity being offered
  if (offerObj?.name && offerObj?.identityid) {
    return {
      type: 'identity',
      name: offerObj.name,
      identityAddress: offerObj.identityid
    };
  }
  
  // Currency being offered - look for i-address keys with amounts
  const currencyKeys = Object.keys(offerObj || {}).filter(key => 
    key.startsWith('i') && typeof offerObj[key] === 'number'
  );
  
  if (currencyKeys.length > 0) {
    // Use the first currency (primary one being offered)
    const currencyAddress = currencyKeys[0];
    const amount = offerObj[currencyAddress];
    
    return {
      type: 'currency',
      name: currencyAddress, // Will be resolved to friendly name later
      amount: amount,
      currencyAddress: currencyAddress
    };
  }
  
  return {
    type: 'currency',
    name: 'Unknown',
    amount: 0
  };
}

/**
 * Extract accepting data from offer.accept object
 */
function extractAcceptingData(acceptObj: any, category: string, currencyid?: string, identityid?: string): ParsedOffer['accepting'] {
  console.log('Extracting accepting data:', { acceptObj, category, currencyid, identityid });
  
  // Identity being accepted
  if (acceptObj?.name && acceptObj?.identityid) {
    return {
      type: 'identity',
      name: acceptObj.name,
      identityAddress: acceptObj.identityid
    };
  }
  
  // Currency being accepted - look for i-address keys with amounts
  const currencyKeys = Object.keys(acceptObj || {}).filter(key => 
    key.startsWith('i') && typeof acceptObj[key] === 'number'
  );
  
  if (currencyKeys.length > 0) {
    // Use the first currency (primary one being accepted)
    const currencyAddress = currencyKeys[0];
    const amount = acceptObj[currencyAddress];
    
    return {
      type: 'currency',
      name: currencyAddress, // Will be resolved to friendly name later
      amount: amount,
      currencyAddress: currencyAddress
    };
  }
  
  return {
    type: 'currency',
    name: 'Unknown',
    amount: 0
  };
}

/**
 * Check if an offer is expired based on block height
 * Note: This is a simplified check - in reality we'd need current block height
 */
function isOfferExpired(blockExpiry?: number): boolean {
  if (!blockExpiry) return false;
  // For now, assume all offers are active
  // TODO: Compare with actual current block height when available
  return false;
}

/**
 * Format crypto amounts for display
 */
export function formatCryptoAmount(amount: number): string {
  // For very small amounts (less than or equal to 0.01), show full precision up to 8 decimal places
  if (amount <= 0.01 && amount > 0) {
    // Use toFixed(8) and manually remove trailing zeros to avoid scientific notation
    let formatted = amount.toFixed(8);
    // Remove trailing zeros and unnecessary decimal point
    formatted = formatted.replace(/\.?0+$/, '');
    // If we removed all decimals, ensure we keep at least one zero for clarity
    if (!formatted.includes('.')) {
      formatted += '.0';
    }
    return formatted;
  }
  // For larger amounts, use normal locale formatting
  return amount.toLocaleString();
}

/**
 * Format offer display text for cards
 */
export function formatOfferSummary(offering: ParsedOffer['offering'], accepting: ParsedOffer['accepting']): {
  offeringText: string;
  acceptingText: string;
} {
  const offeringText = offering.amount !== undefined 
    ? `${formatCryptoAmount(offering.amount)} ${offering.name}`
    : offering.name;
    
  const acceptingText = accepting.amount !== undefined
    ? `${formatCryptoAmount(accepting.amount)} ${accepting.name}`
    : accepting.name;

  return { offeringText, acceptingText };
}

/**
 * Get display icon for offer type
 */
export function getOfferTypeIcon(type: OfferType): string {
  switch (type) {
    case 'ID→ID': return '👤';
    case 'ID→Currency': return '👤💰';
    case 'Currency→ID': return '💰👤';
    case 'Currency→Currency': return '💰';
    default: return '🔄';
  }
}