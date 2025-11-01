// Secure Credential Management using OS Keychain
use crate::rpc::{RpcCredentials, RpcError, SupportedChain};
use keyring::Entry;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct StoredCredentials {
    username: String,
    password: String,
    host: String,
    port: u16,
}

/// Manages secure storage and retrieval of RPC credentials
pub struct CredentialManager {
    service_name: String,
}

impl CredentialManager {
    pub fn new() -> Self {
        Self {
            service_name: "VerusIDX".to_string(),
        }
    }

    /// Store credentials securely in OS keychain
    pub fn store_credentials(
        &self, 
        chain: &SupportedChain, 
        credentials: &RpcCredentials
    ) -> Result<(), RpcError> {
        let account_name = self.get_account_name(chain);
        let entry = Entry::new(&self.service_name, &account_name)
            .map_err(|e| RpcError::Configuration(format!("Failed to create keyring entry: {}", e)))?;

        let stored_creds = StoredCredentials {
            username: credentials.username.clone(),
            password: credentials.password.clone(),
            host: credentials.host.clone(),
            port: credentials.port,
        };

        let credentials_json = serde_json::to_string(&stored_creds)
            .map_err(|e| RpcError::JsonParse(e.to_string()))?;

        entry.set_password(&credentials_json)
            .map_err(|e| RpcError::Configuration(format!("Failed to store credentials: {}", e)))?;

        Ok(())
    }

    /// Load credentials securely from OS keychain
    pub fn load_credentials(
        &self, 
        chain: &SupportedChain
    ) -> Result<RpcCredentials, RpcError> {
        let account_name = self.get_account_name(chain);
        let entry = Entry::new(&self.service_name, &account_name)
            .map_err(|e| RpcError::Configuration(format!("Failed to create keyring entry: {}", e)))?;

        let credentials_json = entry.get_password()
            .map_err(|e| RpcError::Configuration(format!("Failed to load credentials: {}", e)))?;

        let stored_creds: StoredCredentials = serde_json::from_str(&credentials_json)
            .map_err(|e| RpcError::JsonParse(e.to_string()))?;

        Ok(RpcCredentials {
            username: stored_creds.username,
            password: stored_creds.password,
            host: stored_creds.host,
            port: stored_creds.port,
        })
    }

    /// Clear stored credentials from OS keychain
    pub fn clear_credentials(
        &self, 
        chain: &SupportedChain
    ) -> Result<(), RpcError> {
        let account_name = self.get_account_name(chain);
        let entry = Entry::new(&self.service_name, &account_name)
            .map_err(|e| RpcError::Configuration(format!("Failed to create keyring entry: {}", e)))?;

        entry.delete_credential()
            .map_err(|e| RpcError::Configuration(format!("Failed to clear credentials: {}", e)))?;

        Ok(())
    }

    /// Check if credentials exist for a chain
    pub fn has_credentials(&self, chain: &SupportedChain) -> bool {
        let account_name = self.get_account_name(chain);
        if let Ok(entry) = Entry::new(&self.service_name, &account_name) {
            entry.get_password().is_ok()
        } else {
            false
        }
    }

    /// List all chains that have stored credentials
    pub fn list_stored_chains(&self) -> Vec<SupportedChain> {
        vec![
            SupportedChain::Vrsc,
            SupportedChain::VrscTest,
            SupportedChain::Varrr,
            SupportedChain::Vdex,
            SupportedChain::Chips,
        ]
        .into_iter()
        .filter(|chain| self.has_credentials(chain))
        .collect()
    }

    /// Temporarily store credentials in memory for session use
    /// This is used when credentials are loaded from config files
    pub fn cache_session_credentials(
        &self,
        _chain: &SupportedChain,
        credentials: &RpcCredentials
    ) -> Result<(), RpcError> {
        // For session-only storage, we could use a different approach
        // For now, we'll just validate the credentials format
        if credentials.username.is_empty() || credentials.password.is_empty() {
            return Err(RpcError::Configuration(
                "Invalid credentials: username and password required".to_string()
            ));
        }

        if credentials.port == 0 {
            return Err(RpcError::Configuration(
                "Invalid credentials: valid port required".to_string()
            ));
        }

        // In a real implementation, we might store these in a secure in-memory cache
        // For now, we'll just validate them
        Ok(())
    }

    fn get_account_name(&self, chain: &SupportedChain) -> String {
        format!("verusidx-{}", chain.to_string())
    }
}

/// Credential validation utilities
impl CredentialManager {
    /// Validate credential format
    pub fn validate_credentials(credentials: &RpcCredentials) -> Result<(), RpcError> {
        if credentials.username.trim().is_empty() {
            return Err(RpcError::Configuration("Username cannot be empty".to_string()));
        }

        if credentials.password.trim().is_empty() {
            return Err(RpcError::Configuration("Password cannot be empty".to_string()));
        }

        if credentials.host.trim().is_empty() {
            return Err(RpcError::Configuration("Host cannot be empty".to_string()));
        }

        if credentials.port == 0 || credentials.port > 65535 {
            return Err(RpcError::Configuration("Port must be between 1 and 65535".to_string()));
        }

        Ok(())
    }

    /// Sanitize credentials for logging (remove sensitive data)
    pub fn sanitize_for_logging(credentials: &RpcCredentials) -> String {
        format!(
            "RpcCredentials {{ username: {}, password: [REDACTED], host: {}, port: {} }}",
            credentials.username,
            credentials.host,
            credentials.port
        )
    }
}

/// Default implementation
impl Default for CredentialManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_credentials() {
        let valid_creds = RpcCredentials {
            username: "testuser".to_string(),
            password: "testpass".to_string(),
            host: "127.0.0.1".to_string(),
            port: 8332,
        };

        assert!(CredentialManager::validate_credentials(&valid_creds).is_ok());

        let invalid_creds = RpcCredentials {
            username: "".to_string(),
            password: "testpass".to_string(),
            host: "127.0.0.1".to_string(),
            port: 8332,
        };

        assert!(CredentialManager::validate_credentials(&invalid_creds).is_err());
    }

    #[test]
    fn test_sanitize_for_logging() {
        let creds = RpcCredentials {
            username: "testuser".to_string(),
            password: "secret123".to_string(),
            host: "127.0.0.1".to_string(),
            port: 8332,
        };

        let sanitized = CredentialManager::sanitize_for_logging(&creds);
        assert!(sanitized.contains("testuser"));
        assert!(sanitized.contains("127.0.0.1"));
        assert!(sanitized.contains("8332"));
        assert!(!sanitized.contains("secret123"));
        assert!(sanitized.contains("[REDACTED]"));
    }
}