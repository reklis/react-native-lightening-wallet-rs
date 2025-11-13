use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Network, PrivateKey};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum KeyError {
    #[error("Invalid mnemonic: {0}")]
    InvalidMnemonic(String),
    #[error("Derivation error: {0}")]
    Derivation(String),
    #[error("Invalid network: {0}")]
    InvalidNetwork(String),
}

pub struct KeyManager {
    master_key: Xpriv,
    network: Network,
    secp: Secp256k1<bitcoin::secp256k1::All>,
}

impl KeyManager {
    /// Create a new KeyManager from a mnemonic phrase
    pub fn from_mnemonic(mnemonic: &str, network: Network) -> Result<Self, KeyError> {
        // Parse mnemonic
        let mnemonic = bip39::Mnemonic::from_str(mnemonic)
            .map_err(|e| KeyError::InvalidMnemonic(format!("{:?}", e)))?;

        // Generate seed
        let seed = mnemonic.to_seed("");

        // Create master key
        let secp = Secp256k1::new();
        let master_key = Xpriv::new_master(network, &seed)
            .map_err(|e| KeyError::Derivation(format!("Failed to create master key: {}", e)))?;

        Ok(KeyManager {
            master_key,
            network,
            secp,
        })
    }

    /// Generate a new random mnemonic
    pub fn generate_mnemonic() -> String {
        use rand::RngCore;
        let mut entropy = [0u8; 32]; // 32 bytes = 24 words
        rand::thread_rng().fill_bytes(&mut entropy);
        let mnemonic = bip39::Mnemonic::from_entropy(&entropy)
            .expect("Failed to generate mnemonic from entropy");
        mnemonic.to_string()
    }

    /// Get the network
    pub fn network(&self) -> Network {
        self.network
    }

    /// Derive a key at a specific path
    pub fn derive_key(&self, path: &str) -> Result<Xpriv, KeyError> {
        let path = DerivationPath::from_str(path)
            .map_err(|e| KeyError::Derivation(format!("Invalid path: {}", e)))?;

        self.master_key
            .derive_priv(&self.secp, &path)
            .map_err(|e| KeyError::Derivation(format!("Derivation failed: {}", e)))
    }

    /// Derive a public key at a specific path
    pub fn derive_pubkey(&self, path: &str) -> Result<Xpub, KeyError> {
        let derived_key = self.derive_key(path)?;
        Ok(Xpub::from_priv(&self.secp, &derived_key))
    }

    /// Derive a Bitcoin address at a specific path (Native SegWit)
    pub fn derive_address(&self, path: &str) -> Result<bitcoin::Address, KeyError> {
        let pubkey = self.derive_pubkey(path)?;
        let address = bitcoin::Address::p2wpkh(&pubkey.to_pub(), self.network)
            .map_err(|e| KeyError::Derivation(format!("Address derivation failed: {}", e)))?;
        Ok(address)
    }

    /// Get a private key for signing at a specific path
    pub fn get_private_key(&self, path: &str) -> Result<PrivateKey, KeyError> {
        let derived_key = self.derive_key(path)?;
        Ok(PrivateKey {
            compressed: true,
            network: self.network,
            inner: derived_key.private_key,
        })
    }

    /// Get the Lightning seed bytes for LDK
    /// LDK uses a 32-byte seed derived from the master key
    pub fn get_lightning_seed(&self) -> Result<[u8; 32], KeyError> {
        // Derive the Lightning node key at m/535h/0h
        let path = "m/535'/0'";
        let derived = self.derive_key(path)?;
        Ok(derived.private_key.secret_bytes())
    }

    /// Get the Bitcoin wallet extended public key (for address generation)
    /// Using BIP84 path: m/84'/network'/0'
    pub fn get_wallet_xpub(&self) -> Result<Xpub, KeyError> {
        let coin_type = match self.network {
            Network::Bitcoin => 0,
            Network::Testnet => 1,
            _ => return Err(KeyError::InvalidNetwork(format!("{:?}", self.network))),
        };

        let path = format!("m/84'/{}'/{}'", coin_type, 0);
        self.derive_pubkey(&path)
    }

    /// Derive receiving address at index (BIP84 path)
    pub fn get_receiving_address(&self, index: u32) -> Result<bitcoin::Address, KeyError> {
        let coin_type = match self.network {
            Network::Bitcoin => 0,
            Network::Testnet => 1,
            _ => return Err(KeyError::InvalidNetwork(format!("{:?}", self.network))),
        };

        let path = format!("m/84'/{}'/{}'/{}/{}", coin_type, 0, 0, index);
        self.derive_address(&path)
    }

    /// Derive change address at index (BIP84 path)
    pub fn get_change_address(&self, index: u32) -> Result<bitcoin::Address, KeyError> {
        let coin_type = match self.network {
            Network::Bitcoin => 0,
            Network::Testnet => 1,
            _ => return Err(KeyError::InvalidNetwork(format!("{:?}", self.network))),
        };

        let path = format!("m/84'/{}'/{}'/{}/{}", coin_type, 0, 1, index);
        self.derive_address(&path)
    }

    /// Get private key for receiving address at index
    pub fn get_receiving_private_key(&self, index: u32) -> Result<PrivateKey, KeyError> {
        let coin_type = match self.network {
            Network::Bitcoin => 0,
            Network::Testnet => 1,
            _ => return Err(KeyError::InvalidNetwork(format!("{:?}", self.network))),
        };

        let path = format!("m/84'/{}'/{}'/{}/{}", coin_type, 0, 0, index);
        self.get_private_key(&path)
    }

    /// Get private key for change address at index
    pub fn get_change_private_key(&self, index: u32) -> Result<PrivateKey, KeyError> {
        let coin_type = match self.network {
            Network::Bitcoin => 0,
            Network::Testnet => 1,
            _ => return Err(KeyError::InvalidNetwork(format!("{:?}", self.network))),
        };

        let path = format!("m/84'/{}'/{}'/{}/{}", coin_type, 0, 1, index);
        self.get_private_key(&path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_mnemonic() {
        let mnemonic = KeyManager::generate_mnemonic();
        assert!(!mnemonic.is_empty());
        let words: Vec<&str> = mnemonic.split_whitespace().collect();
        assert_eq!(words.len(), 24);
    }

    #[test]
    fn test_from_mnemonic() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
        let manager = KeyManager::from_mnemonic(mnemonic, Network::Testnet).unwrap();
        assert_eq!(manager.network(), Network::Testnet);
    }

    #[test]
    fn test_derive_address() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
        let manager = KeyManager::from_mnemonic(mnemonic, Network::Testnet).unwrap();

        let address = manager.get_receiving_address(0).unwrap();
        assert!(!address.to_string().is_empty());
    }

    #[test]
    fn test_lightning_seed() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
        let manager = KeyManager::from_mnemonic(mnemonic, Network::Testnet).unwrap();

        let seed = manager.get_lightning_seed().unwrap();
        assert_eq!(seed.len(), 32);
    }
}
