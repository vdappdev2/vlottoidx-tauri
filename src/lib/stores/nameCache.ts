import { invoke } from "@tauri-apps/api/core";
import { getChainParam } from "./connection";

interface CacheEntry {
  name: string;
  timestamp: number;
}

// In-memory cache for friendly names
const identityNameCache = new Map<string, CacheEntry>();
const currencyNameCache = new Map<string, CacheEntry>();

// Cache duration in milliseconds (5 minutes)
const CACHE_DURATION = 5 * 60 * 1000;

/**
 * Get friendly name for an identity, with caching
 */
export async function getIdentityFriendlyName(
  identityAddress: string, 
  chainParam?: string
): Promise<string> {
  // Check cache first
  const cached = identityNameCache.get(identityAddress);
  const now = Date.now();
  
  if (cached && (now - cached.timestamp) < CACHE_DURATION) {
    return cached.name;
  }

  try {
    console.log("NameCache: Fetching identity name for:", identityAddress);
    const result = await invoke("get_identity", { 
      name: identityAddress,
      chain: chainParam
    });

    let friendlyName = identityAddress; // Fallback to i-address
    
    if (result && typeof result === 'object') {
      console.log("NameCache: Identity API response:", result);
      // Extract friendly name from identity data
      if (result.identity?.name) {
        friendlyName = result.identity.name;
        console.log("NameCache: Found identity name:", friendlyName);
      } else if (result.fullyqualifiedname) {
        friendlyName = result.fullyqualifiedname;
        console.log("NameCache: Found fully qualified name:", friendlyName);
      } else {
        console.warn("NameCache: No friendly name found in identity response, using i-address");
      }
    } else {
      console.warn("NameCache: Invalid or empty identity response");
    }

    // Cache the result
    identityNameCache.set(identityAddress, {
      name: friendlyName,
      timestamp: now
    });

    return friendlyName;
  } catch (err) {
    console.warn("NameCache: Failed to fetch identity name for", identityAddress, err);
    return identityAddress; // Return i-address as fallback
  }
}

/**
 * Get friendly name for a currency, with caching
 */
export async function getCurrencyFriendlyName(
  currencyAddress: string, 
  chainParam?: string
): Promise<string> {
  // Check cache first
  const cached = currencyNameCache.get(currencyAddress);
  const now = Date.now();
  
  if (cached && (now - cached.timestamp) < CACHE_DURATION) {
    return cached.name;
  }

  try {
    console.log("NameCache: Fetching currency name for:", currencyAddress, "chain:", chainParam);
    const result = await invoke("get_currency", { 
      currencyName: currencyAddress,
      height: null,
      chain: chainParam
    });

    let friendlyName = currencyAddress; // Fallback to i-address
    
    if (result && typeof result === 'object') {
      console.log("NameCache: Currency API response:", result);
      // Extract friendly name from currency data
      if (result.fullyqualifiedname) {
        friendlyName = result.fullyqualifiedname;
        console.log("NameCache: Found currency fully qualified name:", friendlyName);
      } else if (result.name) {
        friendlyName = result.name;
        console.log("NameCache: Found currency name:", friendlyName);
      } else {
        console.warn("NameCache: No friendly name found in currency response, using i-address");
      }
    } else {
      console.warn("NameCache: Invalid or empty currency response");
    }

    // Cache the result
    currencyNameCache.set(currencyAddress, {
      name: friendlyName,
      timestamp: now
    });

    return friendlyName;
  } catch (err) {
    console.warn("NameCache: Failed to fetch currency name for", currencyAddress, err);
    return currencyAddress; // Return i-address as fallback
  }
}

/**
 * Clear all cached names (useful for chain switches)
 */
export function clearNameCache() {
  identityNameCache.clear();
  currencyNameCache.clear();
  console.log("NameCache: Cleared all cached names");
}

/**
 * Get cache statistics for debugging
 */
export function getCacheStats() {
  return {
    identities: {
      count: identityNameCache.size,
      entries: Array.from(identityNameCache.entries())
    },
    currencies: {
      count: currencyNameCache.size,
      entries: Array.from(currencyNameCache.entries())
    }
  };
}