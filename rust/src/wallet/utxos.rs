use bitcoin::{OutPoint, TxOut, Txid};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UtxoError {
    #[error("UTXO not found")]
    NotFound,
    #[error("Invalid UTXO data: {0}")]
    InvalidData(String),
    #[error("Storage error: {0}")]
    Storage(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Utxo {
    pub outpoint: OutPointData,
    pub txout: TxOutData,
    pub height: Option<u32>,
    pub address: String,
    pub derivation_index: u32,
    pub is_change: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutPointData {
    pub txid: String,
    pub vout: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxOutData {
    pub value: u64,
    pub script_pubkey: String,
}

impl Utxo {
    pub fn to_bitcoin_outpoint(&self) -> Result<OutPoint, UtxoError> {
        let txid = Txid::from_str(&self.outpoint.txid)
            .map_err(|e| UtxoError::InvalidData(format!("Invalid txid: {}", e)))?;

        Ok(OutPoint {
            txid,
            vout: self.outpoint.vout,
        })
    }

    pub fn to_bitcoin_txout(&self) -> Result<TxOut, UtxoError> {
        let script_bytes = hex::decode(&self.txout.script_pubkey)
            .map_err(|e| UtxoError::InvalidData(format!("Invalid script: {}", e)))?;

        Ok(TxOut {
            value: bitcoin::Amount::from_sat(self.txout.value),
            script_pubkey: bitcoin::ScriptBuf::from_bytes(script_bytes),
        })
    }
}

pub struct UtxoManager {
    utxos: Arc<Mutex<HashMap<String, Utxo>>>, // key: "txid:vout"
}

impl UtxoManager {
    pub fn new() -> Self {
        UtxoManager {
            utxos: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn utxo_key(txid: &str, vout: u32) -> String {
        format!("{}:{}", txid, vout)
    }

    pub fn add_utxo(&self, utxo: Utxo) -> Result<(), UtxoError> {
        let key = Self::utxo_key(&utxo.outpoint.txid, utxo.outpoint.vout);

        let mut utxos = self.utxos.lock()
            .map_err(|e| UtxoError::Storage(format!("Lock error: {}", e)))?;

        utxos.insert(key, utxo);
        Ok(())
    }

    pub fn remove_utxo(&self, txid: &str, vout: u32) -> Result<Option<Utxo>, UtxoError> {
        let key = Self::utxo_key(txid, vout);

        let mut utxos = self.utxos.lock()
            .map_err(|e| UtxoError::Storage(format!("Lock error: {}", e)))?;

        Ok(utxos.remove(&key))
    }

    pub fn get_utxo(&self, txid: &str, vout: u32) -> Result<Option<Utxo>, UtxoError> {
        let key = Self::utxo_key(txid, vout);

        let utxos = self.utxos.lock()
            .map_err(|e| UtxoError::Storage(format!("Lock error: {}", e)))?;

        Ok(utxos.get(&key).cloned())
    }

    pub fn get_all_utxos(&self) -> Result<Vec<Utxo>, UtxoError> {
        let utxos = self.utxos.lock()
            .map_err(|e| UtxoError::Storage(format!("Lock error: {}", e)))?;

        Ok(utxos.values().cloned().collect())
    }

    pub fn get_confirmed_utxos(&self) -> Result<Vec<Utxo>, UtxoError> {
        let utxos = self.utxos.lock()
            .map_err(|e| UtxoError::Storage(format!("Lock error: {}", e)))?;

        Ok(utxos.values()
            .filter(|utxo| utxo.height.is_some())
            .cloned()
            .collect())
    }

    pub fn get_unconfirmed_utxos(&self) -> Result<Vec<Utxo>, UtxoError> {
        let utxos = self.utxos.lock()
            .map_err(|e| UtxoError::Storage(format!("Lock error: {}", e)))?;

        Ok(utxos.values()
            .filter(|utxo| utxo.height.is_none())
            .cloned()
            .collect())
    }

    pub fn get_total_balance(&self) -> Result<u64, UtxoError> {
        let utxos = self.get_all_utxos()?;
        Ok(utxos.iter().map(|u| u.txout.value).sum())
    }

    pub fn get_confirmed_balance(&self) -> Result<u64, UtxoError> {
        let utxos = self.get_confirmed_utxos()?;
        Ok(utxos.iter().map(|u| u.txout.value).sum())
    }

    pub fn get_unconfirmed_balance(&self) -> Result<u64, UtxoError> {
        let utxos = self.get_unconfirmed_utxos()?;
        Ok(utxos.iter().map(|u| u.txout.value).sum())
    }

    pub fn clear(&self) -> Result<(), UtxoError> {
        let mut utxos = self.utxos.lock()
            .map_err(|e| UtxoError::Storage(format!("Lock error: {}", e)))?;

        utxos.clear();
        Ok(())
    }

    /// Select UTXOs for a transaction using a simple largest-first strategy
    pub fn select_utxos(&self, target_amount: u64, fee_per_vbyte: u64) -> Result<Vec<Utxo>, UtxoError> {
        let mut available_utxos = self.get_confirmed_utxos()?;

        // Sort by value descending (largest first)
        available_utxos.sort_by(|a, b| b.txout.value.cmp(&a.txout.value));

        let mut selected = Vec::new();
        let mut total_value = 0u64;

        // Estimate fee: base + (inputs * 68) + (outputs * 31)
        // We'll use 2 outputs (recipient + change)
        let mut estimated_size = 10 + (2 * 31); // base + outputs

        for utxo in available_utxos {
            selected.push(utxo.clone());
            total_value += utxo.txout.value;

            // Update estimated size
            estimated_size += 68; // approximate input size for P2WPKH
            let estimated_fee = (estimated_size as u64) * fee_per_vbyte;

            if total_value >= target_amount + estimated_fee {
                return Ok(selected);
            }
        }

        Err(UtxoError::InvalidData(
            format!("Insufficient funds: have {}, need {}", total_value, target_amount)
        ))
    }
}

impl Default for UtxoManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_utxo(value: u64, index: u32) -> Utxo {
        Utxo {
            outpoint: OutPointData {
                txid: format!("{:064x}", index),
                vout: 0,
            },
            txout: TxOutData {
                value,
                script_pubkey: "0014abcd".to_string(),
            },
            height: Some(100),
            address: "tb1test".to_string(),
            derivation_index: 0,
            is_change: false,
        }
    }

    #[test]
    fn test_add_and_get_utxo() {
        let manager = UtxoManager::new();
        let utxo = create_test_utxo(100000, 1);

        manager.add_utxo(utxo.clone()).unwrap();

        let retrieved = manager.get_utxo(&utxo.outpoint.txid, utxo.outpoint.vout).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().txout.value, 100000);
    }

    #[test]
    fn test_balance_calculation() {
        let manager = UtxoManager::new();

        manager.add_utxo(create_test_utxo(100000, 1)).unwrap();
        manager.add_utxo(create_test_utxo(200000, 2)).unwrap();
        manager.add_utxo(create_test_utxo(300000, 3)).unwrap();

        let balance = manager.get_total_balance().unwrap();
        assert_eq!(balance, 600000);
    }

    #[test]
    fn test_utxo_selection() {
        let manager = UtxoManager::new();

        manager.add_utxo(create_test_utxo(100000, 1)).unwrap();
        manager.add_utxo(create_test_utxo(200000, 2)).unwrap();
        manager.add_utxo(create_test_utxo(300000, 3)).unwrap();

        let selected = manager.select_utxos(250000, 1).unwrap();
        assert!(!selected.is_empty());

        let total: u64 = selected.iter().map(|u| u.txout.value).sum();
        assert!(total >= 250000);
    }
}
