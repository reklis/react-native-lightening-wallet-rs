use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub node_id: String,
    pub address: Option<String>,
    pub port: Option<u16>,
    pub is_connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectPeerParams {
    pub node_id: String,
    pub address: String,
    pub port: u16,
}
