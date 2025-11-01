// Identity Management RPC Commands
use crate::rpc::{
    VerusRpcClient, RpcError, Identity, RegisterNameCommitmentResponse,
    IdentityRegistration, IdentityUpdate
};
use serde_json::{json, Value};

impl VerusRpcClient {
    /// List all identities
    pub async fn list_identities(
        &self, 
        include_can_spend: Option<bool>,
        include_can_sign: Option<bool>,
        include_watch_only: Option<bool>,
        chain: Option<&str>
    ) -> Result<Value, RpcError> {
        let mut params = Vec::new();
        
        if let Some(spend) = include_can_spend {
            params.push(json!(spend));
            if let Some(sign) = include_can_sign {
                params.push(json!(sign));
                if let Some(watch) = include_watch_only {
                    params.push(json!(watch));
                }
            }
        }
        self.call_with_chain("listidentities", json!(params), chain).await
    }

    /// Get specific identity details
    pub async fn get_identity(
        &self,
        name: &str,
        height: Option<u64>,
        tx_proof: Option<bool>,
        tx_proof_height: Option<u64>,
        chain: Option<&str>
    ) -> Result<Value, RpcError> {
        let mut params = vec![];

        params.push(json!(name));

        if let Some(h) = height {
            params.push(json!(h));
            if let Some(proof) = tx_proof {
                params.push(json!(proof));
                if let Some(proof_height) = tx_proof_height {
                    params.push(json!(proof_height));
                }
            }
        }
        self.call_with_chain("getidentity", json!(params), chain).await
    }

    /// Get identity content with full contentmultimap history
    pub async fn get_identity_content(
        &self,
        name: &str,
        heightstart: Option<u64>,
        heightend: Option<u64>,
        txproofs: Option<bool>,
        vdxfkey: Option<&str>,
        chain: Option<&str>
    ) -> Result<Value, RpcError> {
        let mut params = vec![json!(name)];

        // Add optional parameters in order
        if let Some(start) = heightstart {
            params.push(json!(start));
            if let Some(end) = heightend {
                params.push(json!(end));
                if let Some(proofs) = txproofs {
                    params.push(json!(proofs));
                    if let Some(key) = vdxfkey {
                        params.push(json!(key));
                    }
                }
            }
        }

        self.call_with_chain("getidentitycontent", json!(params), chain).await
    }

    /// Reserve identity name (step 1 of registration)
    pub async fn register_name_commitment(
        &self,
        name: &str,
        control_address: &str,
        referral: Option<&str>
    ) -> Result<RegisterNameCommitmentResponse, RpcError> {
        let mut params = vec![json!(name), json!(control_address)];
        if let Some(ref_addr) = referral {
            params.push(json!(ref_addr));
        }
        self.call("registernamecommitment", json!(params)).await
    }

    /// Complete identity registration (step 2)
    pub async fn register_identity(
        &self,
        identity_registration: &IdentityRegistration,
        return_tx: Option<bool>,
        fee_offer: Option<Value>
    ) -> Result<Value, RpcError> {
        let mut params = vec![json!(identity_registration)];
        if let Some(ret_tx) = return_tx {
            params.push(json!(ret_tx));
            if let Some(fee) = fee_offer {
                params.push(fee);
            }
        }
        self.call("registeridentity", json!(params)).await
    }

    /// Update identity properties
    pub async fn update_identity(
        &self,
        from_address: &str,
        identity_update: &IdentityUpdate,
        return_tx: Option<bool>,
        fee_offer: Option<Value>
    ) -> Result<Value, RpcError> {
        let mut params = vec![json!(from_address), json!(identity_update)];
        if let Some(ret_tx) = return_tx {
            params.push(json!(ret_tx));
            if let Some(fee) = fee_offer {
                params.push(fee);
            }
        }
        self.call("updateidentity", json!(params)).await
    }

    /// Apply timelock protection to identity
    pub async fn set_identity_timelock(
        &self,
        identity: &str,
        timelock_params: &Value,
        return_tx: Option<bool>,
        fee_offer: Option<f64>,
        source_of_funds: Option<&str>
    ) -> Result<Value, RpcError> {
        let mut params = vec![json!(identity), timelock_params.clone()];
        
        // Add optional parameters in the correct order
        if let Some(ret_tx) = return_tx {
            params.push(json!(ret_tx));
            if let Some(fee) = fee_offer {
                params.push(json!(fee));
                if let Some(source) = source_of_funds {
                    params.push(json!(source));
                }
            }
        }
        
        self.call("setidentitytimelock", json!(params)).await
    }

    /// Revoke identity access
    pub async fn revoke_identity(
        &self,
        name_or_id: &str,
        return_tx: Option<bool>,
        token_revoke: Option<bool>,
        fee_offer: Option<f64>,
        source_of_funds: Option<&str>
    ) -> Result<Value, RpcError> {
        let mut params = vec![json!(name_or_id)];
        
        // Add optional parameters in the correct order
        if let Some(ret_tx) = return_tx {
            params.push(json!(ret_tx));
            if let Some(token) = token_revoke {
                params.push(json!(token));
                if let Some(fee) = fee_offer {
                    params.push(json!(fee));
                    if let Some(source) = source_of_funds {
                        params.push(json!(source));
                    }
                }
            }
        }
        
        self.call("revokeidentity", json!(params)).await
    }

    /// Recover compromised identity
    pub async fn recover_identity(
        &self,
        json_identity: &Value,
        return_tx: Option<bool>,
        token_recover: Option<bool>,
        fee_offer: Option<f64>,
        source_of_funds: Option<&str>
    ) -> Result<Value, RpcError> {
        let mut params = vec![json_identity.clone()];

        // Add optional parameters in the correct order
        if let Some(ret_tx) = return_tx {
            params.push(json!(ret_tx));
            if let Some(token) = token_recover {
                params.push(json!(token));
                if let Some(fee) = fee_offer {
                    params.push(json!(fee));
                    if let Some(source) = source_of_funds {
                        params.push(json!(source));
                    }
                }
            }
        }

        self.call("recoveridentity", json!(params)).await
    }

    /// Verify a signed message
    ///
    /// Verifies that a message was signed by a specific address or identity.
    /// Used for ticket authenticity verification in vlotto.
    pub async fn verify_message(
        &self,
        taddr_or_identity: &str,
        signature: &str,
        message: &str,
        checklatest: Option<bool>,
        chain: Option<&str>
    ) -> Result<bool, RpcError> {
        let mut params = vec![
            json!(taddr_or_identity),
            json!(signature),
            json!(message)
        ];

        if let Some(check) = checklatest {
            params.push(json!(check));
        }

        let result: Value = self.call_with_chain("verifymessage", json!(params), chain).await?;

        // Result should be a boolean
        result.as_bool()
            .ok_or(RpcError::InvalidResponse)
    }

    /// Verify a signed hash
    ///
    /// Verifies that a hash was signed by a specific address or identity.
    /// Used for ticket authenticity verification in vlotto.
    pub async fn verify_hash(
        &self,
        taddr_or_identity: &str,
        signature: &str,
        hexhash: &str,
        checklatest: Option<bool>,
        chain: Option<&str>
    ) -> Result<bool, RpcError> {
        let mut params = vec![
            json!(taddr_or_identity),
            json!(signature),
            json!(hexhash)
        ];

        if let Some(check) = checklatest {
            params.push(json!(check));
        }

        let result: Value = self.call_with_chain("verifyhash", json!(params), chain).await?;

        // Result should be a boolean
        result.as_bool()
            .ok_or(RpcError::InvalidResponse)
    }
}