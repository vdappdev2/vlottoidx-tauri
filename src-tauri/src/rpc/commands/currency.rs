// Currency Management RPC Commands
use crate::rpc::{
    VerusRpcClient, RpcError, Currency, DefineCurrencyResponse, 
    CurrencyDefinition
};
use serde_json::json;

impl VerusRpcClient {
    /// List all currencies
    pub async fn list_currencies(
        &self,
        query: Option<&str>,
        verbose: Option<bool>
    ) -> Result<Vec<Currency>, RpcError> {
        let mut params = Vec::new();
        if let Some(q) = query {
            params.push(json!(q));
            if let Some(v) = verbose {
                params.push(json!(v));
            }
        }
        self.call("listcurrencies", json!(params)).await
    }

    /// Get currency details
    pub async fn get_currency(
        &self,
        currency_name: &str,
        height: Option<u64>
    ) -> Result<Currency, RpcError> {
        let mut params = vec![json!(currency_name)];
        if let Some(h) = height {
            params.push(json!(h));
        }
        self.call("getcurrency", json!(params)).await
    }

    /// Define new currency
    pub async fn define_currency(
        &self,
        currency_definition: &CurrencyDefinition,
        fractional_gateway: Option<bool>,
        reserves: Option<Vec<String>>
    ) -> Result<DefineCurrencyResponse, RpcError> {
        let mut params = vec![json!(currency_definition)];
        if let Some(gateway) = fractional_gateway {
            params.push(json!(gateway));
            if let Some(res) = reserves {
                params.push(json!(res));
            }
        }
        self.call("definecurrency", json!(params)).await
    }

    /// Calculate conversion costs
    pub async fn estimate_conversion(
        &self,
        currency: &str,
        amount: f64,
        convert_to: &str,
        via: Option<&str>
    ) -> Result<serde_json::Value, RpcError> {
        let mut conversion_obj = json!({
            "currency": currency,
            "amount": amount,
            "convertto": convert_to
        });
        
        // Only include via if provided
        if let Some(via_currency) = via {
            if !via_currency.is_empty() {
                conversion_obj["via"] = json!(via_currency);
            }
        }
        
        // estimateconversion expects an array of conversion objects
        self.call("estimateconversion", json!([conversion_obj])).await
    }

    /// Get currency converters for finding basket currencies
    pub async fn get_currency_converters(
        &self,
        currencies: Vec<&str>
    ) -> Result<serde_json::Value, RpcError> {
        self.call("getcurrencyconverters", json!(currencies)).await
    }

    /// Get currency state(s) with optional height range and conversion data
    pub async fn get_currency_state(
        &self,
        currency_name: &str,
        height_range: Option<&str>,
        conversion_currency: Option<&str>
    ) -> Result<serde_json::Value, RpcError> {
        let mut params = vec![json!(currency_name)];
        
        if let Some(range) = height_range {
            params.push(json!(range));
            if let Some(conv_currency) = conversion_currency {
                params.push(json!(conv_currency));
            }
        } else if let Some(conv_currency) = conversion_currency {
            // If only conversion currency is provided, add null for height range
            params.push(json!(null));
            params.push(json!(conv_currency));
        }
        
        self.call("getcurrencystate", json!(params)).await
    }
}