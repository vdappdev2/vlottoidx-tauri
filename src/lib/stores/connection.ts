import { writable } from 'svelte/store';

export interface RpcConnection {
  host: string;
  port: number;
  username: string;
  password: string;
  isConnected: boolean;
  chainName: string;
}

export interface ChainConfig {
  name: string;
  display_name: string;
  credentials: {
    host: string;
    port: number;
    username: string;
    password: string;
  };
  is_active: boolean;
}

export interface ConnectionState {
  current: RpcConnection | null;
  isConnecting: boolean;
  lastError: string | null;
  availableChains: ChainConfig[];
  selectedChain: string | null;
}

export const connectionStore = writable<ConnectionState>({
  current: null,
  isConnecting: false,
  lastError: null,
  availableChains: [],
  selectedChain: null
});

/**
 * Helper function to determine the correct chain parameter for RPC calls
 * @param selectedChain The currently selected chain name
 * @returns null for VRSC mainnet (no -chain= needed), chain name for others
 */
export function getChainParam(selectedChain: string | null): string | null {
  if (!selectedChain) {
    console.warn('getChainParam: selectedChain is null, defaulting to null (VRSC mainnet)');
    return null;
  }

  // VRSC mainnet doesn't need -chain= parameter
  if (selectedChain.toLowerCase() === 'vrsc') {
    return null;
  }

  // All other chains (vrsctest, varrr, vdex, etc.) need -chain=chainname
  return selectedChain.toLowerCase();
}

/**
 * Helper function to get the display name for a chain
 * @param state The connection state containing available chains
 * @param chainName The chain name to look up
 * @returns The display_name from availableChains, or fallback to uppercase chainName
 */
export function getChainDisplayName(state: ConnectionState, chainName: string | null): string {
  if (!chainName) return 'Unknown';
  const chain = state.availableChains.find(c => c.name === chainName);
  return chain?.display_name || chainName.toUpperCase();
}