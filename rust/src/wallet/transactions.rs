use bitcoin::{Address, Transaction, TxIn, TxOut, Sequence, Witness, Amount};
use bitcoin::sighash::{SighashCache, EcdsaSighashType};
use bitcoin::secp256k1::{Secp256k1, Message};
use bitcoin::PrivateKey;
use std::str::FromStr;
use thiserror::Error;

use super::utxos::Utxo;

#[derive(Debug, Error)]
pub enum TxBuilderError {
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
    #[error("Signing error: {0}")]
    SigningError(String),
    #[error("Build error: {0}")]
    BuildError(String),
}

pub struct TransactionBuilder {
    inputs: Vec<(Utxo, PrivateKey)>,
    outputs: Vec<TxOut>,
    fee_rate: u64, // satoshis per vbyte
}

impl TransactionBuilder {
    pub fn new(fee_rate: u64) -> Self {
        TransactionBuilder {
            inputs: Vec::new(),
            outputs: Vec::new(),
            fee_rate,
        }
    }

    pub fn add_input(&mut self, utxo: Utxo, private_key: PrivateKey) -> Result<(), TxBuilderError> {
        self.inputs.push((utxo, private_key));
        Ok(())
    }

    pub fn add_output(&mut self, address: &str, amount: u64) -> Result<(), TxBuilderError> {
        let address = Address::from_str(address)
            .map_err(|e| TxBuilderError::InvalidAddress(format!("{}", e)))?
            .assume_checked();

        if amount == 0 {
            return Err(TxBuilderError::InvalidAmount("Amount must be greater than 0".to_string()));
        }

        self.outputs.push(TxOut {
            value: Amount::from_sat(amount),
            script_pubkey: address.script_pubkey(),
        });

        Ok(())
    }

    pub fn estimate_fee(&self) -> u64 {
        // Estimate transaction size
        // Base: 10 bytes
        // Each input (P2WPKH): ~68 vbytes
        // Each output: ~31 bytes
        let base_size = 10;
        let input_size = self.inputs.len() as u64 * 68;
        let output_size = self.outputs.len() as u64 * 31;

        let estimated_size = base_size + input_size + output_size;
        estimated_size * self.fee_rate
    }

    pub fn get_total_input_value(&self) -> u64 {
        self.inputs.iter().map(|(utxo, _)| utxo.txout.value).sum()
    }

    pub fn get_total_output_value(&self) -> u64 {
        self.outputs.iter().map(|output| output.value.to_sat()).sum()
    }

    pub fn build(&self) -> Result<Transaction, TxBuilderError> {
        if self.inputs.is_empty() {
            return Err(TxBuilderError::BuildError("No inputs".to_string()));
        }

        if self.outputs.is_empty() {
            return Err(TxBuilderError::BuildError("No outputs".to_string()));
        }

        // Create transaction inputs
        let tx_inputs: Vec<TxIn> = self.inputs
            .iter()
            .map(|(utxo, _)| {
                let outpoint = utxo.to_bitcoin_outpoint()
                    .expect("Invalid outpoint");

                TxIn {
                    previous_output: outpoint,
                    script_sig: bitcoin::ScriptBuf::new(),
                    sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
                    witness: Witness::default(),
                }
            })
            .collect();

        // Create unsigned transaction
        let mut tx = Transaction {
            version: bitcoin::transaction::Version(2),
            lock_time: bitcoin::absolute::LockTime::ZERO,
            input: tx_inputs,
            output: self.outputs.clone(),
        };

        Ok(tx)
    }

    pub fn sign(&self) -> Result<Transaction, TxBuilderError> {
        let mut tx = self.build()?;
        let secp = Secp256k1::new();

        // Sign each input
        for (i, (utxo, private_key)) in self.inputs.iter().enumerate() {
            let txout = utxo.to_bitcoin_txout()
                .map_err(|e| TxBuilderError::SigningError(format!("Invalid UTXO: {}", e)))?;

            // Create sighash
            let mut sighash_cache = SighashCache::new(&tx);

            let sighash = sighash_cache
                .p2wpkh_signature_hash(
                    i,
                    &txout.script_pubkey,
                    txout.value,
                    EcdsaSighashType::All,
                )
                .map_err(|e| TxBuilderError::SigningError(format!("Sighash error: {}", e)))?;

            // Sign
            let message = Message::from_digest_slice(sighash.as_ref())
                .map_err(|e| TxBuilderError::SigningError(format!("Message error: {}", e)))?;

            let signature = secp.sign_ecdsa(&message, &private_key.inner);

            // Create witness
            let mut sig_bytes = signature.serialize_der().to_vec();
            sig_bytes.push(EcdsaSighashType::All.to_u32() as u8);

            let pubkey = private_key.public_key(&secp);
            let witness = Witness::from_slice(&[sig_bytes.as_slice(), &pubkey.to_bytes()]);

            tx.input[i].witness = witness;
        }

        Ok(tx)
    }

    pub fn finalize(self) -> Result<Transaction, TxBuilderError> {
        let total_input = self.get_total_input_value();
        let total_output = self.get_total_output_value();
        let fee = self.estimate_fee();

        if total_input < total_output + fee {
            return Err(TxBuilderError::InsufficientFunds);
        }

        self.sign()
    }
}

pub fn create_send_transaction(
    utxos: Vec<Utxo>,
    private_keys: Vec<PrivateKey>,
    recipient_address: &str,
    amount: u64,
    change_address: &str,
    fee_rate: u64,
) -> Result<Transaction, TxBuilderError> {
    if utxos.len() != private_keys.len() {
        return Err(TxBuilderError::BuildError(
            "UTXO count must match private key count".to_string()
        ));
    }

    let mut builder = TransactionBuilder::new(fee_rate);

    // Add inputs
    for (utxo, pk) in utxos.iter().zip(private_keys.iter()) {
        builder.add_input(utxo.clone(), pk.clone())?;
    }

    // Add recipient output
    builder.add_output(recipient_address, amount)?;

    // Calculate change
    let total_input = builder.get_total_input_value();
    let fee = builder.estimate_fee();
    let total_needed = amount + fee;

    if total_input < total_needed {
        return Err(TxBuilderError::InsufficientFunds);
    }

    let change = total_input - total_needed;

    // Add change output if significant (> dust limit ~546 sats)
    if change > 1000 {
        builder.add_output(change_address, change)?;
    }

    builder.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::Network;

    #[test]
    fn test_builder_creation() {
        let builder = TransactionBuilder::new(1);
        assert_eq!(builder.fee_rate, 1);
    }

    #[test]
    fn test_fee_estimation() {
        let mut builder = TransactionBuilder::new(1);

        // Add dummy input
        let utxo = Utxo {
            outpoint: super::super::utxos::OutPointData {
                txid: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
                vout: 0,
            },
            txout: super::super::utxos::TxOutData {
                value: 100000,
                script_pubkey: "0014abcd".to_string(),
            },
            height: Some(100),
            address: "test".to_string(),
            derivation_index: 0,
            is_change: false,
        };

        let pk = PrivateKey::from_wif("cNJFgo1driFnPcBdBX8BrJrpxchBWXwXCvNH5SoSkdcF6JXXwHMm")
            .unwrap();

        builder.add_input(utxo, pk).unwrap();
        builder.add_output("tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx", 50000).unwrap();

        let fee = builder.estimate_fee();
        assert!(fee > 0);
    }
}
