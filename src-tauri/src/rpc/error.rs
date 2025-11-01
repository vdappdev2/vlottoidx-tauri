use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum RpcError {
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("RPC call failed: {0}")]
    RpcCall(String),
    
    #[error("JSON parsing error: {0}")]
    JsonParse(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Daemon offline or unreachable")]
    DaemonOffline,
    
    #[error("Invalid response from daemon")]
    InvalidResponse,
    
    #[error("Insufficient funds: {0}")]
    InsufficientFunds(String),
    
    #[error("Identity not found: {0}")]
    IdentityNotFound(String),
    
    #[error("Currency not found: {0}")]
    CurrencyNotFound(String),
    
    #[error("Chain sync in progress")]
    ChainSyncing,
    
    #[error("Wallet locked")]
    WalletLocked,
    
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    
    #[error("Offer not found: {0}")]
    OfferNotFound(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Validation error: {0}")]
    ValidationError(String),
}

impl From<reqwest::Error> for RpcError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_connect() {
            RpcError::DaemonOffline
        } else if err.is_timeout() {
            RpcError::Network("Request timeout".to_string())
        } else {
            RpcError::Network(err.to_string())
        }
    }
}

impl From<serde_json::Error> for RpcError {
    fn from(err: serde_json::Error) -> Self {
        RpcError::JsonParse(err.to_string())
    }
}

/// Enhanced error handling utilities
impl RpcError {
    /// Parse RPC error from daemon response and categorize it
    pub fn from_daemon_error(code: i32, message: &str) -> Self {
        match code {
            -1 => RpcError::DaemonOffline,
            -3 => RpcError::Authentication("Invalid RPC credentials".to_string()),
            -4 => RpcError::WalletLocked,
            -5 => RpcError::InvalidAddress(message.to_string()),
            -6 => RpcError::InsufficientFunds(message.to_string()),
            -8 => RpcError::InvalidAddress(message.to_string()),
            -13 => RpcError::WalletLocked,
            -14 => RpcError::WalletLocked,
            -15 => RpcError::InvalidAddress(message.to_string()),
            -17 => RpcError::ChainSyncing,
            -18 => RpcError::IdentityNotFound(message.to_string()),
            -19 => RpcError::CurrencyNotFound(message.to_string()),
            -20 => RpcError::OfferNotFound(message.to_string()),
            -21 => RpcError::PermissionDenied(message.to_string()),
            -22 => RpcError::ValidationError(message.to_string()),
            -25 => RpcError::TransactionFailed(message.to_string()),
            _ => RpcError::RpcCall(format!("Code {}: {}", code, message)),
        }
    }

    /// Get user-friendly error message for display
    pub fn user_friendly_message(&self) -> String {
        match self {
            RpcError::DaemonOffline => {
                "The Verus daemon is not running or unreachable. Please start your Verus wallet and try again.".to_string()
            }
            RpcError::Authentication(_) => {
                "Invalid RPC credentials. Please check your username and password in the configuration.".to_string()
            }
            RpcError::WalletLocked => {
                "Your wallet is locked. Please unlock it with your passphrase to perform this operation.".to_string()
            }
            RpcError::ChainSyncing => {
                "The blockchain is still syncing. Please wait for sync to complete before trying again.".to_string()
            }
            RpcError::InsufficientFunds(details) => {
                format!("Insufficient funds to complete this transaction. {}", details)
            }
            RpcError::InvalidAddress(addr) => {
                format!("The address '{}' is not valid. Please check the address format.", addr)
            }
            RpcError::IdentityNotFound(name) => {
                format!("Identity '{}' was not found. Please verify the identity name.", name)
            }
            RpcError::CurrencyNotFound(name) => {
                format!("Currency '{}' was not found. Please verify the currency name.", name)
            }
            RpcError::OfferNotFound(id) => {
                format!("Offer '{}' was not found or has expired.", id)
            }
            RpcError::PermissionDenied(details) => {
                format!("Permission denied: {}. You may not have the required authority.", details)
            }
            RpcError::TransactionFailed(details) => {
                format!("Transaction failed: {}. Please check your inputs and try again.", details)
            }
            RpcError::ValidationError(details) => {
                format!("Validation error: {}. Please check your inputs.", details)
            }
            RpcError::Network(details) => {
                format!("Network error: {}. Please check your internet connection.", details)
            }
            RpcError::Configuration(details) => {
                format!("Configuration error: {}. Please check your settings.", details)
            }
            RpcError::Connection(details) => {
                format!("Connection error: {}. Please verify your daemon is running.", details)
            }
            RpcError::RateLimitExceeded => {
                "Too many requests. Please wait a moment before trying again.".to_string()
            }
            _ => {
                "An unexpected error occurred. Please try again or contact support.".to_string()
            }
        }
    }

    /// Get suggested resolution steps
    pub fn resolution_steps(&self) -> Vec<String> {
        match self {
            RpcError::DaemonOffline => vec![
                "Start your Verus wallet application".to_string(),
                "Wait for the daemon to fully load".to_string(),
                "Check that RPC is enabled in your configuration".to_string(),
            ],
            RpcError::Authentication(_) => vec![
                "Check your RPC username and password".to_string(),
                "Verify your wallet configuration file".to_string(),
                "Restart the wallet if credentials were recently changed".to_string(),
            ],
            RpcError::WalletLocked => vec![
                "Unlock your wallet with the passphrase".to_string(),
                "Consider setting up automatic unlocking for staking".to_string(),
            ],
            RpcError::ChainSyncing => vec![
                "Wait for blockchain synchronization to complete".to_string(),
                "Check sync progress in your wallet".to_string(),
                "Ensure stable internet connection".to_string(),
            ],
            RpcError::InsufficientFunds(_) => vec![
                "Check your available balance".to_string(),
                "Wait for pending transactions to confirm".to_string(),
                "Consider reducing transaction amount or fees".to_string(),
            ],
            RpcError::InvalidAddress(_) => vec![
                "Double-check the address format".to_string(),
                "Copy address from a reliable source".to_string(),
                "Verify you're using the correct chain address".to_string(),
            ],
            _ => vec![
                "Check your internet connection".to_string(),
                "Verify daemon is running properly".to_string(),
                "Try the operation again in a few moments".to_string(),
            ],
        }
    }

    /// Check if this is a recoverable error
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            RpcError::Network(_) | 
            RpcError::DaemonOffline | 
            RpcError::ChainSyncing |
            RpcError::RateLimitExceeded |
            RpcError::WalletLocked
        )
    }

    /// Check if this error suggests the user should retry
    pub fn should_retry(&self) -> bool {
        matches!(
            self,
            RpcError::Network(_) | 
            RpcError::DaemonOffline | 
            RpcError::ChainSyncing |
            RpcError::RateLimitExceeded
        )
    }
}