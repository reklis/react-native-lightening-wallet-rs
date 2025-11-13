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

/// Parameters for sending a keysend (spontaneous) Lightning payment
///
/// # Podcasting 2.0 Support
/// For Podcasting 2.0 value-for-value payments, use `custom_records` to attach metadata:
/// - Key: customKey (TLV type number, e.g., 696969)
/// - Value: customValue (data as bytes)
///
/// ## Standard Podcasting 2.0 TLV Types:
/// - 7629169 (0x7461A1): podcast name
/// - 7629171 (0x7461A3): episode GUID
/// - 7629173 (0x7461A5): action (stream/boost)
/// - 7629175 (0x7461A7): timestamp
/// - Custom keys: any number (e.g., 696969 as shown in value splits)
///
/// ## Example:
/// ```json
/// {
///   "destination_pubkey": "035c42ea...",
///   "amount_sats": 1000,
///   "custom_records": {
///     "7629169": [112, 111, 100, 99, 97, 115, 116], // "podcast" as bytes
///     "696969": [56]                                 // "8" as bytes (customValue="8")
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeysendParams {
    /// Lightning node public key (33-byte hex string)
    pub destination_pubkey: String,

    /// Amount to send in satoshis
    pub amount_sats: u64,

    /// Custom TLV records for Podcasting 2.0 metadata
    /// Map of customKey (u64) -> customValue (bytes)
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
