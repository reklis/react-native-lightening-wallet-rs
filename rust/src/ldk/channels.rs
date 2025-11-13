use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelInfo {
    pub channel_id: String,
    pub counterparty_node_id: String,
    pub channel_value_sats: u64,
    pub balance_sats: u64,
    pub outbound_capacity_sats: u64,
    pub inbound_capacity_sats: u64,
    pub is_usable: bool,
    pub is_public: bool,
    pub is_ready: bool,
    pub is_closing: bool,
    pub confirmations_required: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenChannelParams {
    pub counterparty_node_id: String,
    pub channel_value_satoshis: u64,
    #[serde(default)]
    pub push_msat: u64,
    #[serde(default)]
    pub is_public: bool,
    pub peer_address: Option<String>,
    pub peer_port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseChannelParams {
    pub channel_id: String,
    #[serde(default)]
    pub force: bool,
    pub peer_address: Option<String>,
    pub peer_port: Option<u16>,
}
