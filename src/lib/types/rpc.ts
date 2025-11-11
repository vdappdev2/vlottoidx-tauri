// Basic RPC types that match the Rust backend

export interface GetInfoResponse {
  version: string;
  VRSCversion?: string;
  protocolversion: number;
  blocks: number;
  difficulty: number;
  connections: number;
  halving?: number;
}

export interface GetWalletInfoResponse {
  walletname: string;
  walletversion: number;
  balance: number;
  unconfirmed_balance: number;
  immature_balance: number;
  reserve_balance?: number;
  txcount: number;
  keypoololdest: number;
  keypoolsize: number;
  unlocked_until?: number;
}

export interface Identity {
  identity: string;
  identityaddress: string;
  parent: string;
  name: string;
  contentmap: Record<string, string>;
  contentmultimap: Record<string, string[]>;
  canspendfor: boolean;
  cansignfor: boolean;
  status: string;
  version: number;
}