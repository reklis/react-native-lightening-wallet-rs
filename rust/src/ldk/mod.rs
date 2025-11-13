mod node;
mod channels;
mod invoices;
mod payments;
mod peers;
pub mod podcasting;

pub use node::LightningNode;
pub use channels::{ChannelInfo, OpenChannelParams, CloseChannelParams};
pub use invoices::{InvoiceInfo, CreateInvoiceParams};
pub use payments::{PaymentInfo, PayInvoiceParams, KeysendParams, SendOnChainParams};
pub use peers::{PeerInfo, ConnectPeerParams};
pub use podcasting::{Podcasting20Builder, tlv_types};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum LightningError {
    #[error("Not initialized")]
    NotInitialized,
    #[error("Already initialized")]
    AlreadyInitialized,
    #[error("LDK error: {0}")]
    LdkError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Peer error: {0}")]
    PeerError(String),
    #[error("Channel error: {0}")]
    ChannelError(String),
    #[error("Payment error: {0}")]
    PaymentError(String),
    #[error("Invoice error: {0}")]
    InvoiceError(String),
    #[error("Storage error: {0}")]
    StorageError(String),
}
