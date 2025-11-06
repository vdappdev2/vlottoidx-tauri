// Chain Discovery and Configuration Management
use crate::rpc::{RpcCredentials, ChainConfig, SupportedChain, RpcError};
use std::path::{Path, PathBuf};
use std::fs;

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

        // Scan Komodo directory for main chains
        if let Ok(komodo_chains) = self.scan_komodo_directory().await {
            chains.extend(komodo_chains);
        }

        // Scan .verus directory for PBaaS chains
        if let Ok(verus_chains) = self.scan_verus_directory().await {
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
                    name: SupportedChain::Vrsc,
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
                    name: SupportedChain::VrscTest,
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

        // Read all subdirectories (each represents a PBaaS chain by hash)
        if let Ok(entries) = fs::read_dir(&verus_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let chain_hash = entry.file_name().to_string_lossy().to_string();
                    let config_file = entry.path().join(format!("{}.conf", chain_hash));
                    
                    if config_file.exists() {
                        if let Ok(credentials) = parse_config_file(&config_file) {
                            // Try to determine which supported chain this is
                            if let Some(chain_type) = identify_pbaas_chain(&chain_hash) {
                                chains.push(ChainConfig {
                                    name: chain_type,
                                    credentials,
                                    is_active: false,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(chains)
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
                "VRSC.conf" => ChainConfig::default_port(&SupportedChain::Vrsc),
                "vrsctest.conf" => ChainConfig::default_port(&SupportedChain::VrscTest),
                _ => {
                    // For PBaaS chains, try to identify by parent directory name (currencyidhex)
                    if let Some(parent) = config_path.parent() {
                        if let Some(chain_hash) = parent.file_name().and_then(|f| f.to_str()) {
                            if let Some(chain_type) = identify_pbaas_chain(chain_hash) {
                                ChainConfig::default_port(&chain_type)
                            } else {
                                27486 // Fallback to VRSC default
                            }
                        } else {
                            27486 // Fallback to VRSC default
                        }
                    } else {
                        27486 // Fallback to VRSC default
                    }
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

/// Get the Verus-specific data directory (.verus folder)
fn get_verus_data_directory() -> Result<PathBuf, RpcError> {
    #[cfg(target_os = "windows")]
    {
        dirs::data_dir()
            .map(|dir| dir.join(".verus"))
            .ok_or_else(|| RpcError::Configuration("Could not determine Windows AppData directory".to_string()))
    }
    
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .map(|dir| dir.join("Library").join("Application Support").join(".verus"))
            .ok_or_else(|| RpcError::Configuration("Could not determine macOS Application Support directory".to_string()))
    }
    
    #[cfg(target_os = "linux")]
    {
        dirs::home_dir()
            .map(|dir| dir.join(".verus"))
            .ok_or_else(|| RpcError::Configuration("Could not determine Linux home directory".to_string()))
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err(RpcError::Configuration("Unsupported operating system".to_string()))
    }
}

/// Identify which supported PBaaS chain a hash corresponds to
fn identify_pbaas_chain(chain_hash: &str) -> Option<SupportedChain> {
    // Use actual currencyidhex values from the Verus network
    match chain_hash {
        "e9e10955b7d16031e3d6f55d9c908a038e3ae47d" => Some(SupportedChain::Varrr),
        "53fe39eea8c06bba32f1a4e20db67e5524f0309d" => Some(SupportedChain::Vdex),
        "f315367528394674d45277e369629605a1c3ce9f" => Some(SupportedChain::Chips),
        _ => None, // Unknown PBaaS chain
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
    /// Get the default RPC port for a chain type
    pub fn default_port(chain: &SupportedChain) -> u16 {
        match chain {
            SupportedChain::Vrsc => 27486,
            SupportedChain::VrscTest => 18843, 
            SupportedChain::Varrr => 9333,    // Example port
            SupportedChain::Vdex => 9334,     // Example port  
            SupportedChain::Chips => 9335,    // Example port
        }
    }

    /// Get the configuration directory path for a chain (OS-aware)
    pub fn config_directory(chain: &SupportedChain) -> Result<PathBuf, RpcError> {
        let data_dir = get_data_directory()?;
        
        let path = match chain {
            SupportedChain::Vrsc => data_dir.join("VRSC"),
            SupportedChain::VrscTest => data_dir.join("vrsctest"),
            // PBaaS chains would need their specific hash paths
            _ => return Err(RpcError::Configuration(
                "PBaaS chain config paths need specific hash".to_string()
            )),
        };

        Ok(path)
    }

    /// Get human-readable expected paths for debugging
    pub fn get_expected_paths() -> String {
        #[cfg(target_os = "windows")]
        {
            "%AppData%\\Roaming\\Komodo\\VRSC\\vrsc.conf\n%AppData%\\Roaming\\Komodo\\vrsctest\\vrsctest.conf".to_string()
        }
        
        #[cfg(target_os = "macos")]
        {
            "~/Library/Application Support/Komodo/VRSC/vrsc.conf\n~/Library/Application Support/Komodo/vrsctest/vrsctest.conf".to_string()
        }
        
        #[cfg(target_os = "linux")]
        {
            "~/.komodo/VRSC/vrsc.conf\n~/.komodo/vrsctest/vrsctest.conf".to_string()
        }
        
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            "Unsupported operating system".to_string()
        }
    }
}