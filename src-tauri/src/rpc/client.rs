use super::{RpcError, RpcCredentials, JsonRpcRequest, VerusJsonRpcResponse};
use reqwest::Client;
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct VerusRpcClient {
    credentials: RpcCredentials,
    client: Client,
    request_counter: AtomicU64,
}

impl VerusRpcClient {
    pub fn new(credentials: RpcCredentials) -> Result<Self, RpcError> {
        let client = Client::builder()
            .http1_only()
            .build()
            .map_err(RpcError::from)?;
            
        Ok(Self {
            credentials,
            client,
            request_counter: AtomicU64::new(1),
        })
    }
    
    pub async fn call<T>(&self, method: &str, params: Value) -> Result<T, RpcError>
    where
        T: serde::de::DeserializeOwned,
    {
        let request_id = format!("verusidx_{}", self.request_counter.fetch_add(1, Ordering::SeqCst));
        
        let rpc_request = JsonRpcRequest {
            jsonrpc: "1.0".to_string(),
            method: method.to_string(),
            params,
            id: request_id,
        };
        
        let url = format!("http://{}:{}", self.credentials.host, self.credentials.port);
        
        let request = self.client
            .post(&url)
            .basic_auth(&self.credentials.username, Some(&self.credentials.password))
            .header("Content-Type", "application/json")
            .json(&rpc_request)
            .build()
            .map_err(RpcError::from)?;
        
        let response = self.client
            .execute(request)
            .await?;
            
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(RpcError::RpcCall(format!("HTTP {}: {}", 
                status, 
                error_text
            )));
        }
        
        let response_text = response.text().await?;
        
        // Parse as Verus JSON-RPC response (missing jsonrpc field) - this is the expected format
        match serde_json::from_str::<VerusJsonRpcResponse<serde_json::Value>>(&response_text) {
            Ok(verus_wrapper) => {
                if let Some(error) = verus_wrapper.error {
                    return Err(RpcError::from_daemon_error(error.code, &error.message));
                }
                
                if let Some(result) = verus_wrapper.result {
                    serde_json::from_value::<T>(result)
                        .map_err(|e| RpcError::JsonParse(format!("Failed to deserialize result for {}: {}", method, e)))
                } else {
                    // Handle missing result field - some commands like closeoffers return null
                    // Try to deserialize null as the target type (works for Option<T> and ())
                    serde_json::from_value::<T>(serde_json::Value::Null)
                        .map_err(|_| RpcError::JsonParse(format!("No result field found for method: {}", method)))
                }
            }
            Err(_) => {
                // Final fallback: try to parse response_text directly as T
                serde_json::from_str(&response_text)
                    .map_err(|e| RpcError::JsonParse(e.to_string()))
            }
        }
    }
    
    pub async fn call_with_chain<T>(&self, method: &str, params: Value, _chain: Option<&str>) -> Result<T, RpcError>
    where
        T: serde::de::DeserializeOwned,
    {
        // Chain already determined by which daemon we're connected to
        // The chain parameter from frontend is redundant but harmless
        self.call(method, params).await
    }
    
    // Test connection
    pub async fn test_connection(&self) -> Result<bool, RpcError> {
        // Try a simple getinfo call to test connectivity
        let _response: serde_json::Value = self.call("getinfo", Value::Array(vec![])).await?;
        Ok(true)
    }
}