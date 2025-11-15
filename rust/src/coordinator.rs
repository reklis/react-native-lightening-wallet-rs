use std::sync::{Arc, Mutex};
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
            BitcoinNetwork::Bitcoin => (ldk_node::bitcoin::Network::Bitcoin, "https://blockstream.info/api"),
            BitcoinNetwork::Testnet => (ldk_node::bitcoin::Network::Testnet, "https://blockstream.info/testnet/api"),
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

    /// Stop the Lightning node and cleanup
    pub fn disconnect(&self) -> Result<(), CoordinatorError> {
        self.lightning_node.stop()
            .map_err(|e| CoordinatorError::LightningError(format!("{}", e)))?;

        let mut km = self.key_manager.lock()
            .map_err(|e| CoordinatorError::KeyError(format!("Lock error: {}", e)))?;
        *km = None;

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
