// Utilities for processing marketplace offers and converting them for takeoffer functionality

export type OfferType = 'id-for-id' | 'id-for-currency' | 'currency-for-id' | 'currency-for-currency' | 'currency-for-ids' | 'ids-for-currency';

export interface ParsedOfferData {
  txid: string;
  offerType: OfferType;
  // What the original maker offered (becomes what taker accepts)
  makerOffer: {
    type: 'identity' | 'currency';
    identityName?: string;
    currencyId?: string;
    amount?: number;
  };
  // What the original maker wanted (becomes what taker delivers)
  makerFor: {
    type: 'identity' | 'currency';
    identityId?: string;
    identityName?: string;
    currencyId?: string;
    amount?: number;
  };
}

export interface TakeOfferFormMapping {
  // What user will accept (receive) - from maker's offer
  acceptIdentityName?: string;
  acceptCurrency?: string;
  acceptAmount?: string;
  // What user will deliver (provide) - from maker's for
  deliverIdentity?: string;
  deliverIdentityName?: string;
  deliverCurrency?: string;
  deliverAmount?: string;
  // Determined offer type
  offerType: OfferType;
}

/**
 * Validates that an offer result has the required structure for take offer processing
 */
export function validateOfferStructure(offerResult: any): boolean {
  console.log('🔍 Validating offer structure:', offerResult);
  
  if (!offerResult) {
    console.warn('❌ Offer result is null/undefined');
    return false;
  }

  // Check for category (injected during search)
  if (!offerResult._category && !offerResult.category) {
    console.warn('❌ Missing category in offer result');
    return false;
  }

  // Check for offer data structure - based on real getoffers structure
  const offerData = offerResult.offer;
  
  if (!offerData) {
    console.warn('❌ Missing offer data structure at offerResult.offer');
    return false;
  }

  if (!offerData.txid) {
    console.warn('❌ Missing transaction ID in offer data');
    return false;
  }

  // Check for offer and accept structures (not 'for' - real data uses 'accept')
  if (!offerData.offer || !offerData.accept) {
    console.warn('❌ Missing offer.offer or offer.accept structures');
    return false;
  }

  console.log('✅ Offer structure validation passed');
  return true;
}

/**
 * Determines the offer type from category string
 */
export function determineOfferTypeFromCategory(category: string): OfferType {
  console.log('🎯 Determining offer type from category:', category);
  
  const cat = category.toLowerCase();
  
  // Pattern matching for all 6 real category formats from Verus
  
  // 1. ID for Currency: id_{identityid}_for_currency_{currencyid}
  if (cat.match(/^id_.*_for_currency_/)) {
    return 'id-for-currency';
  }
  
  // 2. Currency for ID: currency_{currencyid}_for_id_{identityid}
  if (cat.match(/^currency_.*_for_id_[^s]/)) { // not ending with 's' to distinguish from 'ids'
    return 'currency-for-id';
  }
  
  // 3. Currency for IDs (plural): currency_{currencyid}_for_ids
  if (cat.match(/^currency_.*_for_ids$/)) {
    return 'currency-for-ids';
  }
  
  // 4. IDs for Currency: ids_for_currency_{currencyid}
  if (cat.match(/^ids_for_currency_/)) {
    return 'ids-for-currency';
  }
  
  // 5. ID for IDs: id_{identityid}_for_ids or ids_for_id_{identityid}
  if (cat.match(/^id_.*_for_ids$/) || cat.match(/^ids_for_id_/)) {
    return 'id-for-id';
  }
  
  // 6. Currency for Currency: currency_{currencyid}_offers_in_currency_{currencyid}
  if (cat.match(/^currency_.*_offers_in_currency_/)) {
    return 'currency-for-currency';
  }
  
  console.warn('⚠️ Could not determine offer type from category, defaulting to currency-for-currency. Category:', category);
  return 'currency-for-currency';
}

/**
 * Parses offer data from search results into structured format
 */
export function parseOfferData(offerResult: any): ParsedOfferData | null {
  console.log('📋 Parsing offer data:', offerResult);
  
  if (!validateOfferStructure(offerResult)) {
    return null;
  }

  const category = offerResult._category || offerResult.category;
  const offerData = offerResult.offer; // Direct path based on real structure
  const txid = offerData.txid;
  
  const offerType = determineOfferTypeFromCategory(category);
  
  // Extract what the maker offered (becomes what taker accepts)
  const makerOfferData = offerData.offer || {};
  const makerOffer = extractOfferSide(makerOfferData);
  
  // Extract what the maker wanted (becomes what taker delivers) - real data uses 'accept'
  const makerForData = offerData.accept || {};
  const makerFor = extractOfferSide(makerForData);
  
  const parsed: ParsedOfferData = {
    txid,
    offerType,
    makerOffer,
    makerFor
  };
  
  console.log('✅ Parsed offer data:', parsed);
  return parsed;
}

/**
 * Helper function to extract offer side (offer or for/accept)
 */
function extractOfferSide(sideData: any): any {
  console.log('🔧 Extracting offer side:', sideData);
  
  // Check for identity
  if (sideData.name || sideData.identityid) {
    return {
      type: 'identity' as const,
      identityName: sideData.name,
      identityId: sideData.identityid
    };
  }
  
  // Check for currency object (like {"iXXXXX": 100})
  if (typeof sideData === 'object' && !sideData.name && !sideData.identityid) {
    const currencyKeys = Object.keys(sideData).filter(key => 
      key.startsWith('i') && typeof sideData[key] === 'number'
    );
    
    if (currencyKeys.length > 0) {
      const currencyId = currencyKeys[0];
      return {
        type: 'currency' as const,
        currencyId,
        amount: sideData[currencyId]
      };
    }
  }
  
  console.warn('⚠️ Could not extract offer side data:', sideData);
  return { type: 'currency' as const };
}

/**
 * Maps parsed offer data to form fields for take offer
 * Key insight: maker's offer becomes taker's accept, maker's for becomes taker's deliver
 */
export function mapToTakeOfferForm(parsedData: ParsedOfferData): TakeOfferFormMapping {
  console.log('🗺️ Mapping parsed data to take offer form:', parsedData);
  
  const mapping: TakeOfferFormMapping = {
    offerType: parsedData.offerType
  };
  
  // Map what user will ACCEPT (receive) - from maker's offer
  if (parsedData.makerOffer.type === 'identity') {
    mapping.acceptIdentityName = parsedData.makerOffer.identityName;
  } else if (parsedData.makerOffer.type === 'currency') {
    mapping.acceptCurrency = parsedData.makerOffer.currencyId;
    mapping.acceptAmount = parsedData.makerOffer.amount?.toString();
  }
  
  // Map what user will DELIVER (provide) - from maker's for
  if (parsedData.makerFor.type === 'identity') {
    mapping.deliverIdentity = parsedData.makerFor.identityId;
    mapping.deliverIdentityName = parsedData.makerFor.identityName;
  } else if (parsedData.makerFor.type === 'currency') {
    mapping.deliverCurrency = parsedData.makerFor.currencyId;
    mapping.deliverAmount = parsedData.makerFor.amount?.toString();
  }
  
  console.log('✅ Take offer form mapping:', mapping);
  return mapping;
}

/**
 * Main function to process offer result for take offer form
 */
export function processOfferForTakeForm(offerResult: any): TakeOfferFormMapping | null {
  console.log('🚀 Processing offer for take form:', offerResult);
  
  const parsedData = parseOfferData(offerResult);
  if (!parsedData) {
    console.error('❌ Failed to parse offer data');
    return null;
  }
  
  const mapping = mapToTakeOfferForm(parsedData);
  console.log('🎯 Final take offer mapping:', mapping);
  
  return mapping;
}