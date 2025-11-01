// Transaction RPC Commands
use crate::rpc::{
    VerusRpcClient, RpcError,
    SendCurrencyRequest
};
use serde_json::json;

impl VerusRpcClient {
    /// Master command for currency operations
    pub async fn send_currency(
        &self,
        from_address: &str,
        outputs: Vec<SendCurrencyRequest>,
        min_conf: Option<u32>,
        fee_amount: Option<f64>,
        subtract_fee_from_amount: Option<bool>
    ) -> Result<String, RpcError> {
        let mut params = vec![json!(from_address), json!(outputs)];
        
        if let Some(conf) = min_conf {
            params.push(json!(conf));
            if let Some(fee) = fee_amount {
                params.push(json!(fee));
                if let Some(subtract) = subtract_fee_from_amount {
                    params.push(json!(subtract));
                }
            }
        }
        
        // sendcurrency returns plain opid string, not JSON object
        let result: serde_json::Value = self.call("sendcurrency", json!(params)).await?;
        
        // Extract the string value (opid)
        result.as_str()
            .ok_or_else(|| RpcError::InvalidResponse)
            .map(|s| s.to_string())
    }

    /// Simple currency send
    pub async fn send_currency_simple(
        &self,
        from_address: &str,
        currency: &str,
        amount: f64,
        to_address: &str
    ) -> Result<String, RpcError> {
        let output = SendCurrencyRequest {
            currency: currency.to_string(),
            amount,
            address: to_address.to_string(),
            convertto: None,
            via: None,
            exportto: None,
            exportcurrency: None,
            exportid: None,
            preconvert: None,
            memo: None,
            feecurrency: None,
            addconversionfees: None,
        };
        
        self.send_currency(from_address, vec![output], None, None, None).await
    }

    /// Currency conversion
    pub async fn convert_currency(
        &self,
        from_address: &str,
        source_currency: &str,
        amount: f64,
        target_currency: &str,
        to_address: &str
    ) -> Result<String, RpcError> {
        let output = SendCurrencyRequest {
            currency: source_currency.to_string(),
            amount,
            address: to_address.to_string(),
            convertto: Some(target_currency.to_string()),
            via: None,
            exportto: None,
            exportcurrency: None,
            exportid: None,
            preconvert: None,
            memo: None,
            feecurrency: None,
            addconversionfees: None,
        };
        
        self.send_currency(from_address, vec![output], None, None, None).await
    }

    /// Cross-chain export
    pub async fn export_currency(
        &self,
        from_address: &str,
        currency: &str,
        amount: f64,
        destination_chain: &str,
        to_address: &str
    ) -> Result<String, RpcError> {
        let output = SendCurrencyRequest {
            currency: currency.to_string(),
            amount,
            address: to_address.to_string(),
            convertto: None,
            via: None,
            exportto: Some(destination_chain.to_string()),
            exportcurrency: None,
            exportid: None,
            preconvert: None,
            memo: None,
            feecurrency: None,
            addconversionfees: None,
        };
        
        self.send_currency(from_address, vec![output], None, None, None).await
    }

    /// Export currency definition
    pub async fn export_currency_definition(
        &self,
        from_address: &str,
        currency: &str,
        destination_chain: &str,
        to_address: &str
    ) -> Result<String, RpcError> {
        let output = SendCurrencyRequest {
            currency: currency.to_string(),
            amount: 0.0,
            address: to_address.to_string(),
            convertto: None,
            via: None,
            exportto: Some(destination_chain.to_string()),
            exportcurrency: Some(true),
            exportid: None,
            preconvert: None,
            memo: None,
            feecurrency: None,
            addconversionfees: None,
        };
        
        self.send_currency(from_address, vec![output], None, None, None).await
    }

    /// Export identity
    pub async fn export_identity(
        &self,
        from_address: &str,
        destination_chain: &str,
        to_address: &str
    ) -> Result<String, RpcError> {
        let output = SendCurrencyRequest {
            currency: "".to_string(), // Not used for identity export
            amount: 0.0,
            address: to_address.to_string(),
            convertto: None,
            via: None,
            exportto: Some(destination_chain.to_string()),
            exportcurrency: None,
            exportid: Some(true),
            preconvert: None,
            memo: None,
            feecurrency: None,
            addconversionfees: None,
        };
        
        self.send_currency(from_address, vec![output], None, None, None).await
    }

    /// Broadcast raw transaction
    pub async fn send_raw_transaction(
        &self,
        hex: &str,
        allow_high_fees: Option<bool>
    ) -> Result<String, RpcError> {
        let mut params = vec![json!(hex)];
        if let Some(allow_fees) = allow_high_fees {
            params.push(json!(allow_fees));
        }
        
        // sendrawtransaction returns plain txid string, not JSON object
        let result: serde_json::Value = self.call("sendrawtransaction", json!(params)).await?;
        
        // Extract the string value (txid)
        result.as_str()
            .ok_or_else(|| RpcError::InvalidResponse)
            .map(|s| s.to_string())
    }
}