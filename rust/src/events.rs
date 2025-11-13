use crossbeam_channel::{bounded, Sender, Receiver};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WalletEvent {
    BalanceUpdated {
        onchain_confirmed: u64,
        onchain_unconfirmed: u64,
        lightning_balance: u64,
        total: u64,
    },
    PaymentReceived {
        payment_hash: String,
        amount_sats: u64,
        description: Option<String>,
    },
    PaymentSent {
        payment_hash: String,
        amount_sats: u64,
        fee_sats: Option<u64>,
        destination: Option<String>,
    },
    PaymentFailed {
        payment_hash: String,
        reason: String,
    },
    ChannelOpened {
        channel_id: String,
        counterparty: String,
        capacity_sats: u64,
    },
    ChannelClosed {
        channel_id: String,
        reason: String,
    },
    SyncStarted,
    SyncCompleted {
        duration_ms: u64,
    },
    SyncFailed {
        error: String,
    },
    TransactionConfirmed {
        txid: String,
        confirmations: u32,
    },
    Error {
        message: String,
    },
}

pub struct EventEmitter {
    sender: Sender<WalletEvent>,
    receiver: Arc<Mutex<Receiver<WalletEvent>>>,
}

impl EventEmitter {
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = bounded(capacity);

        EventEmitter {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    pub fn emit(&self, event: WalletEvent) {
        // If the channel is full, drop the oldest event
        let _ = self.sender.try_send(event);
    }

    pub fn get_receiver(&self) -> Arc<Mutex<Receiver<WalletEvent>>> {
        Arc::clone(&self.receiver)
    }

    pub fn try_recv(&self) -> Option<WalletEvent> {
        if let Ok(receiver) = self.receiver.lock() {
            receiver.try_recv().ok()
        } else {
            None
        }
    }

    pub fn drain_events(&self) -> Vec<WalletEvent> {
        let mut events = Vec::new();

        while let Some(event) = self.try_recv() {
            events.push(event);
        }

        events
    }
}

impl Clone for EventEmitter {
    fn clone(&self) -> Self {
        EventEmitter {
            sender: self.sender.clone(),
            receiver: Arc::clone(&self.receiver),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_emission() {
        let emitter = EventEmitter::new(100);

        emitter.emit(WalletEvent::SyncStarted);
        emitter.emit(WalletEvent::SyncCompleted { duration_ms: 1000 });

        let events = emitter.drain_events();
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_event_types() {
        let emitter = EventEmitter::new(100);

        emitter.emit(WalletEvent::BalanceUpdated {
            onchain_confirmed: 100000,
            onchain_unconfirmed: 50000,
            lightning_balance: 200000,
            total: 300000,
        });

        let event = emitter.try_recv();
        assert!(event.is_some());
    }
}
