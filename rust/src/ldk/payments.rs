use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentInfo {
    pub payment_hash: String,
    pub payment_type: PaymentType,
    pub amount_sats: u64,
    pub fee_sats: Option<u64>,
    pub status: PaymentStatus,
    pub timestamp: u64,
    pub description: Option<String>,
    pub destination: Option<String>,
    pub preimage: Option<String>,
    pub bolt11: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentType {
    Sent,
    Received,
    OnchainSent,
    OnchainReceived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaymentStatus {
    Pending,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayInvoiceParams {
    pub bolt11: String,
    pub amount_sats: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeysendParams {
    pub destination_pubkey: String,
    pub amount_sats: u64,
    #[serde(default)]
    pub custom_records: HashMap<u64, Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendOnChainParams {
    pub address: String,
    pub amount_sats: u64,
    #[serde(default = "default_fee_rate")]
    pub fee_rate_sat_per_vbyte: u64,
}

fn default_fee_rate() -> u64 {
    10
}
