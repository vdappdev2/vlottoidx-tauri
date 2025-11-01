// Tauri command handlers
// These functions are called from the frontend

use crate::rpc::{VerusRpcClient, RpcCredentials, ChainConfig, SupportedChain, ChainDiscovery, CredentialManager, CurrencyDefinition};
use serde_json::{json, Value};
use tauri::State;
use std::sync::Arc;
use tokio::sync::RwLock;

// Application state
pub struct AppState {
    pub active_client: Arc<RwLock<Option<VerusRpcClient>>>,
    pub discovered_chains: Arc<RwLock<Vec<ChainConfig>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_client: Arc::new(RwLock::new(None)),
            discovered_chains: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

// Test command (from template)
#[tauri::command]
pub async fn greet(name: &str) -> Result<String, String> {
    Ok(format!("Hello, {}! You've been greeted from Rust and Tauri!", name))
}


// System & Wallet Commands
#[tauri::command]
pub async fn get_info(chain: Option<String>, state: State<'_, AppState>) -> Result<Value, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or("No active RPC connection")?;
        
    match client.get_info(chain.as_deref()).await {
        Ok(response) => {
            Ok(serde_json::to_value(response).unwrap())
        }
        Err(e) => {
            eprintln!("ERROR: get_info command failed: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn get_wallet_info(chain: Option<String>, state: State<'_, AppState>) -> Result<Value, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    match client.get_wallet_info(chain.as_deref()).await {
        Ok(response) => {
            Ok(serde_json::to_value(response).unwrap())
        }
        Err(e) => {
            eprintln!("ERROR: get_wallet_info command failed: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn get_mining_info(chain: Option<String>, state: State<'_, AppState>) -> Result<Value, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    client.get_mining_info(chain.as_deref())
        .await
        .map(|r| serde_json::to_value(r).unwrap())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn z_get_total_balance(
    minconf: Option<u32>,
    include_watchonly: Option<bool>,
    chain: Option<String>,
    state: State<'_, AppState>
) -> Result<Value, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    client.z_get_total_balance(minconf, include_watchonly, chain.as_deref())
        .await
        .map(|r| serde_json::to_value(r).unwrap())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_blockchain_info(chain: Option<String>, state: State<'_, AppState>) -> Result<Value, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    client.get_blockchain_info(chain.as_deref())
        .await
        .map(|r| serde_json::to_value(r).unwrap())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_identities(chain: Option<String>, state: State<'_, AppState>) -> Result<Value, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    client.list_identities(None, None, None, chain.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_identity(name: String, chain: Option<String>, state: State<'_, AppState>) -> Result<Value, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;

    // Use the RPC client method to handle chain parameters properly
    client.get_identity(&name, None, None, None, chain.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_identity_content(
    name: String,
    heightstart: Option<u64>,
    heightend: Option<u64>,
    txproofs: Option<bool>,
    vdxfkey: Option<String>,
    chain: Option<String>,
    state: State<'_, AppState>
) -> Result<Value, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;

    client.get_identity_content(
        &name,
        heightstart,
        heightend,
        txproofs,
        vdxfkey.as_deref(),
        chain.as_deref()
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn verify_message(
    taddr_or_identity: String,
    signature: String,
    message: String,
    checklatest: Option<bool>,
    chain: Option<String>,
    state: State<'_, AppState>
) -> Result<bool, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;

    client.verify_message(
        &taddr_or_identity,
        &signature,
        &message,
        checklatest,
        chain.as_deref()
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn verify_hash(
    taddr_or_identity: String,
    signature: String,
    hexhash: String,
    checklatest: Option<bool>,
    chain: Option<String>,
    state: State<'_, AppState>
) -> Result<bool, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;

    client.verify_hash(
        &taddr_or_identity,
        &signature,
        &hexhash,
        checklatest,
        chain.as_deref()
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_currencies(
    query: Option<String>,
    verbose: Option<bool>,
    launchstate: Option<String>,
    systemtype: Option<String>,
    fromsystem: Option<String>,
    converter: Option<Vec<String>>,
    state: State<'_, AppState>
) -> Result<Value, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    // Build query object if any advanced parameters are provided
    let mut query_obj = serde_json::Map::new();
    let mut has_query_params = false;
    
    if let Some(ls) = launchstate {
        query_obj.insert("launchstate".to_string(), serde_json::json!(ls));
        has_query_params = true;
    }
    
    if let Some(st) = systemtype {
        query_obj.insert("systemtype".to_string(), serde_json::json!(st));
        has_query_params = true;
    }
    
    if let Some(fs) = fromsystem {
        query_obj.insert("fromsystem".to_string(), serde_json::json!(fs));
        has_query_params = true;
    }
    
    if let Some(conv) = converter {
        if !conv.is_empty() {
            query_obj.insert("converter".to_string(), serde_json::json!(conv));
            has_query_params = true;
        }
    }
    
    let mut params = Vec::new();
    
    // If we have query object parameters, use them; otherwise fall back to legacy query string
    if has_query_params {
        params.push(serde_json::json!(query_obj));
        if let Some(v) = verbose {
            params.push(serde_json::json!(v));
        }
    } else if let Some(q) = query {
        params.push(serde_json::json!(q));
        if let Some(v) = verbose {
            params.push(serde_json::json!(v));
        }
    }
    
    client.call("listcurrencies", serde_json::json!(params))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_currency(
    currency_name: String,
    height: Option<u64>,
    state: State<'_, AppState>
) -> Result<Value, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    // Call the RPC method directly without trying to deserialize into a struct
    // since the actual response structure varies
    let mut params = vec![];
    
    params.push(serde_json::json!(currency_name));
    
    if let Some(h) = height {
        params.push(serde_json::json!(h));
    }
    
    client.call("getcurrency", serde_json::json!(params))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_currency_converters(
    currencies: Vec<String>,
    state: State<'_, AppState>
) -> Result<Value, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    let currency_refs: Vec<&str> = currencies.iter().map(|s| s.as_str()).collect();
    
    client.get_currency_converters(currency_refs)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_currency_state(
    currency_name: String,
    height_range: Option<String>,
    conversion_currency: Option<String>,
    state: State<'_, AppState>
) -> Result<Value, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    let height_range_ref = height_range.as_ref().map(|s| s.as_str());
    let conversion_currency_ref = conversion_currency.as_ref().map(|s| s.as_str());
    
    client.get_currency_state(&currency_name, height_range_ref, conversion_currency_ref)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_offers(
    currency_or_id: String, 
    is_currency: Option<bool>, 
    with_tx: Option<bool>, 
    state: State<'_, AppState>
) -> Result<Value, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    client.get_offers(&currency_or_id, is_currency, with_tx)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_open_offers(state: State<'_, AppState>) -> Result<Value, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    client.list_open_offers()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn close_offers(
    offer_tx_ids: Vec<String>,
    destination_address: Option<String>,
    state: State<'_, AppState>
) -> Result<bool, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    client.close_offers(offer_tx_ids, destination_address.as_deref())
        .await
        .map(|_| true)  // Return true on success since closeoffers returns null
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn make_offer(
    from_address: String,
    offer_data: Value,
    return_tx: Option<bool>,
    state: State<'_, AppState>
) -> Result<Value, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    client.make_offer(&from_address, &offer_data, return_tx)
        .await
        .map(|r| serde_json::to_value(r).unwrap())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn take_offer(
    from_address: String,
    offer_data: Value,
    return_tx: Option<bool>,
    state: State<'_, AppState>
) -> Result<Value, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    client.take_offer(&from_address, &offer_data, return_tx)
        .await
        .map(|r| serde_json::to_value(r).unwrap())
        .map_err(|e| e.to_string())
}

// Chain Discovery Commands
#[tauri::command]
pub async fn discover_chains(state: State<'_, AppState>) -> Result<Value, String> {
    let mut discovery = ChainDiscovery::new();
    
    match discovery.discover_chains().await {
        Ok(chains) => {
            // Test connectivity for discovered chains
            if let Err(e) = discovery.test_chain_connectivity().await {
                eprintln!("Warning: Failed to test some chain connections: {}", e);
            }
            
            // Update app state with discovered chains
            let mut discovered_chains = state.discovered_chains.write().await;
            *discovered_chains = discovery.discovered_chains.clone();
            
            Ok(serde_json::to_value(discovery.discovered_chains).unwrap())
        }
        Err(e) => Err(format!("Chain discovery failed: {}", e))
    }
}

#[tauri::command]
pub async fn get_discovered_chains(state: State<'_, AppState>) -> Result<Value, String> {
    let discovered_chains = state.discovered_chains.read().await;
    Ok(serde_json::to_value(discovered_chains.clone()).unwrap())
}

#[tauri::command]
pub async fn connect_to_chain(
    chain_name: String,
    state: State<'_, AppState>
) -> Result<bool, String> {
    let discovered_chains = state.discovered_chains.read().await;
    
    // Log chain connection attempts
    eprintln!("Connecting to chain: {}", chain_name);
    
    // Find the requested chain (case-insensitive comparison)
    let chain = discovered_chains
        .iter()
        .find(|c| c.name.to_string() == chain_name.to_lowercase())
        .ok_or(format!("Chain '{}' not found in discovered chains. Available: {:?}", 
            chain_name, 
            discovered_chains.iter().map(|c| c.name.to_string()).collect::<Vec<_>>()))?;
    
    if !chain.is_active {
        return Err("Chain is not active/reachable".to_string());
    }
    
    // Create new client for this chain
    let client = VerusRpcClient::new(chain.credentials.clone()).map_err(|e| e.to_string())?;
    
    // Test the connection
    match client.test_connection().await {
        Ok(_) => {
            // Store the working client
            let mut active_client = state.active_client.write().await;
            *active_client = Some(client);
            Ok(true)
        }
        Err(e) => Err(format!("Failed to connect to chain: {}", e))
    }
}

// Credential Management Commands
#[tauri::command]
pub async fn store_credentials(
    chain_name: String,
    username: String,
    password: String,
    host: String,
    port: u16
) -> Result<bool, String> {
    let chain = match chain_name.as_str() {
        "vrsc" => SupportedChain::Vrsc,
        "vrsctest" => SupportedChain::VrscTest,
        "varrr" => SupportedChain::Varrr,
        "vdex" => SupportedChain::Vdex,
        "chips" => SupportedChain::Chips,
        _ => return Err("Unsupported chain".to_string()),
    };

    let credentials = RpcCredentials {
        username,
        password,
        host,
        port,
    };

    // Validate credentials first
    CredentialManager::validate_credentials(&credentials)
        .map_err(|e| e.to_string())?;

    let cred_manager = CredentialManager::new();
    cred_manager.store_credentials(&chain, &credentials)
        .map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub async fn load_credentials(chain_name: String) -> Result<Value, String> {
    let chain = match chain_name.as_str() {
        "vrsc" => SupportedChain::Vrsc,
        "vrsctest" => SupportedChain::VrscTest,
        "varrr" => SupportedChain::Varrr,
        "vdex" => SupportedChain::Vdex,
        "chips" => SupportedChain::Chips,
        _ => return Err("Unsupported chain".to_string()),
    };

    let cred_manager = CredentialManager::new();
    match cred_manager.load_credentials(&chain) {
        Ok(credentials) => {
            // Return credentials but redact the password for security
            let safe_creds = serde_json::json!({
                "username": credentials.username,
                "host": credentials.host,
                "port": credentials.port,
                "has_password": !credentials.password.is_empty()
            });
            Ok(safe_creds)
        }
        Err(e) => Err(e.to_string())
    }
}

#[tauri::command]
pub async fn clear_credentials(chain_name: String) -> Result<bool, String> {
    let chain = match chain_name.as_str() {
        "vrsc" => SupportedChain::Vrsc,
        "vrsctest" => SupportedChain::VrscTest,
        "varrr" => SupportedChain::Varrr,
        "vdex" => SupportedChain::Vdex,
        "chips" => SupportedChain::Chips,
        _ => return Err("Unsupported chain".to_string()),
    };

    let cred_manager = CredentialManager::new();
    cred_manager.clear_credentials(&chain)
        .map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub async fn list_stored_chains() -> Result<Value, String> {
    let cred_manager = CredentialManager::new();
    let chains = cred_manager.list_stored_chains();
    let chain_names: Vec<String> = chains.iter().map(|c| c.to_string().to_string()).collect();
    Ok(serde_json::to_value(chain_names).unwrap())
}

#[tauri::command]
pub async fn get_expected_config_paths() -> Result<String, String> {
    Ok(crate::rpc::ChainConfig::get_expected_paths())
}

// Additional system commands
#[tauri::command]
pub async fn get_block_count(chain: Option<String>, state: State<'_, AppState>) -> Result<u64, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    // Use the RPC client method to handle chain parameters properly
    client.get_block_count(chain.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_address_groupings(chain: Option<String>, state: State<'_, AppState>) -> Result<Value, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    client.list_address_groupings(chain.as_deref())
        .await
        .map(|r| serde_json::to_value(r).unwrap())
        .map_err(|e| e.to_string())
}

// Currency conversion and estimation commands
#[tauri::command]
pub async fn estimate_conversion(
    currency: String,
    amount: f64,
    convertto: String,
    via: Option<String>,
    state: State<'_, AppState>
) -> Result<Value, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    client.estimate_conversion(
        &currency, 
        amount, 
        &convertto, 
        via.as_deref()
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn convert_currency(
    from_address: String,
    source_currency: String,
    amount: f64,
    target_currency: String,
    to_address: String,
    state: State<'_, AppState>
) -> Result<Value, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    client.convert_currency(&from_address, &source_currency, amount, &target_currency, &to_address)
        .await
        .map(|opid| serde_json::Value::String(opid))
        .map_err(|e| e.to_string())
}

// Simplified conversion specifically for agents funding
#[tauri::command]
pub async fn convert_to_verusidx(
    from_address: String,
    source_currency: String,
    amount: f64,
    to_address: String,
    state: State<'_, AppState>
) -> Result<Value, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    // Fixed target currency for agents funding
    // Testing configuration
    client.convert_currency(&from_address, &source_currency, amount, "agents", &to_address)
    // Production configuration
    // client.convert_currency(&from_address, &source_currency, amount, "VerusIDX", &to_address)
        .await
        .map(|opid| serde_json::Value::String(opid))
        .map_err(|e| e.to_string())
}

// DeFi & Cross-Chain Operations Commands
#[tauri::command]
pub async fn send_currency(
    from_address: String,
    outputs: Value,
    min_conf: Option<u32>,
    fee_amount: Option<f64>,
    subtract_fee_from_amount: Option<bool>,
    state: State<'_, AppState>
) -> Result<String, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    // Parse outputs from frontend JSON
    let outputs: Vec<crate::rpc::SendCurrencyRequest> = serde_json::from_value(outputs)
        .map_err(|e| format!("Invalid outputs format: {}", e))?;
    
    client.send_currency(&from_address, outputs, min_conf, fee_amount, subtract_fee_from_amount)
        .await
        .map_err(|e| e.to_string())
}

// Identity management commands
#[tauri::command]
pub async fn revoke_identity(
    name_or_id: String,
    return_tx: Option<bool>,
    token_revoke: Option<bool>,
    fee_offer: Option<f64>,
    source_of_funds: Option<String>,
    state: State<'_, AppState>
) -> Result<Value, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    client.revoke_identity(
        &name_or_id,
        return_tx,
        token_revoke,
        fee_offer,
        source_of_funds.as_deref()
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn recover_identity(
    json_identity: Value,
    return_tx: Option<bool>,
    token_recover: Option<bool>,
    fee_offer: Option<f64>,
    source_of_funds: Option<String>,
    state: State<'_, AppState>
) -> Result<Value, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    client.recover_identity(
        &json_identity,
        return_tx,
        token_recover,
        fee_offer,
        source_of_funds.as_deref()
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_identity(
    json_identity: Value,
    return_tx: Option<bool>,
    token_update: Option<bool>,
    fee_offer: Option<f64>,
    source_of_funds: Option<String>,
    state: State<'_, AppState>
) -> Result<Value, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    // Build parameters for updateidentity RPC call
    let mut params = vec![json_identity];
    
    // Add optional parameters in order as per Verus RPC spec
    if let Some(ret_tx) = return_tx {
        params.push(serde_json::json!(ret_tx));
    } else {
        params.push(serde_json::json!(false));
    }
    
    if let Some(token_upd) = token_update {
        params.push(serde_json::json!(token_upd));
    } else {
        params.push(serde_json::json!(false));
    }
    
    // When source_of_funds is provided, automatically use standard fee if not specified
    if let Some(fee) = fee_offer {
        params.push(serde_json::json!(fee));
    } else if source_of_funds.is_some() {
        // Automatically use 0.0001 fee when sourceoffunds is specified
        params.push(serde_json::json!(0.0001));
    } else if token_update.is_some() || return_tx.is_some() {
        params.push(serde_json::json!(""));
    }

    if let Some(source) = source_of_funds {
        params.push(serde_json::json!(source));
    }
    
    // Make direct RPC call to updateidentity
    client.call("updateidentity", serde_json::json!(params))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_identity_timelock(
    identity: String,
    timelock_params: Value,
    return_tx: Option<bool>,
    fee_offer: Option<f64>,
    source_of_funds: Option<String>,
    state: State<'_, AppState>
) -> Result<Value, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    client.set_identity_timelock(
        &identity,
        &timelock_params,
        return_tx,
        fee_offer,
        source_of_funds.as_deref()
    )
    .await
    .map_err(|e| e.to_string())
}

// Identity registration commands
#[tauri::command]
pub async fn register_name_commitment(
    name: String,
    control_address: String,
    referral: Option<String>,
    parent_name_or_id: Option<String>,
    source_of_funds: Option<String>,
    state: State<'_, AppState>
) -> Result<Value, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    // Build parameters array for registernamecommitment RPC call
    let mut params = vec![
        serde_json::json!(name),
        serde_json::json!(control_address)
    ];
    
    // Add optional parameters in order as per Verus RPC spec
    if let Some(ref_id) = referral {
        if !ref_id.is_empty() {
            params.push(serde_json::json!(ref_id));
        } else {
            params.push(serde_json::json!(""));
        }
    } else {
        params.push(serde_json::json!(""));
    }
    
    if let Some(parent) = parent_name_or_id {
        if !parent.is_empty() {
            params.push(serde_json::json!(parent));
        } else {
            params.push(serde_json::json!(""));
        }
    } else {
        params.push(serde_json::json!(""));
    }
    
    if let Some(source) = source_of_funds {
        if !source.is_empty() {
            params.push(serde_json::json!(source));
        }
    }
    
    // Make direct RPC call to registernamecommitment
    client.call("registernamecommitment", serde_json::json!(params))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn register_identity(
    txid: String,
    namereservation: Value,
    identity: Value,
    return_tx: Option<bool>,
    fee_offer: Option<f64>,
    source_of_funds: Option<String>,
    state: State<'_, AppState>
) -> Result<Value, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;

    // Construct the complete registeridentity payload as per Verus RPC spec
    let registration_payload = serde_json::json!({
        "txid": txid,
        "namereservation": namereservation,
        "identity": identity
    });

    // Build parameters array for RPC call
    // IMPORTANT: registeridentity expects POSITIONAL parameters: [payload, returntx, feeoffer, sourceoffunds]
    // We must maintain proper order - can't skip params in the middle
    let mut params = vec![registration_payload];

    // Determine if we need to add optional params (if any are present, we need proper positional order)
    if fee_offer.is_some() || source_of_funds.is_some() || return_tx.is_some() {
        // Always include return_tx if we're adding any other params
        let return_transaction = return_tx.unwrap_or(false);
        params.push(serde_json::json!(return_transaction));

        // If we have source_of_funds, we MUST include feeoffer (even if 0) to maintain position
        if source_of_funds.is_some() {
            let fee = fee_offer.unwrap_or(0.0);
            params.push(serde_json::json!(fee));

            if let Some(source) = source_of_funds {
                params.push(serde_json::json!(source));
            }
        } else if let Some(fee) = fee_offer {
            // We have feeoffer but no source_of_funds
            params.push(serde_json::json!(fee));
        }
    }

    // Make direct RPC call to registeridentity
    client.call("registeridentity", serde_json::json!(params))
        .await
        .map_err(|e| e.to_string())
}

// Wallet-specific commands
#[tauri::command]
pub async fn list_transactions(
    account: Option<String>,
    count: Option<u32>,
    from: Option<u32>,
    include_watchonly: Option<bool>,
    chain: Option<String>,
    state: State<'_, AppState>
) -> Result<Value, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    client.list_transactions(
        account.as_deref(),
        count,
        from,
        include_watchonly,
        chain.as_deref()
    )
    .await
    .map(|r| serde_json::to_value(r).unwrap())
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_currency_balance(
    address: String,
    minconf: Option<u32>,
    friendly_names: Option<bool>,
    include_shared: Option<bool>,
    chain: Option<String>,
    state: State<'_, AppState>
) -> Result<Value, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    client.get_currency_balance(
        &address,
        minconf,
        friendly_names,
        include_shared,
        chain.as_deref()
    )
    .await
    .map(|r| serde_json::to_value(r).unwrap())
    .map_err(|e| e.to_string())
}

// Currency Definition Commands
#[tauri::command]
pub async fn define_currency(
    currency_definition: CurrencyDefinition,
    fractional_gateway: Option<bool>,
    reserves: Option<Vec<String>>,
    state: State<'_, AppState>
) -> Result<Value, String> {
    // Debug logging
    eprintln!("Backend received define_currency with:");
    eprintln!("  name: {}", currency_definition.name);
    eprintln!("  options: {}", currency_definition.options);
    eprintln!("  proofprotocol: {:?}", currency_definition.proofprotocol);
    eprintln!("  currencies: {:?}", currency_definition.currencies);
    eprintln!("  maxpreconversion: {:?}", currency_definition.maxpreconversion);
    eprintln!("  preallocations: {:?}", currency_definition.preallocations);
    eprintln!("  fractional_gateway: {:?}", fractional_gateway);
    eprintln!("  reserves: {:?}", reserves);
    
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    client.define_currency(&currency_definition, fractional_gateway, reserves)
        .await
        .map(|r| serde_json::to_value(r).unwrap())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn send_raw_transaction(
    hex_data: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    client.send_raw_transaction(&hex_data, None)
        .await
        .map_err(|e| e.to_string())
}

// Private address commands already exist as z_list_addresses
// New address generation commands
#[tauri::command]
pub async fn get_new_address(
    account: Option<String>,
    chain: Option<String>,
    state: State<'_, AppState>
) -> Result<String, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    client.get_new_address(account.as_deref(), chain.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_addresses_by_account(
    account: String,
    chain: Option<String>,
    state: State<'_, AppState>
) -> Result<Vec<String>, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    client.get_addresses_by_account(&account, chain.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn z_get_new_address(
    address_type: Option<String>,
    chain: Option<String>,
    state: State<'_, AppState>
) -> Result<String, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    client.z_get_new_address(address_type.as_deref(), chain.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn z_list_addresses(
    include_watchonly: Option<bool>,
    chain: Option<String>,
    state: State<'_, AppState>
) -> Result<Vec<String>, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;
    
    client.z_list_addresses(include_watchonly, chain.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn z_get_operation_status(
    operation_ids: Option<Vec<String>>,
    chain: Option<String>,
    state: State<'_, AppState>
) -> Result<Value, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;

    client.z_get_operation_status(operation_ids, chain.as_deref())
        .await
        .map(|r| serde_json::to_value(r).unwrap())
        .map_err(|e| e.to_string())
}

// vlotto Commands

/// Send vlotto ticket to graveyard address
/// This transfers ticket ownership to the graveyard, making it eligible for unsold ticket handling
#[tauri::command]
pub async fn send_ticket_to_graveyard(
    ticket_identity_address: String,
    source_of_funds: Option<String>,
    chain: Option<String>,
    state: State<'_, AppState>
) -> Result<Value, String> {
    let client_guard = state.active_client.read().await;
    let client = client_guard.as_ref().ok_or("No active RPC connection")?;

    // Step 1: Get current ticket identity using identity address
    let identity_result = client.get_identity(
        &ticket_identity_address,
        None,
        None,
        None,
        chain.as_deref()
    ).await.map_err(|e| e.to_string())?;

    // Extract identity details
    let identity_obj = identity_result.get("identity")
        .ok_or("Identity object not found in response")?;

    let name = identity_obj.get("name")
        .and_then(|v| v.as_str())
        .ok_or("Missing identity name")?
        .to_string();

    let parent = identity_obj.get("parent")
        .and_then(|v| v.as_str())
        .ok_or("Missing parent")?
        .to_string();

    let identity_address = identity_obj.get("identityaddress")
        .and_then(|v| v.as_str())
        .ok_or("Missing identity address")?
        .to_string();

    let contentmultimap = identity_obj.get("contentmultimap")
        .cloned();

    // Step 2: Determine graveyard address based on chain
    let graveyard_address = if let Some(chain_str) = chain.as_ref() {
        if chain_str.to_lowercase() == "vrsctest" {
            "RMzd5vMptsxxz1tWH2FeSdUgRSNgS4G52w"
        } else {
            "RAXCjm9Z4RJWEmsNgo83B8JevTcJRt6Tj5"
        }
    } else {
        // Default to testnet
        "RMzd5vMptsxxz1tWH2FeSdUgRSNgS4G52w"
    };

    // Step 3: Build identity JSON for updateidentity RPC
    let mut identity_json = serde_json::json!({
        "name": name,
        "parent": parent,
        "primaryaddresses": vec![graveyard_address],
        "minimumsignatures": 1,
        "revocationauthority": identity_address.clone(),
        "recoveryauthority": identity_address
    });

    // Add contentmultimap if present
    if let Some(cm) = contentmultimap {
        identity_json["contentmultimap"] = cm;
    }

    // Step 4: Build params array with correct parameter order
    // updateidentity params: [identity_json, return_tx, token_update, fee_offer, source_of_funds]
    let mut params = vec![identity_json];

    // Always add return_tx and token_update
    params.push(json!(false)); // return_tx
    params.push(json!(false)); // token_update

    // Add fee_offer - must be 0.0001 if source_of_funds is provided, empty string otherwise
    if source_of_funds.is_some() {
        params.push(json!(0.0001));
    } else {
        params.push(json!(""));
    }

    // Add source_of_funds if provided
    if let Some(source) = source_of_funds {
        params.push(json!(source));
    }

    // Step 5: Make direct RPC call to updateidentity
    client.call("updateidentity", json!(params))
        .await
        .map_err(|e| e.to_string())
}