use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use bitcoin::Network as BitcoinNetwork;
use thiserror::Error;

use crate::wallet::KeyManager;
use crate::storage::Database;
use crate::events::EventEmitter;
use crate::ldk::LightningNode;

#[derive(Debug, Error)]
pub enum CoordinatorError {
    #[error("Already initialized")]
    AlreadyInitialized,
    #[error("Key error: {0}")]
    KeyError(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Lightning error: {0}")]
    LightningError(String),
}

pub struct WalletCoordinator {
    key_manager: Arc<Mutex<Option<KeyManager>>>,
    lightning_node: Arc<LightningNode>,
    database: Arc<Database>,
    event_emitter: Arc<EventEmitter>,
    network: Arc<Mutex<Option<BitcoinNetwork>>>,
    storage_path: String,
    sync_thread_stop: Arc<AtomicBool>,
}

impl WalletCoordinator {
    pub fn new(db_path: &str) -> Result<Self, CoordinatorError> {
        let database = Database::new(db_path)
            .map_err(|e| CoordinatorError::StorageError(format!("{}", e)))?;

        // Extract directory path for Lightning storage
        let storage_path = std::path::Path::new(db_path)
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .to_string_lossy()
            .to_string();

        Ok(WalletCoordinator {
            key_manager: Arc::new(Mutex::new(None)),
            lightning_node: Arc::new(LightningNode::new()),
            database: Arc::new(database),
            event_emitter: Arc::new(EventEmitter::new(1000)),
            network: Arc::new(Mutex::new(None)),
            storage_path,
            sync_thread_stop: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn initialize(
        &self,
        mnemonic: &str,
        network: BitcoinNetwork,
    ) -> Result<(), CoordinatorError> {
        // Check if already initialized
        {
            let km = self.key_manager.lock()
                .map_err(|e| CoordinatorError::KeyError(format!("Lock error: {}", e)))?;

            if km.is_some() {
                return Err(CoordinatorError::AlreadyInitialized);
            }
        }

        // Initialize key manager
        let key_manager = KeyManager::from_mnemonic(mnemonic, network)
            .map_err(|e| CoordinatorError::KeyError(format!("{}", e)))?;

        // Get Lightning seed before moving key_manager
        let lightning_seed = key_manager.get_lightning_seed()
            .map_err(|e| CoordinatorError::KeyError(format!("{}", e)))?;

        {
            let mut km = self.key_manager.lock()
                .map_err(|e| CoordinatorError::KeyError(format!("Lock error: {}", e)))?;
            *km = Some(key_manager);
        }

        {
            let mut net = self.network.lock()
                .map_err(|e| CoordinatorError::KeyError(format!("Lock error: {}", e)))?;
            *net = Some(network);
        }

        // Initialize Lightning node (includes on-chain wallet via BDK)
        let lightning_storage = format!("{}/lightning", self.storage_path);
        // Convert bitcoin::Network to ldk_node::bitcoin::Network
        let (ldk_network, esplora_url) = match network {
            BitcoinNetwork::Bitcoin => (ldk_node::bitcoin::Network::Bitcoin, "https://mempool.space/api"),
            BitcoinNetwork::Testnet => (ldk_node::bitcoin::Network::Testnet, "https://mempool.space/testnet/api"),
            BitcoinNetwork::Signet => (ldk_node::bitcoin::Network::Signet, "https://mempool.space/signet/api"),
            BitcoinNetwork::Regtest => (ldk_node::bitcoin::Network::Regtest, "http://localhost:3000"),
            _ => return Err(CoordinatorError::LightningError("Unsupported network".to_string())),
        };
        self.lightning_node.initialize(
            lightning_seed,
            ldk_network,
            &lightning_storage,
            Some(esplora_url),
        ).map_err(|e| CoordinatorError::LightningError(format!("{}", e)))?;

        // Start Lightning node
        self.lightning_node.start()
            .map_err(|e| CoordinatorError::LightningError(format!("{}", e)))?;

        // Start background payment sync thread
        self.start_payment_sync_thread();

        Ok(())
    }

    /// Sync the Lightning node's wallets with the blockchain
    pub fn sync(&self) -> Result<(), CoordinatorError> {
        self.lightning_node.sync_wallets()
            .map_err(|e| CoordinatorError::LightningError(format!("{}", e)))
    }

    /// Get Lightning node for direct access to methods
    pub fn get_lightning_node(&self) -> Arc<LightningNode> {
        Arc::clone(&self.lightning_node)
    }

    /// Get database for direct access
    pub fn get_database(&self) -> Arc<Database> {
        Arc::clone(&self.database)
    }

    /// Get event emitter for listening to events
    pub fn get_event_emitter(&self) -> Arc<EventEmitter> {
        Arc::clone(&self.event_emitter)
    }

    /// Start background thread for automatic payment syncing
    /// Syncs every 30 seconds to keep local cache up-to-date
    fn start_payment_sync_thread(&self) {
        // Reset stop flag
        self.sync_thread_stop.store(false, Ordering::Relaxed);

        let lightning_node = Arc::clone(&self.lightning_node);
        let database = Arc::clone(&self.database);
        let stop_flag = Arc::clone(&self.sync_thread_stop);

        thread::spawn(move || {
            println!("[Payment Sync] Background thread started");

            // Initial sync after 5 seconds (give wallet time to fully initialize)
            thread::sleep(Duration::from_secs(5));

            loop {
                // Check if we should stop
                if stop_flag.load(Ordering::Relaxed) {
                    println!("[Payment Sync] Background thread stopping");
                    break;
                }

                // Sync payments from network to cache
                match lightning_node.list_payments() {
                    Ok(payments) => {
                        let mut synced_count = 0;

                        for payment_info in payments {
                            let db_payment = crate::storage::db::Payment {
                                id: None,
                                payment_hash: payment_info.payment_hash.clone(),
                                payment_type: match payment_info.payment_type {
                                    crate::ldk::PaymentType::Sent => "sent".to_string(),
                                    crate::ldk::PaymentType::Received => "received".to_string(),
                                    crate::ldk::PaymentType::OnchainSent => "onchain_sent".to_string(),
                                    crate::ldk::PaymentType::OnchainReceived => "onchain_received".to_string(),
                                },
                                amount_sats: payment_info.amount_sats as i64,
                                fee_sats: payment_info.fee_sats.map(|f| f as i64),
                                status: match payment_info.status {
                                    crate::ldk::PaymentStatus::Pending => "pending".to_string(),
                                    crate::ldk::PaymentStatus::Completed => "completed".to_string(),
                                    crate::ldk::PaymentStatus::Failed => "failed".to_string(),
                                },
                                timestamp: payment_info.timestamp as i64,
                                description: payment_info.description,
                                destination: payment_info.destination,
                                txid: None,
                                preimage: payment_info.preimage,
                                bolt11: payment_info.bolt11,
                            };

                            if let Err(e) = database.upsert_payment(&db_payment) {
                                eprintln!("[Payment Sync] Failed to upsert payment: {}", e);
                            } else {
                                synced_count += 1;
                            }
                        }

                        if synced_count > 0 {
                            println!("[Payment Sync] Synced {} payments to cache", synced_count);
                        }
                    }
                    Err(e) => {
                        eprintln!("[Payment Sync] Failed to fetch payments: {}", e);
                    }
                }

                // Sleep for 30 seconds before next sync
                for _ in 0..30 {
                    if stop_flag.load(Ordering::Relaxed) {
                        println!("[Payment Sync] Background thread stopping");
                        return;
                    }
                    thread::sleep(Duration::from_secs(1));
                }
            }

            println!("[Payment Sync] Background thread stopped");
        });
    }

    /// Sync payments from the Lightning network to local database cache
    /// This fetches payment history from ldk-node and updates the local SQLite cache
    /// This is now called automatically by the background thread, but kept for manual syncs
    pub fn sync_payments(&self) -> Result<usize, CoordinatorError> {
        // Fetch payments from ldk-node (network source)
        let payments = self.lightning_node.list_payments()
            .map_err(|e| CoordinatorError::LightningError(format!("Failed to list payments: {}", e)))?;

        let mut synced_count = 0;

        // Convert and upsert each payment to the database
        for payment_info in payments {
            let db_payment = crate::storage::db::Payment {
                id: None, // Auto-generated by database
                payment_hash: payment_info.payment_hash.clone(),
                payment_type: match payment_info.payment_type {
                    crate::ldk::PaymentType::Sent => "sent".to_string(),
                    crate::ldk::PaymentType::Received => "received".to_string(),
                    crate::ldk::PaymentType::OnchainSent => "onchain_sent".to_string(),
                    crate::ldk::PaymentType::OnchainReceived => "onchain_received".to_string(),
                },
                amount_sats: payment_info.amount_sats as i64,
                fee_sats: payment_info.fee_sats.map(|f| f as i64),
                status: match payment_info.status {
                    crate::ldk::PaymentStatus::Pending => "pending".to_string(),
                    crate::ldk::PaymentStatus::Completed => "completed".to_string(),
                    crate::ldk::PaymentStatus::Failed => "failed".to_string(),
                },
                timestamp: payment_info.timestamp as i64,
                description: payment_info.description,
                destination: payment_info.destination,
                txid: None, // Not available in PaymentInfo
                preimage: payment_info.preimage,
                bolt11: payment_info.bolt11,
            };

            // Upsert to database (insert if new, update if exists)
            self.database.upsert_payment(&db_payment)
                .map_err(|e| CoordinatorError::StorageError(format!("Failed to upsert payment: {}", e)))?;

            synced_count += 1;
        }

        Ok(synced_count)
    }

    /// Stop the Lightning node and cleanup
    pub fn disconnect(&self) -> Result<(), CoordinatorError> {
        // Stop the background payment sync thread
        println!("[Payment Sync] Signaling background thread to stop");
        self.sync_thread_stop.store(true, Ordering::Relaxed);

        // Give the thread a moment to finish its current iteration
        thread::sleep(Duration::from_millis(500));

        // Stop the Lightning node
        self.lightning_node.stop()
            .map_err(|e| CoordinatorError::LightningError(format!("{}", e)))?;

        // Clear the key manager
        let mut km = self.key_manager.lock()
            .map_err(|e| CoordinatorError::KeyError(format!("Lock error: {}", e)))?;
        *km = None;

        // Delete the Lightning storage directory
        let lightning_storage = format!("{}/lightning", self.storage_path);
        if std::path::Path::new(&lightning_storage).exists() {
            std::fs::remove_dir_all(&lightning_storage)
                .map_err(|e| CoordinatorError::StorageError(format!("Failed to delete Lightning storage: {}", e)))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinator_creation() {
        let coord = WalletCoordinator::new(":memory:");
        assert!(coord.is_ok());
    }
}
