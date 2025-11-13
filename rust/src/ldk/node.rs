use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::str::FromStr;
use bitcoin::Network;
use ldk_node::{Builder, Node as LdkNode, Config, NetAddress};
use ldk_node::bitcoin::secp256k1::PublicKey;
use ldk_node::lightning::ln::msgs::SocketAddress;
use ldk_node::lightning_invoice::Bolt11Invoice;

use super::{LightningError, ChannelInfo, OpenChannelParams, CloseChannelParams};
use super::{InvoiceInfo, CreateInvoiceParams};
use super::{PaymentInfo, PayInvoiceParams, KeysendParams, SendOnChainParams};
use super::{PeerInfo, ConnectPeerParams};

pub struct LightningNode {
    node: Arc<Mutex<Option<Arc<LdkNode>>>>,
    network: Arc<Mutex<Option<Network>>>,
    storage_path: Arc<Mutex<Option<PathBuf>>>,
}

impl LightningNode {
    pub fn new() -> Self {
        LightningNode {
            node: Arc::new(Mutex::new(None)),
            network: Arc::new(Mutex::new(None)),
            storage_path: Arc::new(Mutex::new(None)),
        }
    }

    pub fn initialize(
        &self,
        entropy: [u8; 32],
        network: Network,
        storage_path: &str,
        esplora_server: Option<&str>,
    ) -> Result<(), LightningError> {
        let mut node_guard = self.node.lock()
            .map_err(|e| LightningError::LdkError(format!("Lock error: {}", e)))?;

        if node_guard.is_some() {
            return Err(LightningError::AlreadyInitialized);
        }

        // Create ldk-node config
        let mut config = Config::default();
        config.network = network;
        config.storage_dir_path = PathBuf::from(storage_path);

        // Set up listening address
        config.listening_addresses = Some(vec![
            SocketAddress::TcpIpV4 {
                addr: [0, 0, 0, 0],
                port: 9735,
            }
        ]);

        // Build the node
        let mut builder = Builder::from_config(config);

        // Set entropy for key generation
        builder.set_entropy_seed_bytes(entropy)
            .map_err(|e| LightningError::LdkError(format!("Failed to set entropy: {:?}", e)))?;

        // Set Esplora server for chain data
        if let Some(esplora_url) = esplora_server {
            builder.set_esplora_server(esplora_url.to_string());
        }

        // Build the node
        let node = builder.build()
            .map_err(|e| LightningError::LdkError(format!("Failed to build node: {:?}", e)))?;

        *node_guard = Some(Arc::new(node));

        let mut network_guard = self.network.lock()
            .map_err(|e| LightningError::LdkError(format!("Lock error: {}", e)))?;
        *network_guard = Some(network);

        let mut path_guard = self.storage_path.lock()
            .map_err(|e| LightningError::LdkError(format!("Lock error: {}", e)))?;
        *path_guard = Some(PathBuf::from(storage_path));

        Ok(())
    }

    pub fn start(&self) -> Result<(), LightningError> {
        let node_guard = self.node.lock()
            .map_err(|e| LightningError::LdkError(format!("Lock error: {}", e)))?;

        let node = node_guard.as_ref()
            .ok_or(LightningError::NotInitialized)?;

        node.start()
            .map_err(|e| LightningError::LdkError(format!("Failed to start node: {:?}", e)))?;

        Ok(())
    }

    pub fn stop(&self) -> Result<(), LightningError> {
        let node_guard = self.node.lock()
            .map_err(|e| LightningError::LdkError(format!("Lock error: {}", e)))?;

        let node = node_guard.as_ref()
            .ok_or(LightningError::NotInitialized)?;

        node.stop()
            .map_err(|e| LightningError::LdkError(format!("Failed to stop node: {:?}", e)))?;

        Ok(())
    }

    pub fn sync_wallets(&self) -> Result<(), LightningError> {
        let node_guard = self.node.lock()
            .map_err(|e| LightningError::LdkError(format!("Lock error: {}", e)))?;

        let node = node_guard.as_ref()
            .ok_or(LightningError::NotInitialized)?;

        node.sync_wallets()
            .map_err(|e| LightningError::LdkError(format!("Failed to sync wallets: {:?}", e)))?;

        Ok(())
    }

    pub fn get_node_id(&self) -> Result<String, LightningError> {
        let node_guard = self.node.lock()
            .map_err(|e| LightningError::LdkError(format!("Lock error: {}", e)))?;

        let node = node_guard.as_ref()
            .ok_or(LightningError::NotInitialized)?;

        Ok(node.node_id().to_string())
    }

    // Peer management
    pub fn connect_peer(&self, params: ConnectPeerParams) -> Result<(), LightningError> {
        let node_guard = self.node.lock()
            .map_err(|e| LightningError::LdkError(format!("Lock error: {}", e)))?;

        let node = node_guard.as_ref()
            .ok_or(LightningError::NotInitialized)?;

        let pubkey = PublicKey::from_str(&params.node_id)
            .map_err(|e| LightningError::PeerError(format!("Invalid pubkey: {}", e)))?;

        let net_address = NetAddress::from_str(&format!("{}:{}", params.address, params.port))
            .map_err(|e| LightningError::PeerError(format!("Invalid address: {}", e)))?;

        node.connect(pubkey, net_address, true)
            .map_err(|e| LightningError::PeerError(format!("Failed to connect: {:?}", e)))?;

        Ok(())
    }

    pub fn disconnect_peer(&self, node_id: &str) -> Result<(), LightningError> {
        let node_guard = self.node.lock()
            .map_err(|e| LightningError::LdkError(format!("Lock error: {}", e)))?;

        let node = node_guard.as_ref()
            .ok_or(LightningError::NotInitialized)?;

        let pubkey = PublicKey::from_str(node_id)
            .map_err(|e| LightningError::PeerError(format!("Invalid pubkey: {}", e)))?;

        node.disconnect(pubkey)
            .map_err(|e| LightningError::PeerError(format!("Failed to disconnect: {:?}", e)))?;

        Ok(())
    }

    pub fn list_peers(&self) -> Result<Vec<PeerInfo>, LightningError> {
        let node_guard = self.node.lock()
            .map_err(|e| LightningError::LdkError(format!("Lock error: {}", e)))?;

        let node = node_guard.as_ref()
            .ok_or(LightningError::NotInitialized)?;

        let peer_details = node.list_peers();

        let peers: Vec<PeerInfo> = peer_details.iter().map(|peer| {
            PeerInfo {
                node_id: peer.node_id.to_string(),
                address: peer.address.as_ref().map(|addr| format!("{:?}", addr)),
                port: None, // ldk-node doesn't expose port separately
                is_connected: peer.is_connected,
            }
        }).collect();

        Ok(peers)
    }

    // Channel management
    pub fn open_channel(&self, params: OpenChannelParams) -> Result<String, LightningError> {
        let node_guard = self.node.lock()
            .map_err(|e| LightningError::LdkError(format!("Lock error: {}", e)))?;

        let node = node_guard.as_ref()
            .ok_or(LightningError::NotInitialized)?;

        let pubkey = PublicKey::from_str(&params.counterparty_node_id)
            .map_err(|e| LightningError::ChannelError(format!("Invalid pubkey: {}", e)))?;

        // Connect to peer first if address is provided
        if let (Some(address), Some(port)) = (params.peer_address, params.peer_port) {
            let net_address = NetAddress::from_str(&format!("{}:{}", address, port))
                .map_err(|e| LightningError::ChannelError(format!("Invalid address: {}", e)))?;

            node.connect(pubkey, net_address, true)
                .map_err(|e| LightningError::ChannelError(format!("Failed to connect: {:?}", e)))?;
        }

        // Open channel
        let user_channel_id = node.open_channel(
            pubkey,
            params.channel_value_satoshis,
            Some(params.push_msat),
            None, // announce_channel (use default)
        ).map_err(|e| LightningError::ChannelError(format!("Failed to open channel: {:?}", e)))?;

        Ok(format!("{:x}", user_channel_id))
    }

    pub fn close_channel(&self, params: CloseChannelParams) -> Result<(), LightningError> {
        let node_guard = self.node.lock()
            .map_err(|e| LightningError::LdkError(format!("Lock error: {}", e)))?;

        let node = node_guard.as_ref()
            .ok_or(LightningError::NotInitialized)?;

        // Parse channel_id as user_channel_id (hex string to u128)
        let user_channel_id = u128::from_str_radix(&params.channel_id, 16)
            .map_err(|e| LightningError::ChannelError(format!("Invalid channel ID: {}", e)))?;

        // Get counterparty pubkey from channel list
        let channels = node.list_channels();
        let channel = channels.iter()
            .find(|c| c.user_channel_id == user_channel_id)
            .ok_or(LightningError::ChannelError("Channel not found".to_string()))?;

        let counterparty_pubkey = channel.counterparty_node_id;

        if params.force {
            node.force_close_channel(&channel.channel_id, counterparty_pubkey)
                .map_err(|e| LightningError::ChannelError(format!("Failed to force close: {:?}", e)))?;
        } else {
            node.close_channel(&channel.channel_id, counterparty_pubkey)
                .map_err(|e| LightningError::ChannelError(format!("Failed to close: {:?}", e)))?;
        }

        Ok(())
    }

    pub fn list_channels(&self) -> Result<Vec<ChannelInfo>, LightningError> {
        let node_guard = self.node.lock()
            .map_err(|e| LightningError::LdkError(format!("Lock error: {}", e)))?;

        let node = node_guard.as_ref()
            .ok_or(LightningError::NotInitialized)?;

        let channel_details = node.list_channels();

        let channels: Vec<ChannelInfo> = channel_details.iter().map(|channel| {
            ChannelInfo {
                channel_id: hex::encode(channel.channel_id.0),
                counterparty_node_id: channel.counterparty_node_id.to_string(),
                channel_value_sats: channel.channel_value_sats,
                balance_sats: channel.outbound_capacity_msat / 1000,
                outbound_capacity_sats: channel.outbound_capacity_msat / 1000,
                inbound_capacity_sats: channel.inbound_capacity_msat / 1000,
                is_usable: channel.is_channel_ready && channel.is_usable,
                is_public: channel.is_public,
                is_ready: channel.is_channel_ready,
                is_closing: false, // ldk-node doesn't expose this directly
                confirmations_required: channel.confirmations_required,
            }
        }).collect();

        Ok(channels)
    }

    // Invoice management
    pub fn create_invoice(&self, params: CreateInvoiceParams) -> Result<InvoiceInfo, LightningError> {
        let node_guard = self.node.lock()
            .map_err(|e| LightningError::LdkError(format!("Lock error: {}", e)))?;

        let node = node_guard.as_ref()
            .ok_or(LightningError::NotInitialized)?;

        let invoice = node.receive_payment(
            params.amount_sats.unwrap_or(0),
            params.description.as_deref().unwrap_or(""),
            params.expiry_secs,
        ).map_err(|e| LightningError::InvoiceError(format!("Failed to create invoice: {:?}", e)))?;

        let parsed_invoice = Bolt11Invoice::from_str(&invoice.to_string())
            .map_err(|e| LightningError::InvoiceError(format!("Failed to parse invoice: {}", e)))?;

        Ok(InvoiceInfo {
            bolt11: invoice.to_string(),
            payment_hash: hex::encode(parsed_invoice.payment_hash().as_ref()),
            amount_sats: params.amount_sats,
            description: params.description,
            created_at: parsed_invoice.timestamp().as_secs(),
            expires_at: parsed_invoice.timestamp().as_secs() + params.expiry_secs as u64,
        })
    }

    // Payment operations
    pub fn pay_invoice(&self, params: PayInvoiceParams) -> Result<PaymentInfo, LightningError> {
        let node_guard = self.node.lock()
            .map_err(|e| LightningError::LdkError(format!("Lock error: {}", e)))?;

        let node = node_guard.as_ref()
            .ok_or(LightningError::NotInitialized)?;

        let invoice = Bolt11Invoice::from_str(&params.bolt11)
            .map_err(|e| LightningError::PaymentError(format!("Invalid invoice: {}", e)))?;

        let payment_id = node.send_payment(&invoice)
            .map_err(|e| LightningError::PaymentError(format!("Payment failed: {:?}", e)))?;

        Ok(PaymentInfo {
            payment_hash: hex::encode(payment_id.0),
            payment_type: super::payments::PaymentType::Sent,
            amount_sats: invoice.amount_milli_satoshis().map(|a| a / 1000).unwrap_or(0),
            fee_sats: None,
            status: super::payments::PaymentStatus::Pending,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            description: invoice.description().map(|d| d.to_string()),
            destination: invoice.recover_payee_pub_key().map(|pk| pk.to_string()),
            preimage: None,
            bolt11: Some(params.bolt11),
        })
    }

    pub fn send_keysend(&self, params: KeysendParams) -> Result<PaymentInfo, LightningError> {
        let node_guard = self.node.lock()
            .map_err(|e| LightningError::LdkError(format!("Lock error: {}", e)))?;

        let node = node_guard.as_ref()
            .ok_or(LightningError::NotInitialized)?;

        let pubkey = PublicKey::from_str(&params.destination_pubkey)
            .map_err(|e| LightningError::PaymentError(format!("Invalid pubkey: {}", e)))?;

        let payment_id = node.send_spontaneous_payment(
            params.amount_sats,
            pubkey,
        ).map_err(|e| LightningError::PaymentError(format!("Keysend failed: {:?}", e)))?;

        Ok(PaymentInfo {
            payment_hash: hex::encode(payment_id.0),
            payment_type: super::payments::PaymentType::Sent,
            amount_sats: params.amount_sats,
            fee_sats: None,
            status: super::payments::PaymentStatus::Pending,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            description: Some("Keysend payment".to_string()),
            destination: Some(params.destination_pubkey),
            preimage: None,
            bolt11: None,
        })
    }

    pub fn send_onchain(&self, params: SendOnChainParams) -> Result<String, LightningError> {
        let node_guard = self.node.lock()
            .map_err(|e| LightningError::LdkError(format!("Lock error: {}", e)))?;

        let node = node_guard.as_ref()
            .ok_or(LightningError::NotInitialized)?;

        let address = bitcoin::Address::from_str(&params.address)
            .map_err(|e| LightningError::PaymentError(format!("Invalid address: {}", e)))?
            .assume_checked();

        let txid = node.send_to_onchain_address(
            &address,
            params.amount_sats,
        ).map_err(|e| LightningError::PaymentError(format!("On-chain send failed: {:?}", e)))?;

        Ok(txid.to_string())
    }

    pub fn list_payments(&self) -> Result<Vec<PaymentInfo>, LightningError> {
        let node_guard = self.node.lock()
            .map_err(|e| LightningError::LdkError(format!("Lock error: {}", e)))?;

        let node = node_guard.as_ref()
            .ok_or(LightningError::NotInitialized)?;

        let payments = node.list_payments();

        let payment_list: Vec<PaymentInfo> = payments.iter().map(|payment| {
            let (payment_type, status) = match payment.direction {
                ldk_node::payment::PaymentDirection::Inbound => {
                    (super::payments::PaymentType::Received,
                     if payment.status == ldk_node::payment::PaymentStatus::Succeeded {
                         super::payments::PaymentStatus::Completed
                     } else if payment.status == ldk_node::payment::PaymentStatus::Failed {
                         super::payments::PaymentStatus::Failed
                     } else {
                         super::payments::PaymentStatus::Pending
                     })
                },
                ldk_node::payment::PaymentDirection::Outbound => {
                    (super::payments::PaymentType::Sent,
                     if payment.status == ldk_node::payment::PaymentStatus::Succeeded {
                         super::payments::PaymentStatus::Completed
                     } else if payment.status == ldk_node::payment::PaymentStatus::Failed {
                         super::payments::PaymentStatus::Failed
                     } else {
                         super::payments::PaymentStatus::Pending
                     })
                },
            };

            PaymentInfo {
                payment_hash: hex::encode(payment.id.0),
                payment_type,
                amount_sats: payment.amount_msat.map(|a| a / 1000).unwrap_or(0),
                fee_sats: payment.fee_msat.map(|f| f / 1000),
                status,
                timestamp: 0, // ldk-node doesn't expose timestamp
                description: None,
                destination: None,
                preimage: payment.preimage.map(|p| hex::encode(p.0)),
                bolt11: None,
            }
        }).collect();

        Ok(payment_list)
    }

    pub fn get_onchain_address(&self) -> Result<String, LightningError> {
        let node_guard = self.node.lock()
            .map_err(|e| LightningError::LdkError(format!("Lock error: {}", e)))?;

        let node = node_guard.as_ref()
            .ok_or(LightningError::NotInitialized)?;

        let address = node.new_onchain_address()
            .map_err(|e| LightningError::LdkError(format!("Failed to get address: {:?}", e)))?;

        Ok(address.to_string())
    }

    pub fn get_spendable_onchain_balance(&self) -> Result<u64, LightningError> {
        let node_guard = self.node.lock()
            .map_err(|e| LightningError::LdkError(format!("Lock error: {}", e)))?;

        let node = node_guard.as_ref()
            .ok_or(LightningError::NotInitialized)?;

        let balance = node.spendable_onchain_balance_sats()
            .map_err(|e| LightningError::LdkError(format!("Failed to get balance: {:?}", e)))?;

        Ok(balance)
    }

    pub fn get_total_onchain_balance(&self) -> Result<u64, LightningError> {
        let node_guard = self.node.lock()
            .map_err(|e| LightningError::LdkError(format!("Lock error: {}", e)))?;

        let node = node_guard.as_ref()
            .ok_or(LightningError::NotInitialized)?;

        let balance = node.total_onchain_balance_sats()
            .map_err(|e| LightningError::LdkError(format!("Failed to get balance: {:?}", e)))?;

        Ok(balance)
    }
}

impl Default for LightningNode {
    fn default() -> Self {
        Self::new()
    }
}
