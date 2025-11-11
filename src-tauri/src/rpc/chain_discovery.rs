// Chain Discovery and Configuration Management
use crate::rpc::{RpcCredentials, ChainConfig, RpcError};
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use serde_json::json;

/// Discovers all available Verus chains by scanning configuration directories
pub struct ChainDiscovery {
    pub discovered_chains: Vec<ChainConfig>,
}

impl ChainDiscovery {
    pub fn new() -> Self {
        Self {
            discovered_chains: Vec::new(),
        }
    }

    /// Discover all chains by scanning both Komodo and .verus directories
    pub async fn discover_chains(&mut self) -> Result<Vec<ChainConfig>, RpcError> {
        let mut chains = Vec::new();

        // Step 1: Scan Komodo directory for main chains (VRSC, VRSCTEST)
        if let Ok(komodo_chains) = self.scan_komodo_directory().await {
            chains.extend(komodo_chains);
        }

        // Step 2: Scan .verus directory for all PBaaS chains
        if let Ok(mut verus_chains) = self.scan_verus_directory().await {
            // Step 3: Resolve friendly names via RPC (or use fallback)
            let _ = self.resolve_pbaas_names(&mut verus_chains).await;
            chains.extend(verus_chains);
        }

        self.discovered_chains = chains.clone();
        Ok(chains)
    }

    /// Scan Komodo directory for main chain configurations (OS-aware)
    async fn scan_komodo_directory(&self) -> Result<Vec<ChainConfig>, RpcError> {
        let mut chains = Vec::new();
        let komodo_dir = get_data_directory()?;

        if !komodo_dir.exists() {
            return Ok(chains);
        }

        // Check for VRSC (main chain)
        let vrsc_config = komodo_dir.join("VRSC").join("VRSC.conf");
        if vrsc_config.exists() {
            eprintln!("FOUND VRSC CONFIG: {:?}", vrsc_config);
            if let Ok(credentials) = parse_config_file(&vrsc_config) {
                eprintln!("PARSED VRSC CREDENTIALS: {}:{}", credentials.host, credentials.port);
                chains.push(ChainConfig {
                    name: "vrsc".to_string(),
                    display_name: "VRSC".to_string(),
                    credentials,
                    is_active: false, // Will be tested later
                });
            } else {
                eprintln!("FAILED TO PARSE VRSC CONFIG: {:?}", vrsc_config);
            }
        } else {
            eprintln!("VRSC CONFIG NOT FOUND: {:?}", vrsc_config);
        }

        // Check for VRSCTEST (testnet)
        let vrsctest_config = komodo_dir.join("vrsctest").join("vrsctest.conf");
        if vrsctest_config.exists() {
            eprintln!("FOUND VRSCTEST CONFIG: {:?}", vrsctest_config);
            if let Ok(credentials) = parse_config_file(&vrsctest_config) {
                eprintln!("PARSED VRSCTEST CREDENTIALS: {}:{}", credentials.host, credentials.port);
                chains.push(ChainConfig {
                    name: "vrsctest".to_string(),
                    display_name: "VRSCTEST".to_string(),
                    credentials,
                    is_active: false,
                });
            } else {
                eprintln!("FAILED TO PARSE VRSCTEST CONFIG: {:?}", vrsctest_config);
            }
        } else {
            eprintln!("VRSCTEST CONFIG NOT FOUND: {:?}", vrsctest_config);
        }

        Ok(chains)
    }

    /// Scan .verus directory for PBaaS chain configurations (OS-aware)
    async fn scan_verus_directory(&self) -> Result<Vec<ChainConfig>, RpcError> {
        let mut chains = Vec::new();
        let verus_dir = get_verus_data_directory()?;

        if !verus_dir.exists() {
            return Ok(chains);
        }

        // Read all subdirectories (each represents a PBaaS chain by currencyidhex)
        if let Ok(entries) = fs::read_dir(&verus_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let currencyidhex = entry.file_name().to_string_lossy().to_string();
                    let config_file = entry.path().join(format!("{}.conf", currencyidhex));

                    if config_file.exists() {
                        if let Ok(credentials) = parse_config_file(&config_file) {
                            // Accept ALL PBaaS chains - no filtering
                            // Initially use currencyidhex as display_name, will be enriched later
                            chains.push(ChainConfig {
                                name: currencyidhex.clone(),
                                display_name: currencyidhex,
                                credentials,
                                is_active: false,
                            });
                        }
                    }
                }
            }
        }

        Ok(chains)
    }

    /// Resolve friendly names for PBaaS chains via VRSC mainnet RPC
    async fn resolve_pbaas_names(&self, chains: &mut [ChainConfig]) -> Result<(), RpcError> {
        // Try to get VRSC mainnet credentials from discovered chains
        let vrsc_credentials = match self.get_vrsc_credentials() {
            Ok(creds) => creds,
            Err(_) => {
                eprintln!("VRSC mainnet credentials not found, using fallback names");
                apply_fallback_names(chains);
                return Ok(());
            }
        };

        // Try to connect to VRSC mainnet
        let client = match crate::rpc::VerusRpcClient::new(vrsc_credentials) {
            Ok(c) => c,
            Err(_) => {
                eprintln!("Failed to create VRSC RPC client, using fallback names");
                apply_fallback_names(chains);
                return Ok(());
            }
        };

        // Call listcurrencies on VRSC mainnet to get PBaaS chain names
        match client.call::<serde_json::Value>("listcurrencies", json!([{"systemtype": "pbaas"}])).await {
            Ok(result) => {
                eprintln!("Successfully called listcurrencies on VRSC mainnet");

                // Build currencyidhex -> name lookup map
                let mut name_map = HashMap::new();
                if let Some(currency_list) = result.as_array() {
                    for currency in currency_list {
                        if let Some(def) = currency.get("currencydefinition") {
                            let currencyidhex = def.get("currencyidhex")
                                .and_then(|h| h.as_str());
                            let name = def.get("name")
                                .and_then(|n| n.as_str());

                            if let (Some(hex), Some(n)) = (currencyidhex, name) {
                                name_map.insert(hex.to_string(), n.to_string());
                                eprintln!("Mapped {} -> {}", hex, n);
                            }
                        }
                    }
                }

                // Apply friendly names from RPC response
                for chain in chains.iter_mut() {
                    if let Some(friendly_name) = name_map.get(&chain.name) {
                        chain.display_name = friendly_name.clone();
                        eprintln!("Set display_name for {} to {}", chain.name, friendly_name);
                    } else {
                        // Try fallback for chains not in RPC response
                        if let Some(fallback_name) = get_fallback_display_name(&chain.name) {
                            chain.display_name = fallback_name.to_string();
                            eprintln!("Used fallback display_name for {}: {}", chain.name, fallback_name);
                        }
                        // Otherwise keep currencyidhex as display_name
                    }
                }
            }
            Err(e) => {
                eprintln!("listcurrencies RPC call failed: {}, using fallback names", e);
                apply_fallback_names(chains);
            }
        }

        Ok(())
    }

    /// Get VRSC mainnet credentials from discovered chains
    fn get_vrsc_credentials(&self) -> Result<RpcCredentials, RpcError> {
        self.discovered_chains
            .iter()
            .find(|c| c.name == "vrsc")
            .map(|c| c.credentials.clone())
            .ok_or_else(|| RpcError::Configuration("VRSC mainnet not found".to_string()))
    }

    /// Test connectivity for all discovered chains
    pub async fn test_chain_connectivity(&mut self) -> Result<(), RpcError> {
        for chain in self.discovered_chains.iter_mut() {
            let connection_result = test_chain_connection(&chain.credentials).await;
            chain.is_active = connection_result.is_ok();
        }

        Ok(())
    }

    /// Get active (connected) chains only
    pub fn get_active_chains(&self) -> Vec<&ChainConfig> {
        self.discovered_chains.iter().filter(|c| c.is_active).collect()
    }
}

/// Parse a Verus configuration file to extract RPC credentials
fn parse_config_file(config_path: &Path) -> Result<RpcCredentials, RpcError> {
    let content = fs::read_to_string(config_path)
        .map_err(|e| RpcError::Configuration(format!("Failed to read config file: {}", e)))?;

    let mut username = None;
    let mut password = None;
    let mut rpc_port = None;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("rpcuser=") {
            username = Some(line.strip_prefix("rpcuser=").unwrap().to_string());
        } else if line.starts_with("rpcpassword=") {
            password = Some(line.strip_prefix("rpcpassword=").unwrap().to_string());
        } else if line.starts_with("rpcport=") {
            if let Ok(port) = line.strip_prefix("rpcport=").unwrap().parse::<u16>() {
                rpc_port = Some(port);
            }
        }
    }

    let username = username.ok_or_else(|| {
        RpcError::Configuration("rpcuser not found in config file".to_string())
    })?;
    
    let password = password.ok_or_else(|| {
        RpcError::Configuration("rpcpassword not found in config file".to_string())
    })?;

    // Determine chain type from config path and use appropriate default port
    let port = rpc_port.unwrap_or_else(|| {
        if let Some(filename) = config_path.file_name().and_then(|f| f.to_str()) {
            match filename {
                "VRSC.conf" => ChainConfig::default_port("vrsc"),
                "vrsctest.conf" => ChainConfig::default_port("vrsctest"),
                _ => {
                    // For PBaaS chains, ports are dynamic and should be in config
                    // If not specified, fallback to VRSC default
                    27486
                }
            }
        } else {
            27486 // Fallback to VRSC default
        }
    });

    Ok(RpcCredentials {
        username,
        password,
        host: "127.0.0.1".to_string(), // Always localhost for config files
        port,
    })
}

/// Get the correct data directory based on OS
fn get_data_directory() -> Result<PathBuf, RpcError> {
    #[cfg(target_os = "windows")]
    {
        dirs::data_dir()
            .map(|dir| dir.join("Komodo"))
            .ok_or_else(|| RpcError::Configuration("Could not determine Windows AppData directory".to_string()))
    }
    
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .map(|dir| dir.join("Library").join("Application Support").join("Komodo"))
            .ok_or_else(|| RpcError::Configuration("Could not determine macOS Application Support directory".to_string()))
    }
    
    #[cfg(target_os = "linux")]
    {
        dirs::home_dir()
            .map(|dir| dir.join(".komodo"))
            .ok_or_else(|| RpcError::Configuration("Could not determine Linux home directory".to_string()))
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err(RpcError::Configuration("Unsupported operating system".to_string()))
    }
}

/// Get the Verus-specific data directory (Verus/pbaas or .verus/pbaas folder)
fn get_verus_data_directory() -> Result<PathBuf, RpcError> {
    #[cfg(target_os = "windows")]
    {
        dirs::data_dir()
            .map(|dir| dir.join("Verus").join("pbaas"))
            .ok_or_else(|| RpcError::Configuration("Could not determine Windows AppData directory".to_string()))
    }

    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .map(|dir| dir.join("Library").join("Application Support").join("Verus").join("pbaas"))
            .ok_or_else(|| RpcError::Configuration("Could not determine macOS Application Support directory".to_string()))
    }

    #[cfg(target_os = "linux")]
    {
        dirs::home_dir()
            .map(|dir| dir.join(".verus").join("pbaas"))
            .ok_or_else(|| RpcError::Configuration("Could not determine Linux home directory".to_string()))
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err(RpcError::Configuration("Unsupported operating system".to_string()))
    }
}

/// Apply fallback names to PBaaS chains when RPC is unavailable
fn apply_fallback_names(chains: &mut [ChainConfig]) {
    for chain in chains.iter_mut() {
        if let Some(fallback_name) = get_fallback_display_name(&chain.name) {
            chain.display_name = fallback_name.to_string();
        }
        // Otherwise keep currencyidhex as display_name
    }
}

/// Get fallback display name for known mainnet PBaaS chains
/// This is used ONLY when VRSC mainnet is offline or RPC fails
fn get_fallback_display_name(currencyidhex: &str) -> Option<&'static str> {
    // ONLY mainnet PBaaS chains - NO testnet chains
    match currencyidhex {
        "e9e10955b7d16031e3d6f55d9c908a038e3ae47d" => Some("VARRR"),
        "53fe39eea8c06bba32f1a4e20db67e5524f0309d" => Some("VDEX"),
        "f315367528394674d45277e369629605a1c3ce9f" => Some("CHIPS"),
        _ => None,
    }
}

/// Test connection to a specific chain
async fn test_chain_connection(credentials: &RpcCredentials) -> Result<(), RpcError> {
    use crate::rpc::VerusRpcClient;
    
    let client = VerusRpcClient::new(credentials.clone())?;
    client.test_connection().await.map(|_| ())
}

/// Chain configuration utilities
impl ChainConfig {
    /// Get the default RPC port for a chain name
    pub fn default_port(chain_name: &str) -> u16 {
        match chain_name {
            "vrsc" => 27486,
            "vrsctest" => 18843,
            // PBaaS chains have dynamic ports defined in their config files
            _ => 27486, // Fallback to VRSC default
        }
    }

    /// Get the configuration directory path for a chain (OS-aware)
    pub fn config_directory(chain_name: &str) -> Result<PathBuf, RpcError> {
        match chain_name {
            "vrsc" => {
                let data_dir = get_data_directory()?;
                Ok(data_dir.join("VRSC"))
            }
            "vrsctest" => {
                let data_dir = get_data_directory()?;
                Ok(data_dir.join("vrsctest"))
            }
            _ => {
                // For PBaaS chains, use currencyidhex as directory name in .verus
                let verus_dir = get_verus_data_directory()?;
                Ok(verus_dir.join(chain_name))
            }
        }
    }

    /// Get human-readable expected paths for debugging
    pub fn get_expected_paths() -> String {
        #[cfg(target_os = "windows")]
        {
            "%AppData%\\Roaming\\Komodo\\VRSC\\VRSC.conf\n%AppData%\\Roaming\\Komodo\\vrsctest\\vrsctest.conf\n%AppData%\\Roaming\\.verus\\{currencyidhex}\\{currencyidhex}.conf".to_string()
        }

        #[cfg(target_os = "macos")]
        {
            "~/Library/Application Support/Komodo/VRSC/VRSC.conf\n~/Library/Application Support/Komodo/vrsctest/vrsctest.conf\n~/Library/Application Support/.verus/{currencyidhex}/{currencyidhex}.conf".to_string()
        }

        #[cfg(target_os = "linux")]
        {
            "~/.komodo/VRSC/VRSC.conf\n~/.komodo/vrsctest/vrsctest.conf\n~/.verus/{currencyidhex}/{currencyidhex}.conf".to_string()
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            "Unsupported operating system".to_string()
        }
    }
}