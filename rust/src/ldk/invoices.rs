use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceInfo {
    pub bolt11: String,
    pub payment_hash: String,
    pub amount_sats: Option<u64>,
    pub description: Option<String>,
    pub created_at: u64,
    pub expires_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInvoiceParams {
    pub amount_sats: Option<u64>,
    pub description: Option<String>,
    #[serde(default = "default_expiry")]
    pub expiry_secs: u32,
}

fn default_expiry() -> u32 {
    3600 // 1 hour
}
