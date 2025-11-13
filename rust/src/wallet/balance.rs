use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BalanceError {
    #[error("Calculation error: {0}")]
    Calculation(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WalletBalance {
    /// Total confirmed on-chain balance in satoshis
    pub onchain_confirmed: u64,

    /// Total unconfirmed on-chain balance in satoshis
    pub onchain_unconfirmed: u64,

    /// Total Lightning balance (sum of all channel balances) in satoshis
    pub lightning_balance: u64,

    /// Lightning balance that can be received (inbound liquidity) in satoshis
    pub lightning_receivable: u64,

    /// Claimable balance from closed channels in satoshis
    pub claimable_balance: u64,

    /// Total balance (onchain_confirmed + lightning_balance)
    pub total: u64,

    /// Pending balance (onchain_unconfirmed + claimable_balance)
    pub pending: u64,
}

impl WalletBalance {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_components(
        onchain_confirmed: u64,
        onchain_unconfirmed: u64,
        lightning_balance: u64,
        lightning_receivable: u64,
        claimable_balance: u64,
    ) -> Self {
        let total = onchain_confirmed + lightning_balance;
        let pending = onchain_unconfirmed + claimable_balance;

        WalletBalance {
            onchain_confirmed,
            onchain_unconfirmed,
            lightning_balance,
            lightning_receivable,
            claimable_balance,
            total,
            pending,
        }
    }

    pub fn update_onchain(&mut self, confirmed: u64, unconfirmed: u64) {
        self.onchain_confirmed = confirmed;
        self.onchain_unconfirmed = unconfirmed;
        self.recalculate();
    }

    pub fn update_lightning(&mut self, balance: u64, receivable: u64) {
        self.lightning_balance = balance;
        self.lightning_receivable = receivable;
        self.recalculate();
    }

    pub fn update_claimable(&mut self, claimable: u64) {
        self.claimable_balance = claimable;
        self.recalculate();
    }

    fn recalculate(&mut self) {
        self.total = self.onchain_confirmed + self.lightning_balance;
        self.pending = self.onchain_unconfirmed + self.claimable_balance;
    }

    /// Returns total spendable balance (confirmed onchain + lightning)
    pub fn spendable(&self) -> u64 {
        self.onchain_confirmed + self.lightning_balance
    }

    /// Returns total balance including pending
    pub fn total_including_pending(&self) -> u64 {
        self.total + self.pending
    }
}

pub struct BalanceManager {
    current_balance: std::sync::Arc<std::sync::Mutex<WalletBalance>>,
}

impl BalanceManager {
    pub fn new() -> Self {
        BalanceManager {
            current_balance: std::sync::Arc::new(std::sync::Mutex::new(WalletBalance::new())),
        }
    }

    pub fn get_balance(&self) -> Result<WalletBalance, BalanceError> {
        let balance = self.current_balance.lock()
            .map_err(|e| BalanceError::Calculation(format!("Lock error: {}", e)))?;

        Ok(balance.clone())
    }

    pub fn update_balance(&self, balance: WalletBalance) -> Result<(), BalanceError> {
        let mut current = self.current_balance.lock()
            .map_err(|e| BalanceError::Calculation(format!("Lock error: {}", e)))?;

        *current = balance;
        Ok(())
    }

    pub fn update_onchain(&self, confirmed: u64, unconfirmed: u64) -> Result<(), BalanceError> {
        let mut balance = self.current_balance.lock()
            .map_err(|e| BalanceError::Calculation(format!("Lock error: {}", e)))?;

        balance.update_onchain(confirmed, unconfirmed);
        Ok(())
    }

    pub fn update_lightning(&self, lightning: u64, receivable: u64) -> Result<(), BalanceError> {
        let mut balance = self.current_balance.lock()
            .map_err(|e| BalanceError::Calculation(format!("Lock error: {}", e)))?;

        balance.update_lightning(lightning, receivable);
        Ok(())
    }

    pub fn update_claimable(&self, claimable: u64) -> Result<(), BalanceError> {
        let mut balance = self.current_balance.lock()
            .map_err(|e| BalanceError::Calculation(format!("Lock error: {}", e)))?;

        balance.update_claimable(claimable);
        Ok(())
    }
}

impl Default for BalanceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balance_creation() {
        let balance = WalletBalance::from_components(100000, 50000, 200000, 100000, 10000);

        assert_eq!(balance.onchain_confirmed, 100000);
        assert_eq!(balance.lightning_balance, 200000);
        assert_eq!(balance.total, 300000);
        assert_eq!(balance.pending, 60000);
    }

    #[test]
    fn test_balance_update() {
        let mut balance = WalletBalance::new();

        balance.update_onchain(100000, 50000);
        assert_eq!(balance.onchain_confirmed, 100000);
        assert_eq!(balance.onchain_unconfirmed, 50000);

        balance.update_lightning(200000, 100000);
        assert_eq!(balance.lightning_balance, 200000);
        assert_eq!(balance.total, 300000);
    }

    #[test]
    fn test_balance_manager() {
        let manager = BalanceManager::new();

        manager.update_onchain(100000, 50000).unwrap();
        manager.update_lightning(200000, 100000).unwrap();

        let balance = manager.get_balance().unwrap();
        assert_eq!(balance.total, 300000);
        assert_eq!(balance.spendable(), 300000);
    }
}
