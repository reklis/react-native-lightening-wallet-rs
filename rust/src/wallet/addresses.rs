use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use bitcoin::Address;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AddressError {
    #[error("Address not found")]
    NotFound,
    #[error("Storage error: {0}")]
    Storage(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressInfo {
    pub address: String,
    pub derivation_index: u32,
    pub is_change: bool,
    pub used: bool,
    pub balance: u64,
    pub tx_count: u32,
}

pub struct AddressManager {
    receiving_addresses: Arc<Mutex<HashMap<u32, AddressInfo>>>,
    change_addresses: Arc<Mutex<HashMap<u32, AddressInfo>>>,
    address_to_index: Arc<Mutex<HashMap<String, (u32, bool)>>>, // address -> (index, is_change)
}

impl AddressManager {
    pub fn new() -> Self {
        AddressManager {
            receiving_addresses: Arc::new(Mutex::new(HashMap::new())),
            change_addresses: Arc::new(Mutex::new(HashMap::new())),
            address_to_index: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_receiving_address(&self, index: u32, address: String) -> Result<(), AddressError> {
        let info = AddressInfo {
            address: address.clone(),
            derivation_index: index,
            is_change: false,
            used: false,
            balance: 0,
            tx_count: 0,
        };

        let mut receiving = self.receiving_addresses.lock()
            .map_err(|e| AddressError::Storage(format!("Lock error: {}", e)))?;

        let mut address_map = self.address_to_index.lock()
            .map_err(|e| AddressError::Storage(format!("Lock error: {}", e)))?;

        receiving.insert(index, info);
        address_map.insert(address, (index, false));

        Ok(())
    }

    pub fn add_change_address(&self, index: u32, address: String) -> Result<(), AddressError> {
        let info = AddressInfo {
            address: address.clone(),
            derivation_index: index,
            is_change: true,
            used: false,
            balance: 0,
            tx_count: 0,
        };

        let mut change = self.change_addresses.lock()
            .map_err(|e| AddressError::Storage(format!("Lock error: {}", e)))?;

        let mut address_map = self.address_to_index.lock()
            .map_err(|e| AddressError::Storage(format!("Lock error: {}", e)))?;

        change.insert(index, info);
        address_map.insert(address, (index, true));

        Ok(())
    }

    pub fn get_receiving_address(&self, index: u32) -> Result<Option<AddressInfo>, AddressError> {
        let receiving = self.receiving_addresses.lock()
            .map_err(|e| AddressError::Storage(format!("Lock error: {}", e)))?;

        Ok(receiving.get(&index).cloned())
    }

    pub fn get_change_address(&self, index: u32) -> Result<Option<AddressInfo>, AddressError> {
        let change = self.change_addresses.lock()
            .map_err(|e| AddressError::Storage(format!("Lock error: {}", e)))?;

        Ok(change.get(&index).cloned())
    }

    pub fn get_address_info(&self, address: &str) -> Result<Option<AddressInfo>, AddressError> {
        let address_map = self.address_to_index.lock()
            .map_err(|e| AddressError::Storage(format!("Lock error: {}", e)))?;

        if let Some((index, is_change)) = address_map.get(address) {
            if *is_change {
                self.get_change_address(*index)
            } else {
                self.get_receiving_address(*index)
            }
        } else {
            Ok(None)
        }
    }

    pub fn mark_used(&self, address: &str) -> Result<(), AddressError> {
        let address_map = self.address_to_index.lock()
            .map_err(|e| AddressError::Storage(format!("Lock error: {}", e)))?;

        if let Some((index, is_change)) = address_map.get(address) {
            if *is_change {
                let mut change = self.change_addresses.lock()
                    .map_err(|e| AddressError::Storage(format!("Lock error: {}", e)))?;

                if let Some(info) = change.get_mut(index) {
                    info.used = true;
                }
            } else {
                let mut receiving = self.receiving_addresses.lock()
                    .map_err(|e| AddressError::Storage(format!("Lock error: {}", e)))?;

                if let Some(info) = receiving.get_mut(index) {
                    info.used = true;
                }
            }
        }

        Ok(())
    }

    pub fn update_balance(&self, address: &str, balance: u64, tx_count: u32) -> Result<(), AddressError> {
        let address_map = self.address_to_index.lock()
            .map_err(|e| AddressError::Storage(format!("Lock error: {}", e)))?;

        if let Some((index, is_change)) = address_map.get(address) {
            if *is_change {
                let mut change = self.change_addresses.lock()
                    .map_err(|e| AddressError::Storage(format!("Lock error: {}", e)))?;

                if let Some(info) = change.get_mut(index) {
                    info.balance = balance;
                    info.tx_count = tx_count;
                    if tx_count > 0 {
                        info.used = true;
                    }
                }
            } else {
                let mut receiving = self.receiving_addresses.lock()
                    .map_err(|e| AddressError::Storage(format!("Lock error: {}", e)))?;

                if let Some(info) = receiving.get_mut(index) {
                    info.balance = balance;
                    info.tx_count = tx_count;
                    if tx_count > 0 {
                        info.used = true;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn get_next_unused_receiving_index(&self) -> Result<u32, AddressError> {
        let receiving = self.receiving_addresses.lock()
            .map_err(|e| AddressError::Storage(format!("Lock error: {}", e)))?;

        let mut max_index = 0u32;
        for (index, info) in receiving.iter() {
            if *index > max_index {
                max_index = *index;
            }
            if !info.used {
                return Ok(*index);
            }
        }

        Ok(max_index + 1)
    }

    pub fn get_next_unused_change_index(&self) -> Result<u32, AddressError> {
        let change = self.change_addresses.lock()
            .map_err(|e| AddressError::Storage(format!("Lock error: {}", e)))?;

        let mut max_index = 0u32;
        for (index, info) in change.iter() {
            if *index > max_index {
                max_index = *index;
            }
            if !info.used {
                return Ok(*index);
            }
        }

        Ok(max_index + 1)
    }

    pub fn get_all_receiving_addresses(&self) -> Result<Vec<AddressInfo>, AddressError> {
        let receiving = self.receiving_addresses.lock()
            .map_err(|e| AddressError::Storage(format!("Lock error: {}", e)))?;

        Ok(receiving.values().cloned().collect())
    }

    pub fn get_all_change_addresses(&self) -> Result<Vec<AddressInfo>, AddressError> {
        let change = self.change_addresses.lock()
            .map_err(|e| AddressError::Storage(format!("Lock error: {}", e)))?;

        Ok(change.values().cloned().collect())
    }

    pub fn get_all_addresses(&self) -> Result<Vec<AddressInfo>, AddressError> {
        let mut all = self.get_all_receiving_addresses()?;
        all.extend(self.get_all_change_addresses()?);
        Ok(all)
    }

    pub fn clear(&self) -> Result<(), AddressError> {
        let mut receiving = self.receiving_addresses.lock()
            .map_err(|e| AddressError::Storage(format!("Lock error: {}", e)))?;

        let mut change = self.change_addresses.lock()
            .map_err(|e| AddressError::Storage(format!("Lock error: {}", e)))?;

        let mut address_map = self.address_to_index.lock()
            .map_err(|e| AddressError::Storage(format!("Lock error: {}", e)))?;

        receiving.clear();
        change.clear();
        address_map.clear();

        Ok(())
    }
}

impl Default for AddressManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_get_address() {
        let manager = AddressManager::new();

        manager.add_receiving_address(0, "tb1test0".to_string()).unwrap();
        manager.add_receiving_address(1, "tb1test1".to_string()).unwrap();

        let info = manager.get_receiving_address(0).unwrap();
        assert!(info.is_some());
        assert_eq!(info.unwrap().address, "tb1test0");
    }

    #[test]
    fn test_mark_used() {
        let manager = AddressManager::new();

        manager.add_receiving_address(0, "tb1test0".to_string()).unwrap();

        let info = manager.get_receiving_address(0).unwrap().unwrap();
        assert!(!info.used);

        manager.mark_used("tb1test0").unwrap();

        let info = manager.get_receiving_address(0).unwrap().unwrap();
        assert!(info.used);
    }

    #[test]
    fn test_next_unused_index() {
        let manager = AddressManager::new();

        manager.add_receiving_address(0, "tb1test0".to_string()).unwrap();
        manager.add_receiving_address(1, "tb1test1".to_string()).unwrap();

        manager.mark_used("tb1test0").unwrap();

        let next = manager.get_next_unused_receiving_index().unwrap();
        assert_eq!(next, 1);
    }
}
