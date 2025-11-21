use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::str::FromStr;
use ldk_node::{Builder, Node as LdkNode};
use ldk_node::config::{Config, AnchorChannelsConfig};
use ldk_node::bitcoin::{Network, Address as BdkAddress};
use ldk_node::bitcoin::secp256k1::PublicKey;
use ldk_node::lightning::ln::msgs::SocketAddress;
use ldk_node::lightning_invoice::{Bolt11Invoice, Description};

use super::{LightningError, ChannelInfo, OpenChannelParams, CloseChannelParams};
use super::{InvoiceInfo, CreateInvoiceParams};
use super::{PaymentInfo, PayInvoiceParams, KeysendParams, SendOnChainParams};
use super::{PeerInfo, ConnectPeerParams};

pub struct LightningNode {
    node: Arc<Mutex<Option<Arc<LdkNode>>>>,
    network: Arc<Mutex<Option<Network>>>,
    storage_path: Arc<Mutex<Option<PathBuf>>>,
}

// Note: Many methods are currently unused but are part of the public API
// that will be exposed to React Native for Lightning functionality
#[allow(dead_code)]
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

        // Expand 32-byte entropy to 64 bytes by repeating it
        // Note: For production, consider using a KDF to derive 64 bytes properly
        let mut entropy_64 = [0u8; 64];
        entropy_64[..32].copy_from_slice(&entropy);
        entropy_64[32..].copy_from_slice(&entropy);

        // Create config with anchor channels enabled
        // Anchor channels allow dynamic fee bumping at close time, avoiding large upfront fee reserves
        let mut config = ldk_node::config::default_config();
        config.network = network;
        config.storage_dir_path = storage_path.to_string();
        config.listening_addresses = Some(vec![
            SocketAddress::TcpIpV4 {
                addr: [0, 0, 0, 0],
                port: 9735,
            }
        ]);

        // Enable anchor channels with default settings
        // This solves the high commitment transaction fee reserve problem on testnet
        config.anchor_channels_config = Some(AnchorChannelsConfig {
            trusted_peers_no_reserve: vec![],
            per_channel_reserve_sats: 25000, // Default reserve for anchor channels
        });

        eprintln!("[LDK INIT] Anchor channels config set: {:?}", config.anchor_channels_config.is_some());
        eprintln!("[LDK INIT] Network: {:?}", config.network);

        // Build the node using the config
        let mut builder = Builder::from_config(config);
        builder.set_entropy_seed_bytes(entropy_64);

        // Set Esplora server for chain data
        if let Some(esplora_url) = esplora_server {
            builder.set_chain_source_esplora(esplora_url.to_string(), None);
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

        let socket_address = SocketAddress::from_str(&format!("{}:{}", params.address, params.port))
            .map_err(|e| LightningError::PeerError(format!("Invalid address: {}", e)))?;

        node.connect(pubkey, socket_address, true)
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
                address: Some(format!("{:?}", peer.address)),
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

        // Log balances before attempting to open channel
        let balances = node.list_balances();
        eprintln!("[LDK] open_channel attempt - Balances: total={}, spendable={}, channel_amount={}",
            balances.total_onchain_balance_sats,
            balances.spendable_onchain_balance_sats,
            params.channel_value_satoshis);

        let pubkey = PublicKey::from_str(&params.counterparty_node_id)
            .map_err(|e| LightningError::ChannelError(format!("Invalid pubkey: {}", e)))?;

        // Prepare socket address if provided
        let socket_address = if let (Some(address), Some(port)) = (params.peer_address, params.peer_port) {
            SocketAddress::from_str(&format!("{}:{}", address, port))
                .map_err(|e| LightningError::ChannelError(format!("Invalid address: {}", e)))?
        } else {
            // If no address provided, we need one for open_channel - use a default that will fail gracefully
            return Err(LightningError::ChannelError("Peer address and port required to open channel".to_string()));
        };

        // Open channel - new signature: open_channel(pubkey, address, amount_sats, push_msat, config)
        let user_channel_id = node.open_channel(
            pubkey,
            socket_address,
            params.channel_value_satoshis,
            Some(params.push_msat),
            None, // channel_config (use default)
        ).map_err(|e| {
            // Pass through the full LDK error message so users can see fee requirements
            let error_msg = format!("{:?}", e); // Use Debug format for more details
            eprintln!("[LDK] Channel open error: {}", error_msg);
            eprintln!("[LDK] Balances at error: total={}, spendable={}",
                balances.total_onchain_balance_sats,
                balances.spendable_onchain_balance_sats);

            // Provide user-friendly error message
            let user_msg = if balances.spendable_onchain_balance_sats < params.channel_value_satoshis {
                format!("Insufficient confirmed funds. You have {} sats confirmed, but need {} sats + fees. Wait for your Bitcoin transaction to confirm.",
                    balances.spendable_onchain_balance_sats, params.channel_value_satoshis)
            } else {
                format!("Failed to create channel: {}", error_msg)
            };

            LightningError::ChannelError(user_msg)
        })?;

        // Convert UserChannelId to hex string
        Ok(hex::encode(user_channel_id.0.to_le_bytes()))
    }

    pub fn close_channel(&self, params: CloseChannelParams) -> Result<(), LightningError> {
        let node_guard = self.node.lock()
            .map_err(|e| LightningError::LdkError(format!("Lock error: {}", e)))?;

        let node = node_guard.as_ref()
            .ok_or(LightningError::NotInitialized)?;

        // Parse channel_id as user_channel_id (hex bytes to u128)
        let channel_id_bytes = hex::decode(&params.channel_id)
            .map_err(|e| LightningError::ChannelError(format!("Invalid channel ID hex: {}", e)))?;

        if channel_id_bytes.len() != 16 {
            return Err(LightningError::ChannelError("Channel ID must be 16 bytes".to_string()));
        }

        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&channel_id_bytes);
        let user_channel_id = ldk_node::UserChannelId(u128::from_le_bytes(bytes));

        // Get counterparty pubkey from channel list
        let channels = node.list_channels();
        let channel = channels.iter()
            .find(|c| c.user_channel_id == user_channel_id)
            .ok_or(LightningError::ChannelError("Channel not found".to_string()))?;

        let counterparty_pubkey = channel.counterparty_node_id;

        if params.force {
            node.force_close_channel(&user_channel_id, counterparty_pubkey, Some("User requested force close".to_string()))
                .map_err(|e| LightningError::ChannelError(format!("Failed to force close: {:?}", e)))?;
        } else {
            node.close_channel(&user_channel_id, counterparty_pubkey)
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
            // Use user_channel_id instead of channel_id for closing channels
            let user_channel_id_bytes = channel.user_channel_id.0.to_le_bytes();
            ChannelInfo {
                channel_id: hex::encode(user_channel_id_bytes),
                counterparty_node_id: channel.counterparty_node_id.to_string(),
                channel_value_sats: channel.channel_value_sats,
                balance_sats: channel.outbound_capacity_msat / 1000,
                outbound_capacity_sats: channel.outbound_capacity_msat / 1000,
                inbound_capacity_sats: channel.inbound_capacity_msat / 1000,
                is_usable: channel.is_channel_ready && channel.is_usable,
                is_public: channel.is_announced,
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

        let desc_str = params.description.clone().unwrap_or_default();
        let description = Description::new(desc_str)
            .map_err(|e| LightningError::InvoiceError(format!("Invalid description: {:?}", e)))?;

        let invoice = if let Some(amount_sats) = params.amount_sats {
            let amount_msat = amount_sats * 1000;
            let desc_ref = ldk_node::lightning_invoice::Bolt11InvoiceDescription::Direct(description.clone());
            node.bolt11_payment().receive(amount_msat, &desc_ref, params.expiry_secs)
                .map_err(|e| LightningError::InvoiceError(format!("Failed to create invoice: {:?}", e)))?
        } else {
            // Variable amount invoice
            let desc_ref = ldk_node::lightning_invoice::Bolt11InvoiceDescription::Direct(description.clone());
            node.bolt11_payment().receive_variable_amount(&desc_ref, params.expiry_secs)
                .map_err(|e| LightningError::InvoiceError(format!("Failed to create variable invoice: {:?}", e)))?
        };

        let timestamp = invoice.timestamp().duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| LightningError::InvoiceError(format!("Invalid timestamp: {}", e)))?
            .as_secs();

        Ok(InvoiceInfo {
            bolt11: invoice.to_string(),
            payment_hash: hex::encode(invoice.payment_hash().as_ref() as &[u8]),
            amount_sats: params.amount_sats,
            description: params.description,
            created_at: timestamp,
            expires_at: timestamp + params.expiry_secs as u64,
        })
    }

    /// Decode a BOLT11 invoice and extract its information
    pub fn decode_invoice(bolt11: &str) -> Result<InvoiceInfo, LightningError> {
        let invoice = Bolt11Invoice::from_str(bolt11)
            .map_err(|e| LightningError::InvoiceError(format!("Invalid invoice: {}", e)))?;

        let timestamp = invoice.timestamp().duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| LightningError::InvoiceError(format!("Invalid timestamp: {}", e)))?
            .as_secs();

        let amount_sats = invoice.amount_milli_satoshis().map(|msat| msat / 1000);

        // Extract description from invoice
        let description_str = match invoice.description() {
            ldk_node::lightning_invoice::Bolt11InvoiceDescriptionRef::Direct(desc) => Some(desc.to_string()),
            ldk_node::lightning_invoice::Bolt11InvoiceDescriptionRef::Hash(_) => None,
        };

        let expiry_secs = invoice.expiry_time().as_secs();

        Ok(InvoiceInfo {
            bolt11: bolt11.to_string(),
            payment_hash: hex::encode(invoice.payment_hash().as_ref() as &[u8]),
            amount_sats,
            description: description_str,
            created_at: timestamp,
            expires_at: timestamp + expiry_secs,
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

        let payment_id = node.bolt11_payment().send(&invoice, None)
            .map_err(|e| LightningError::PaymentError(format!("Payment failed: {:?}", e)))?;

        let description_str = match invoice.description() {
            ldk_node::lightning_invoice::Bolt11InvoiceDescriptionRef::Direct(desc) => Some(desc.to_string()),
            ldk_node::lightning_invoice::Bolt11InvoiceDescriptionRef::Hash(_) => None,
        };

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
            description: description_str,
            destination: Some(invoice.recover_payee_pub_key().to_string()),
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

        let amount_msat = params.amount_sats * 1000;

        // Use send_with_custom_tlvs if custom records are provided (for Podcasting 2.0)
        let payment_id = if params.custom_records.is_empty() {
            node.spontaneous_payment().send(amount_msat, pubkey, None)
                .map_err(|e| LightningError::PaymentError(format!("Keysend failed: {:?}", e)))?
        } else {
            // Convert HashMap to Vec<CustomTlvRecord>
            let custom_tlvs: Vec<ldk_node::CustomTlvRecord> = params.custom_records
                .into_iter()
                .map(|(type_id, data)| ldk_node::CustomTlvRecord {
                    type_num: type_id,
                    value: data,
                })
                .collect();

            node.spontaneous_payment().send_with_custom_tlvs(amount_msat, pubkey, None, custom_tlvs)
                .map_err(|e| LightningError::PaymentError(format!("Keysend with custom TLVs failed: {:?}", e)))?
        };

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

        let address = BdkAddress::from_str(&params.address)
            .map_err(|e| LightningError::PaymentError(format!("Invalid address: {}", e)))?
            .assume_checked();

        let txid = node.onchain_payment().send_to_address(&address, params.amount_sats, None)
            .map_err(|e| LightningError::PaymentError(format!("On-chain send failed: {:?}", e)))?;

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
                fee_sats: payment.fee_paid_msat.map(|f| f / 1000),
                status,
                timestamp: 0, // ldk-node doesn't expose timestamp
                description: None,
                destination: None,
                preimage: None, // preimage not available in PaymentDetails
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

        let address = node.onchain_payment().new_address()
            .map_err(|e| LightningError::LdkError(format!("Failed to get address: {:?}", e)))?;

        Ok(address.to_string())
    }

    pub fn get_spendable_onchain_balance(&self) -> Result<u64, LightningError> {
        let node_guard = self.node.lock()
            .map_err(|e| LightningError::LdkError(format!("Lock error: {}", e)))?;

        let node = node_guard.as_ref()
            .ok_or(LightningError::NotInitialized)?;

        let balances = node.list_balances();
        Ok(balances.spendable_onchain_balance_sats)
    }

    pub fn get_total_onchain_balance(&self) -> Result<u64, LightningError> {
        let node_guard = self.node.lock()
            .map_err(|e| LightningError::LdkError(format!("Lock error: {}", e)))?;

        let node = node_guard.as_ref()
            .ok_or(LightningError::NotInitialized)?;

        let balances = node.list_balances();
        Ok(balances.total_onchain_balance_sats)
    }

    pub fn get_pending_sweep_balance(&self) -> Result<u64, LightningError> {
        let node_guard = self.node.lock()
            .map_err(|e| LightningError::LdkError(format!("Lock error: {}", e)))?;

        let node = node_guard.as_ref()
            .ok_or(LightningError::NotInitialized)?;

        let balances = node.list_balances();

        // Sum up all pending balances from channel closures
        let pending_total: u64 = balances.pending_balances_from_channel_closures
            .iter()
            .map(|balance| {
                match balance {
                    ldk_node::PendingSweepBalance::PendingBroadcast { amount_satoshis, .. } => *amount_satoshis,
                    ldk_node::PendingSweepBalance::BroadcastAwaitingConfirmation { amount_satoshis, .. } => *amount_satoshis,
                    ldk_node::PendingSweepBalance::AwaitingThresholdConfirmations { amount_satoshis, .. } => *amount_satoshis,
                }
            })
            .sum();

        Ok(pending_total)
    }

    pub fn get_pending_sweep_balances_breakdown(&self) -> Result<(u64, u64, u64), LightningError> {
        let node_guard = self.node.lock()
            .map_err(|e| LightningError::LdkError(format!("Lock error: {}", e)))?;

        let node = node_guard.as_ref()
            .ok_or(LightningError::NotInitialized)?;

        let balances = node.list_balances();

        let mut pending_broadcast = 0u64;
        let mut broadcast_awaiting_conf = 0u64;
        let mut awaiting_threshold_conf = 0u64;

        for balance in &balances.pending_balances_from_channel_closures {
            match balance {
                ldk_node::PendingSweepBalance::PendingBroadcast { amount_satoshis, .. } => {
                    pending_broadcast += amount_satoshis;
                }
                ldk_node::PendingSweepBalance::BroadcastAwaitingConfirmation { amount_satoshis, .. } => {
                    broadcast_awaiting_conf += amount_satoshis;
                }
                ldk_node::PendingSweepBalance::AwaitingThresholdConfirmations { amount_satoshis, .. } => {
                    awaiting_threshold_conf += amount_satoshis;
                }
            }
        }

        Ok((pending_broadcast, broadcast_awaiting_conf, awaiting_threshold_conf))
    }

    // Get all balance info in one call to list_balances()
    pub fn get_all_balances(&self) -> Result<(u64, u64, u64, u64, u64, u64), LightningError> {
        let node_guard = self.node.lock()
            .map_err(|e| LightningError::LdkError(format!("Lock error: {}", e)))?;

        let node = node_guard.as_ref()
            .ok_or(LightningError::NotInitialized)?;

        // Call list_balances ONCE
        let balances = node.list_balances();

        // Extract on-chain balances
        let total_onchain = balances.total_onchain_balance_sats;
        let spendable_onchain = balances.spendable_onchain_balance_sats;

        // Calculate pending sweep breakdown
        let mut pending_broadcast = 0u64;
        let mut broadcast_awaiting_conf = 0u64;
        let mut awaiting_threshold_conf = 0u64;

        for balance in &balances.pending_balances_from_channel_closures {
            match balance {
                ldk_node::PendingSweepBalance::PendingBroadcast { amount_satoshis, .. } => {
                    pending_broadcast += amount_satoshis;
                }
                ldk_node::PendingSweepBalance::BroadcastAwaitingConfirmation { amount_satoshis, .. } => {
                    broadcast_awaiting_conf += amount_satoshis;
                }
                ldk_node::PendingSweepBalance::AwaitingThresholdConfirmations { amount_satoshis, .. } => {
                    awaiting_threshold_conf += amount_satoshis;
                }
            }
        }

        // Return: (total_onchain, spendable_onchain, pending_broadcast, broadcast_awaiting, awaiting_threshold, lightning_balance)
        // Note: lightning_balance will be calculated from channels separately
        Ok((total_onchain, spendable_onchain, pending_broadcast, broadcast_awaiting_conf, awaiting_threshold_conf, balances.total_lightning_balance_sats))
    }
}

impl Default for LightningNode {
    fn default() -> Self {
        Self::new()
    }
}
