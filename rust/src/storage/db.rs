use rusqlite::Connection;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Lock error: {0}")]
    Lock(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: Option<i64>,
    pub payment_hash: String,
    pub payment_type: String, // "sent", "received", "onchain_sent", "onchain_received"
    pub amount_sats: i64,
    pub fee_sats: Option<i64>,
    pub status: String, // "pending", "completed", "failed"
    pub timestamp: i64,
    pub description: Option<String>,
    pub destination: Option<String>,
    pub txid: Option<String>,
    pub preimage: Option<String>,
    pub bolt11: Option<String>,
}

pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, DatabaseError> {
        let conn = Connection::open(path)?;

        let db = Database {
            conn: Arc::new(Mutex::new(conn)),
        };

        db.init_tables()?;

        Ok(db)
    }

    #[cfg(test)]
    pub fn in_memory() -> Result<Self, DatabaseError> {
        let conn = Connection::open_in_memory()?;

        let db = Database {
            conn: Arc::new(Mutex::new(conn)),
        };

        db.init_tables()?;

        Ok(db)
    }

    fn init_tables(&self) -> Result<(), DatabaseError> {
        let conn = self.conn.lock()
            .map_err(|e| DatabaseError::Lock(format!("{}", e)))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS payments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                payment_hash TEXT NOT NULL UNIQUE,
                payment_type TEXT NOT NULL,
                amount_sats INTEGER NOT NULL,
                fee_sats INTEGER,
                status TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                description TEXT,
                destination TEXT,
                txid TEXT,
                preimage TEXT,
                bolt11 TEXT
            )",
            [],
        )?;

        // Create indices
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_payments_timestamp ON payments(timestamp DESC)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_payments_type ON payments(payment_type)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_payments_status ON payments(status)",
            [],
        )?;

        Ok(())
    }

    /// Insert a new payment into the database
    pub fn insert_payment(&self, payment: &Payment) -> Result<i64, DatabaseError> {
        let conn = self.conn.lock()
            .map_err(|e| DatabaseError::Lock(format!("{}", e)))?;

        conn.execute(
            "INSERT INTO payments (
                payment_hash, payment_type, amount_sats, fee_sats, status,
                timestamp, description, destination, txid, preimage, bolt11
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                payment.payment_hash,
                payment.payment_type,
                payment.amount_sats,
                payment.fee_sats,
                payment.status,
                payment.timestamp,
                payment.description,
                payment.destination,
                payment.txid,
                payment.preimage,
                payment.bolt11,
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// Upsert a payment - insert if new, update if exists (based on payment_hash)
    /// This is used during sync to update payment status and details from the network
    pub fn upsert_payment(&self, payment: &Payment) -> Result<(), DatabaseError> {
        let conn = self.conn.lock()
            .map_err(|e| DatabaseError::Lock(format!("{}", e)))?;

        conn.execute(
            "INSERT INTO payments (
                payment_hash, payment_type, amount_sats, fee_sats, status,
                timestamp, description, destination, txid, preimage, bolt11
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            ON CONFLICT(payment_hash) DO UPDATE SET
                payment_type = excluded.payment_type,
                amount_sats = excluded.amount_sats,
                fee_sats = excluded.fee_sats,
                status = excluded.status,
                timestamp = CASE WHEN payments.timestamp = 0 THEN excluded.timestamp ELSE payments.timestamp END,
                description = excluded.description,
                destination = excluded.destination,
                txid = excluded.txid,
                preimage = excluded.preimage,
                bolt11 = excluded.bolt11",
            params![
                payment.payment_hash,
                payment.payment_type,
                payment.amount_sats,
                payment.fee_sats,
                payment.status,
                payment.timestamp,
                payment.description,
                payment.destination,
                payment.txid,
                payment.preimage,
                payment.bolt11,
            ],
        )?;

        Ok(())
    }

    /// Get count of payments in the database
    pub fn get_payment_count(&self) -> Result<i64, DatabaseError> {
        let conn = self.conn.lock()
            .map_err(|e| DatabaseError::Lock(format!("{}", e)))?;

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM payments",
            [],
            |row| row.get(0)
        )?;

        Ok(count)
    }

    #[cfg(test)]
    pub fn get_payment(&self, payment_hash: &str) -> Result<Payment, DatabaseError> {
        let conn = self.conn.lock()
            .map_err(|e| DatabaseError::Lock(format!("{}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT id, payment_hash, payment_type, amount_sats, fee_sats, status,
                    timestamp, description, destination, txid, preimage, bolt11
             FROM payments WHERE payment_hash = ?1"
        )?;

        let payment = stmt.query_row(params![payment_hash], |row| {
            Ok(Payment {
                id: row.get(0)?,
                payment_hash: row.get(1)?,
                payment_type: row.get(2)?,
                amount_sats: row.get(3)?,
                fee_sats: row.get(4)?,
                status: row.get(5)?,
                timestamp: row.get(6)?,
                description: row.get(7)?,
                destination: row.get(8)?,
                txid: row.get(9)?,
                preimage: row.get(10)?,
                bolt11: row.get(11)?,
            })
        })?;

        Ok(payment)
    }

    pub fn list_payments(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<Payment>, DatabaseError> {
        let conn = self.conn.lock()
            .map_err(|e| DatabaseError::Lock(format!("{}", e)))?;

        let mut query = String::from(
            "SELECT id, payment_hash, payment_type, amount_sats, fee_sats, status,
                    timestamp, description, destination, txid, preimage, bolt11
             FROM payments ORDER BY timestamp DESC"
        );

        if let Some(limit) = limit {
            query.push_str(&format!(" LIMIT {}", limit));
            if let Some(offset) = offset {
                query.push_str(&format!(" OFFSET {}", offset));
            }
        }

        let mut stmt = conn.prepare(&query)?;

        let payments = stmt.query_map([], |row| {
            Ok(Payment {
                id: row.get(0)?,
                payment_hash: row.get(1)?,
                payment_type: row.get(2)?,
                amount_sats: row.get(3)?,
                fee_sats: row.get(4)?,
                status: row.get(5)?,
                timestamp: row.get(6)?,
                description: row.get(7)?,
                destination: row.get(8)?,
                txid: row.get(9)?,
                preimage: row.get(10)?,
                bolt11: row.get(11)?,
            })
        })?;

        let mut result = Vec::new();
        for payment in payments {
            result.push(payment?);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creation() {
        let db = Database::in_memory().unwrap();
        assert_eq!(db.get_payment_count().unwrap(), 0);
    }

    #[test]
    fn test_insert_and_get_payment() {
        let db = Database::in_memory().unwrap();

        let payment = Payment {
            id: None,
            payment_hash: "test_hash_123".to_string(),
            payment_type: "sent".to_string(),
            amount_sats: 100000,
            fee_sats: Some(1000),
            status: "completed".to_string(),
            timestamp: 1234567890,
            description: Some("Test payment".to_string()),
            destination: Some("tb1test".to_string()),
            txid: None,
            preimage: None,
            bolt11: None,
        };

        let id = db.insert_payment(&payment).unwrap();
        assert!(id > 0);

        let retrieved = db.get_payment("test_hash_123").unwrap();
        assert_eq!(retrieved.amount_sats, 100000);
        assert_eq!(retrieved.payment_type, "sent");
    }

    #[test]
    fn test_list_payments() {
        let db = Database::in_memory().unwrap();

        for i in 0..5 {
            let payment = Payment {
                id: None,
                payment_hash: format!("hash_{}", i),
                payment_type: "sent".to_string(),
                amount_sats: 10000 * (i + 1),
                fee_sats: Some(100),
                status: "completed".to_string(),
                timestamp: 1234567890 + i,
                description: None,
                destination: None,
                txid: None,
                preimage: None,
                bolt11: None,
            };

            db.insert_payment(&payment).unwrap();
        }

        let payments = db.list_payments(Some(3), Some(0)).unwrap();
        assert_eq!(payments.len(), 3);

        // Should be ordered by timestamp descending
        assert!(payments[0].timestamp > payments[1].timestamp);
    }
}
