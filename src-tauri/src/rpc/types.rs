use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcCredentials {
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SupportedChain {
    Vrsc,
    VrscTest,
    Varrr,
    Vdex,
    Chips,
}

impl SupportedChain {
    pub fn to_string(&self) -> &'static str {
        match self {
            SupportedChain::Vrsc => "vrsc",
            SupportedChain::VrscTest => "vrsctest",
            SupportedChain::Varrr => "varrr",
            SupportedChain::Vdex => "vdex",
            SupportedChain::Chips => "chips",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub name: SupportedChain,
    pub credentials: RpcCredentials,
    pub is_active: bool,
}

// RPC Request/Response types
#[derive(Debug, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct JsonRpcResponse<T> {
    pub jsonrpc: String,
    pub result: Option<T>,
    pub error: Option<JsonRpcError>,
    pub id: String,
}

// Flexible JSON-RPC response for Verus daemon (missing jsonrpc field)
#[derive(Debug, Deserialize)]
pub struct VerusJsonRpcResponse<T> {
    pub result: Option<T>,
    pub error: Option<JsonRpcError>,
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

// System & Wallet Command Response Types
#[derive(Debug, Deserialize, Serialize)]
pub struct GetInfoResponse {
    #[serde(rename = "VRSCversion")]
    pub vrsc_version: Option<String>,
    pub version: Option<u64>,
    pub protocolversion: Option<u32>,
    pub chainid: Option<String>,
    pub notarychainid: Option<String>,
    pub name: Option<String>,
    pub notarizedroot: Option<serde_json::Value>,
    pub walletversion: Option<u32>,
    pub blocks: Option<u64>,
    pub longestchain: Option<u64>,
    pub timeoffset: Option<i32>,
    pub tiptime: Option<u64>,
    pub nextblocktime: Option<u64>,
    pub connections: Option<u32>,
    pub proxy: Option<String>,
    pub difficulty: Option<f64>,
    pub testnet: Option<bool>,
    pub keypoololdest: Option<u64>,
    pub keypoolsize: Option<u32>,
    pub paytxfee: Option<f64>,
    pub tls_established: Option<u32>,
    pub tls_verified: Option<u32>,
    pub relayfee: Option<f64>,
    pub errors: Option<String>,
    #[serde(rename = "CCid")]
    pub cc_id: Option<u32>,
    pub p2pport: Option<u16>,
    pub rpcport: Option<u16>,
    pub magic: Option<i64>,
    pub premine: Option<u64>,
    pub reward: Option<String>,
    pub halving: Option<String>,
    pub decay: Option<String>,
    pub endsubsidy: Option<String>,
    pub veruspos: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetWalletInfoResponse {
    pub walletname: Option<String>,
    pub walletversion: Option<u32>,
    pub balance: Option<f64>,
    pub unlocked_balance: Option<f64>,
    pub unconfirmed_balance: Option<f64>,
    pub immature_balance: Option<f64>,
    pub eligible_staking_outputs: Option<u32>,
    pub eligible_staking_balance: Option<f64>,
    pub reserve_balance: Option<HashMap<String, f64>>,
    pub txcount: Option<u32>,
    pub keypoololdest: Option<u64>,
    pub keypoolsize: Option<u32>,
    pub paytxfee: Option<f64>,
    pub unlocked_until: Option<u64>,
    pub seedfp: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetMiningInfoResponse {
    pub blocks: Option<u64>,
    pub currentblocksize: Option<u64>,
    pub currentblocktx: Option<u64>,
    pub averageblockfees: Option<f64>,
    pub difficulty: Option<f64>,
    pub networkhashps: Option<f64>,
    pub stakingsupply: Option<f64>,
    pub errors: Option<String>,
    pub generate: Option<bool>,
    pub genproclimit: Option<u32>,
    pub localhashps: Option<f64>,
    pub pooledtx: Option<u64>,
    pub testnet: Option<bool>,
    pub chain: Option<String>,
    pub staking: Option<bool>,
    pub numthreads: Option<u32>,
    pub mergemining: Option<u32>,
    pub mergeminedchains: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetBlockchainInfoResponse {
    pub chain: String,
    pub blocks: u64,
    pub headers: u64,
    pub bestblockhash: String,
    pub difficulty: f64,
    pub mediantime: u64,
    pub verificationprogress: f64,
    pub chainwork: String,
    pub size_on_disk: u64,
    pub pruned: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetNetworkInfoResponse {
    pub version: u32,
    pub subversion: String,
    pub protocolversion: u32,
    pub localservices: String,
    pub localrelay: bool,
    pub timeoffset: i32,
    pub connections: u32,
    pub networkactive: bool,
    pub networks: Vec<NetworkInfo>,
    pub relayfee: f64,
    pub localaddresses: Vec<LocalAddress>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NetworkInfo {
    pub name: String,
    pub limited: bool,
    pub reachable: bool,
    pub proxy: String,
    pub proxy_randomize_credentials: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LocalAddress {
    pub address: String,
    pub port: u16,
    pub score: u32,
}

#[derive(Debug, Serialize)]
pub struct AddressInfo {
    pub address: String,
    pub amount: f64,
    pub account: Option<String>,
}

// Custom deserializer for the nested array format [address, amount, account?]
impl<'de> Deserialize<'de> for AddressInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let values: Vec<serde_json::Value> = Vec::deserialize(deserializer)?;
        
        if values.len() < 2 {
            return Err(serde::de::Error::custom("AddressInfo array must have at least 2 elements"));
        }
        
        let address = values[0].as_str()
            .ok_or_else(|| serde::de::Error::custom("Address must be a string"))?
            .to_string();
            
        let amount = values[1].as_f64()
            .ok_or_else(|| serde::de::Error::custom("Amount must be a number"))?;
            
        let account = if values.len() > 2 {
            values[2].as_str().map(|s| s.to_string())
        } else {
            None
        };
        
        Ok(AddressInfo { address, amount, account })
    }
}

// Type alias for the full response structure
pub type AddressGroupings = Vec<Vec<AddressInfo>>;

// Currency balance response - dynamic object with currency names as keys
pub type CurrencyBalanceResponse = HashMap<String, f64>;

#[derive(Debug, Deserialize, Serialize)]
pub struct ZTotalBalanceResponse {
    pub transparent: Option<String>,
    pub private: Option<String>,
    pub total: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OperationStatus {
    pub id: String,
    pub status: String,
    pub creation_time: u64,
    pub result: Option<serde_json::Value>,
    pub error: Option<serde_json::Value>,
    pub method: String,
    pub params: serde_json::Value,
}

// Identity Command Response Types
#[derive(Debug, Deserialize, Serialize)]
pub struct IdentityDetails {
    pub version: u32,
    pub flags: u32,
    pub primaryaddresses: Vec<String>,
    pub minimumsignatures: u32,
    pub name: String,
    pub identityaddress: String,
    pub parent: String,
    pub systemid: String,
    pub contentmap: HashMap<String, String>,
    pub contentmultimap: HashMap<String, serde_json::Value>,
    pub revocationauthority: String,
    pub recoveryauthority: String,
    pub privateaddress: Option<String>,
    pub timelock: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Identity {
    pub identity: IdentityDetails,
    pub blockheight: u64,
    pub txid: String,
    pub status: String,
    pub canspendfor: bool,
    pub cansignfor: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RegisterNameCommitmentResponse {
    pub txid: String,
    pub namereservation: NameReservation,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NameReservation {
    pub name: String,
    pub salt: String,
    pub referral: String,
    pub parent: String,
    pub nameid: String,
}

// Marketplace Command Response Types
#[derive(Debug, Deserialize, Serialize)]
pub struct Offer {
    pub txid: String,
    pub offer: OfferDetails,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OfferDetails {
    pub name: String,
    pub offer: OfferTerms,
    pub blockexpiry: u64,
    pub expired: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OfferTerms {
    pub currency: String,
    pub amount: f64,
    pub identity: String,
    pub name: String,
    pub parent: String,
    pub originalcurrency: String,
    pub originalamount: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MakeOfferResponse {
    pub txid: String,
    pub oprettxid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hex: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TakeOfferResponse {
    pub txid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hextx: Option<String>,
}

// Currency Command Response Types
#[derive(Debug, Deserialize, Serialize)]
pub struct Currency {
    pub name: String,
    pub currencyid: String,
    pub parent: String,
    pub systemid: String,
    pub notarizationprotocol: u32,
    pub proofprotocol: u32,
    pub options: u32,
    pub nativecurrencyid: CurrencyId,
    pub launchsystemid: String,
    pub startblock: u64,
    pub endblock: Option<u64>,
    pub currencies: Vec<String>,
    pub weights: Vec<f64>,
    pub reserves: Vec<f64>,
    pub conversions: Vec<f64>,
    pub initialsupply: f64,
    pub prelaunch: Option<bool>,
    pub minpreconversion: Vec<f64>,
    pub maxpreconversion: Vec<f64>,
    pub initialcontributions: Vec<f64>,
    pub preconversions: Vec<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CurrencyId {
    pub hex: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DefineCurrencyResponse {
    pub hex: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EstimateConversionResponse {
    pub estimatedcurrencyout: f64,
    pub estimatedcurrencyin: f64,
    pub estimatedconversionfee: f64,
    pub estimatedpriceinreserve: f64,
    pub estimatedpriceincurrency: f64,
}

// Transaction Command Response Types
#[derive(Debug, Deserialize, Serialize)]
pub struct SendCurrencyResponse {
    pub txid: String,
    pub hex: Option<String>,
}

// Request Types for Complex Commands
#[derive(Debug, Serialize, Deserialize)]
pub struct IdentityRegistration {
    pub name: String,
    pub parent: String,
    pub primaryaddresses: Vec<String>,
    pub minimumsignatures: u32,
    pub revocationauthority: Option<String>,
    pub recoveryauthority: Option<String>,
    pub privateaddress: Option<String>,
    pub timelock: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdentityUpdate {
    pub name: String,
    pub parent: Option<String>,
    pub primaryaddresses: Option<Vec<String>>,
    pub minimumsignatures: Option<u32>,
    pub revocationauthority: Option<String>,
    pub recoveryauthority: Option<String>,
    pub privateaddress: Option<String>,
    pub timelock: Option<u64>,
    pub contentmultimap: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendCurrencyRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    pub amount: f64,
    pub address: String,
    pub convertto: Option<String>,
    pub via: Option<String>,
    pub exportto: Option<String>,
    pub exportcurrency: Option<bool>,
    pub exportid: Option<bool>,
    pub preconvert: Option<String>,
    pub memo: Option<String>,
    pub feecurrency: Option<String>,
    pub addconversionfees: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurrencyDefinition {
    pub name: String,
    pub options: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proofprotocol: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notarizationprotocol: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currencies: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weights: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reserves: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversions: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initialsupply: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minpreconversion: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxpreconversion: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initialcontributions: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub startblock: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endblock: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idregistrationfees: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idreferrallevels: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idimportfees: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preallocations: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prelaunchdiscount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prelaunchcarveout: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub systemid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launchsystemid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nativecurrencyid: Option<serde_json::Value>,
}