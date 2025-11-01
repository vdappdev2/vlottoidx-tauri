// System & Wallet RPC Commands
use crate::rpc::{
    VerusRpcClient, RpcError,
    GetInfoResponse, GetWalletInfoResponse, GetMiningInfoResponse, 
    GetBlockchainInfoResponse, GetNetworkInfoResponse, AddressGroupings,
    ZTotalBalanceResponse, OperationStatus
};
use serde_json::json;

impl VerusRpcClient {
    /// Get current block count
    pub async fn get_block_count(&self, chain: Option<&str>) -> Result<u64, RpcError> {
        self.call_with_chain("getblockcount", json!([]), chain).await
    }

    /// Get general system info
    pub async fn get_info(&self, chain: Option<&str>) -> Result<GetInfoResponse, RpcError> {
        self.call_with_chain("getinfo", json!([]), chain).await
    }

    /// Get wallet-specific info
    pub async fn get_wallet_info(&self, chain: Option<&str>) -> Result<GetWalletInfoResponse, RpcError> {
        self.call_with_chain("getwalletinfo", json!([]), chain).await
    }

    /// Get mining/staking info
    pub async fn get_mining_info(&self, chain: Option<&str>) -> Result<GetMiningInfoResponse, RpcError> {
        self.call_with_chain("getmininginfo", json!([]), chain).await
    }

    /// Get blockchain status
    pub async fn get_blockchain_info(&self, chain: Option<&str>) -> Result<GetBlockchainInfoResponse, RpcError> {
        self.call_with_chain("getblockchaininfo", json!([]), chain).await
    }

    /// Get network status
    pub async fn get_network_info(&self, chain: Option<&str>) -> Result<GetNetworkInfoResponse, RpcError> {
        self.call_with_chain("getnetworkinfo", json!([]), chain).await
    }

    /// List address groups with balances
    pub async fn list_address_groupings(&self, chain: Option<&str>) -> Result<AddressGroupings, RpcError> {
        self.call_with_chain("listaddressgroupings", json!([]), chain).await
    }

    /// List private addresses
    pub async fn z_list_addresses(&self, include_watchonly: Option<bool>, chain: Option<&str>) -> Result<Vec<String>, RpcError> {
        let mut params = vec![];
        
        if let Some(watchonly) = include_watchonly {
            params.push(json!(watchonly));
        }
        
        self.call_with_chain("z_listaddresses", json!(params), chain).await
    }

    /// Generate new transparent address
    pub async fn get_new_address(&self, account: Option<&str>, chain: Option<&str>) -> Result<String, RpcError> {
        let mut params = vec![];
        
        if let Some(acc) = account {
            params.push(json!(acc));
        }
        
        self.call_with_chain("getnewaddress", json!(params), chain).await
    }

    /// Generate new private address
    pub async fn z_get_new_address(&self, address_type: Option<&str>, chain: Option<&str>) -> Result<String, RpcError> {
        let mut params = vec![];
        
        if let Some(addr_type) = address_type {
            params.push(json!(addr_type));
        }
        
        self.call_with_chain("z_getnewaddress", json!(params), chain).await
    }

    /// Get status of async operations
    pub async fn z_get_operation_status(&self, operation_ids: Option<Vec<String>>, chain: Option<&str>) -> Result<Vec<OperationStatus>, RpcError> {
        let mut params = vec![];
        
        if let Some(ids) = operation_ids {
            params.push(json!(ids));
        }
        
        self.call_with_chain("z_getoperationstatus", json!(params), chain).await
    }

    /// Get total balance including private
    pub async fn z_get_total_balance(&self, minconf: Option<u32>, include_watchonly: Option<bool>, chain: Option<&str>) -> Result<ZTotalBalanceResponse, RpcError> {
        let mut params = Vec::new();
        
        if let Some(conf) = minconf {
            params.push(json!(conf));
            if let Some(watchonly) = include_watchonly {
                params.push(json!(watchonly));
            }
        }
        
        self.call_with_chain("z_gettotalbalance", json!(params), chain).await
    }

    /// Get addresses by account (use empty string "" for default account)
    pub async fn get_addresses_by_account(&self, account: &str, chain: Option<&str>) -> Result<Vec<String>, RpcError> {
        // Account is required - typically empty string ""
        let params = vec![json!(account)];
        
        self.call_with_chain("getaddressesbyaccount", json!(params), chain).await
    }

    /// List transactions with various filters
    pub async fn list_transactions(&self, account: Option<&str>, count: Option<u32>, from: Option<u32>, include_watchonly: Option<bool>, chain: Option<&str>) -> Result<Vec<serde_json::Value>, RpcError> {
        let mut params = Vec::new();
        
        // Account or "*" for all accounts
        params.push(json!(account.unwrap_or("*")));
        
        // Count of transactions to return
        if let Some(c) = count {
            params.push(json!(c));
            
            // From index
            if let Some(f) = from {
                params.push(json!(f));
                
                // Include watch only
                if let Some(watchonly) = include_watchonly {
                    params.push(json!(watchonly));
                }
            }
        }
        
        self.call_with_chain("listtransactions", json!(params), chain).await
    }

    /// Get currency balance for a specific address
    pub async fn get_currency_balance(&self, address: &str, minconf: Option<u32>, friendly_names: Option<bool>, include_shared: Option<bool>, chain: Option<&str>) -> Result<serde_json::Value, RpcError> {
        let mut params = Vec::new();
        
        // Address is required
        params.push(json!(address));
        
        // Minimum confirmations
        if let Some(conf) = minconf {
            params.push(json!(conf));
            
            // Friendly names
            if let Some(friendly) = friendly_names {
                params.push(json!(friendly));
                
                // Include shared
                if let Some(shared) = include_shared {
                    params.push(json!(shared));
                }
            }
        }
        
        self.call_with_chain("getcurrencybalance", json!(params), chain).await
    }
}