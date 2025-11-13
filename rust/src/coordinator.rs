use std::sync::{Arc, Mutex};
use bitcoin::Network;
use thiserror::Error;

use crate::wallet::{KeyManager, UtxoManager, AddressManager, BalanceManager, WalletBalance};
use crate::electrum::{ElectrumClient, get_default_peers};
use crate::storage::Database;
use crate::events::{EventEmitter, WalletEvent};

#[derive(Debug, Error)]
pub enum CoordinatorError {
    #[error("Not initialized")]
    NotInitialized,
    #[error("Already initialized")]
    AlreadyInitialized,
    #[error("Key error: {0}")]
    KeyError(String),
    #[error("Electrum error: {0}")]
    ElectrumError(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Sync error: {0}")]
    SyncError(String),
}

pub struct WalletCoordinator {
    key_manager: Arc<Mutex<Option<KeyManager>>>,
    utxo_manager: Arc<UtxoManager>,
    address_manager: Arc<AddressManager>,
    balance_manager: Arc<BalanceManager>,
    electrum_client: Arc<Mutex<Option<ElectrumClient>>>,
    database: Arc<Database>,
    event_emitter: Arc<EventEmitter>,
    network: Arc<Mutex<Option<Network>>>,
}

impl WalletCoordinator {
    pub fn new(db_path: &str) -> Result<Self, CoordinatorError> {
        let database = Database::new(db_path)
            .map_err(|e| CoordinatorError::StorageError(format!("{}", e)))?;

        Ok(WalletCoordinator {
            key_manager: Arc::new(Mutex::new(None)),
            utxo_manager: Arc::new(UtxoManager::new()),
            address_manager: Arc::new(AddressManager::new()),
            balance_manager: Arc::new(BalanceManager::new()),
            electrum_client: Arc::new(Mutex::new(None)),
            database: Arc::new(database),
            event_emitter: Arc::new(EventEmitter::new(1000)),
            network: Arc::new(Mutex::new(None)),
        })
    }

    pub fn initialize(
        &self,
        mnemonic: &str,
        network: Network,
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

        // Connect to Electrum
        self.connect_electrum(network)?;

        // Initialize addresses
        self.derive_initial_addresses()?;

        Ok(())
    }

    fn connect_electrum(&self, network: Network) -> Result<(), CoordinatorError> {
        let network_str = match network {
            Network::Bitcoin => "bitcoin",
            Network::Testnet => "testnet",
            _ => return Err(CoordinatorError::ElectrumError("Unsupported network".to_string())),
        };

        let peers = get_default_peers(network_str);

        for peer in peers {
            if let Some(port) = peer.ssl {
                match ElectrumClient::connect(&peer.host, port) {
                    Ok(client) => {
                        let mut ec = self.electrum_client.lock()
                            .map_err(|e| CoordinatorError::ElectrumError(format!("Lock error: {}", e)))?;
                        *ec = Some(client);
                        return Ok(());
                    }
                    Err(_) => continue,
                }
            }
        }

        Err(CoordinatorError::ElectrumError("Failed to connect to any Electrum server".to_string()))
    }

    fn derive_initial_addresses(&self) -> Result<(), CoordinatorError> {
        let km = self.key_manager.lock()
            .map_err(|e| CoordinatorError::KeyError(format!("Lock error: {}", e)))?;

        let key_manager = km.as_ref()
            .ok_or(CoordinatorError::NotInitialized)?;

        // Derive first 20 receiving addresses
        for i in 0..20 {
            let address = key_manager.get_receiving_address(i)
                .map_err(|e| CoordinatorError::KeyError(format!("{}", e)))?;

            self.address_manager.add_receiving_address(i, address.to_string())
                .map_err(|e| CoordinatorError::StorageError(format!("{}", e)))?;
        }

        // Derive first 20 change addresses
        for i in 0..20 {
            let address = key_manager.get_change_address(i)
                .map_err(|e| CoordinatorError::KeyError(format!("{}", e)))?;

            self.address_manager.add_change_address(i, address.to_string())
                .map_err(|e| CoordinatorError::StorageError(format!("{}", e)))?;
        }

        Ok(())
    }

    pub fn sync(&self) -> Result<(), CoordinatorError> {
        self.event_emitter.emit(WalletEvent::SyncStarted);

        let start = std::time::Instant::now();

        // Scan addresses for transactions and update UTXO set
        // This is a simplified version - production would need more sophisticated scanning
        self.scan_addresses()?;

        // Update balance
        self.update_balance()?;

        let duration = start.elapsed().as_millis() as u64;
        self.event_emitter.emit(WalletEvent::SyncCompleted { duration_ms: duration });

        Ok(())
    }

    fn scan_addresses(&self) -> Result<(), CoordinatorError> {
        let ec = self.electrum_client.lock()
            .map_err(|e| CoordinatorError::ElectrumError(format!("Lock error: {}", e)))?;

        let client = ec.as_ref()
            .ok_or(CoordinatorError::NotInitialized)?;

        let addresses = self.address_manager.get_all_addresses()
            .map_err(|e| CoordinatorError::StorageError(format!("{}", e)))?;

        for address_info in addresses {
            // Convert address to script hash for Electrum
            let script_hash = self.address_to_script_hash(&address_info.address)?;

            // Get history for this address
            match client.get_history(&script_hash) {
                Ok(history) => {
                    if let Some(history_array) = history.as_array() {
                        if !history_array.is_empty() {
                            self.address_manager.mark_used(&address_info.address)
                                .map_err(|e| CoordinatorError::StorageError(format!("{}", e)))?;
                        }
                        // TODO: Process transactions and update UTXO set
                    }
                }
                Err(_) => continue,
            }
        }

        Ok(())
    }

    fn address_to_script_hash(&self, address: &str) -> Result<String, CoordinatorError> {
        use bitcoin::Address;
        use std::str::FromStr;

        let addr = Address::from_str(address)
            .map_err(|e| CoordinatorError::SyncError(format!("Invalid address: {}", e)))?
            .assume_checked();

        let script = addr.script_pubkey();
        let script_bytes = script.as_bytes();

        // Hash the script with SHA256 and reverse for Electrum
        use bitcoin::hashes::{sha256, Hash};
        let hash = sha256::Hash::hash(script_bytes);
        let mut hash_bytes = hash.to_byte_array();
        hash_bytes.reverse();

        Ok(hex::encode(hash_bytes))
    }

    fn update_balance(&self) -> Result<(), CoordinatorError> {
        let confirmed = self.utxo_manager.get_confirmed_balance()
            .map_err(|e| CoordinatorError::StorageError(format!("{}", e)))?;

        let unconfirmed = self.utxo_manager.get_unconfirmed_balance()
            .map_err(|e| CoordinatorError::StorageError(format!("{}", e)))?;

        self.balance_manager.update_onchain(confirmed, unconfirmed)
            .map_err(|e| CoordinatorError::StorageError(format!("{}", e)))?;

        let balance = self.balance_manager.get_balance()
            .map_err(|e| CoordinatorError::StorageError(format!("{}", e)))?;

        self.event_emitter.emit(WalletEvent::BalanceUpdated {
            onchain_confirmed: balance.onchain_confirmed,
            onchain_unconfirmed: balance.onchain_unconfirmed,
            lightning_balance: balance.lightning_balance,
            total: balance.total,
        });

        Ok(())
    }

    pub fn get_balance(&self) -> Result<WalletBalance, CoordinatorError> {
        self.balance_manager.get_balance()
            .map_err(|e| CoordinatorError::StorageError(format!("{}", e)))
    }

    pub fn get_receiving_address(&self) -> Result<String, CoordinatorError> {
        let index = self.address_manager.get_next_unused_receiving_index()
            .map_err(|e| CoordinatorError::StorageError(format!("{}", e)))?;

        let address_info = self.address_manager.get_receiving_address(index)
            .map_err(|e| CoordinatorError::StorageError(format!("{}", e)))?;

        if let Some(info) = address_info {
            Ok(info.address)
        } else {
            // Need to derive a new address
            let km = self.key_manager.lock()
                .map_err(|e| CoordinatorError::KeyError(format!("Lock error: {}", e)))?;

            let key_manager = km.as_ref()
                .ok_or(CoordinatorError::NotInitialized)?;

            let address = key_manager.get_receiving_address(index)
                .map_err(|e| CoordinatorError::KeyError(format!("{}", e)))?;

            self.address_manager.add_receiving_address(index, address.to_string())
                .map_err(|e| CoordinatorError::StorageError(format!("{}", e)))?;

            Ok(address.to_string())
        }
    }

    pub fn get_event_emitter(&self) -> Arc<EventEmitter> {
        Arc::clone(&self.event_emitter)
    }

    pub fn get_database(&self) -> Arc<Database> {
        Arc::clone(&self.database)
    }

    pub fn disconnect(&self) -> Result<(), CoordinatorError> {
        let mut ec = self.electrum_client.lock()
            .map_err(|e| CoordinatorError::ElectrumError(format!("Lock error: {}", e)))?;
        *ec = None;

        let mut km = self.key_manager.lock()
            .map_err(|e| CoordinatorError::KeyError(format!("Lock error: {}", e)))?;
        *km = None;

        self.utxo_manager.clear()
            .map_err(|e| CoordinatorError::StorageError(format!("{}", e)))?;

        self.address_manager.clear()
            .map_err(|e| CoordinatorError::StorageError(format!("{}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinator_creation() {
        let coord = WalletCoordinator::new(":memory:").unwrap();
        assert!(coord.get_balance().is_ok());
    }
}
