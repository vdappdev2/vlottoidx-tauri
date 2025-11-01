// Marketplace RPC Commands
use crate::rpc::{
    VerusRpcClient, RpcError, Offer, MakeOfferResponse, TakeOfferResponse
};
use serde_json::{json, Value};

impl VerusRpcClient {
    /// Get offers for currency or identity
    pub async fn get_offers(
        &self, 
        currency_or_id: &str, 
        is_currency: Option<bool>, 
        with_tx: Option<bool>
    ) -> Result<Value, RpcError> {
        let mut params = vec![json!(currency_or_id)];
        
        // Add iscurrency parameter (default false for identity)
        params.push(json!(is_currency.unwrap_or(false)));
        
        // Add withtx parameter if specified
        if let Some(include_tx) = with_tx {
            params.push(json!(include_tx));
        }
        
        self.call("getoffers", json!(params)).await
    }

    /// List user's open offers (both unexpired and expired)
    pub async fn list_open_offers(&self) -> Result<Value, RpcError> {
        // Use true true to show both unexpired and expired offers
        self.call("listopenoffers", json!([true, true])).await
    }

    /// Create new offer
    pub async fn make_offer(
        &self,
        from_address: &str,
        offer_data: &Value,
        return_tx: Option<bool>
    ) -> Result<MakeOfferResponse, RpcError> {
        let mut params = vec![json!(from_address), offer_data.clone()];
        if let Some(ret_tx) = return_tx {
            params.push(json!(ret_tx));
        }
        self.call("makeoffer", json!(params)).await
    }

    /// Accept existing offer
    pub async fn take_offer(
        &self,
        from_address: &str,
        offer_data: &Value,
        return_tx: Option<bool>
    ) -> Result<TakeOfferResponse, RpcError> {
        let mut params = vec![json!(from_address), offer_data.clone()];
        if let Some(ret_tx) = return_tx {
            params.push(json!(ret_tx));
        }
        self.call("takeoffer", json!(params)).await
    }

    /// Cancel open offers
    /// Returns () since closeoffers always returns null on success
    pub async fn close_offers(
        &self,
        offer_ids: Vec<String>,
        destination_address: Option<&str>
    ) -> Result<(), RpcError> {
        let mut params = vec![json!(offer_ids)];
        if let Some(dest) = destination_address {
            params.push(json!(dest));
        }
        
        // closeoffers returns null on success, so we parse as Option<Value>
        let _result: Option<Value> = self.call("closeoffers", json!(params)).await?;
        Ok(())
    }
}