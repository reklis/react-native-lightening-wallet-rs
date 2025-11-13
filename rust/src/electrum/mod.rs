use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{channel, Sender};

use rustls::{ClientConfig, ClientConnection, StreamOwned};
use serde_json::{json, Value};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ElectrumError {
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Protocol error: {0}")]
    Protocol(String),
    #[error("Timeout error: {0}")]
    Timeout(String),
    #[error("Not connected")]
    NotConnected,
    #[error("Already connected")]
    AlreadyConnected,
    #[error("Serialization error: {0}")]
    Serialization(String),
}

#[derive(Debug, Clone)]
pub struct Peer {
    pub host: String,
    pub ssl: Option<u16>,
    pub tcp: Option<u16>,
    pub protocol: Option<String>,
}

pub struct ElectrumClient {
    stream: Arc<Mutex<BufReader<StreamOwned<ClientConnection, TcpStream>>>>,
    pending_requests: Arc<Mutex<HashMap<u64, Sender<Result<Value, String>>>>>,
}

static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

impl ElectrumClient {
    pub fn connect(host: &str, port: u16) -> Result<Self, ElectrumError> {
        // Create TLS config with webpki root certificates
        let mut root_store = rustls::RootCertStore::empty();
        root_store.add_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
            rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject,
                ta.spki,
                ta.name_constraints,
            )
        }));

        let config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let server_name = host.try_into()
            .map_err(|e| ElectrumError::Connection(format!("Invalid DNS name: {:?}", e)))?;

        let conn = ClientConnection::new(Arc::new(config), server_name)
            .map_err(|e| ElectrumError::Connection(format!("TLS setup: {}", e)))?;

        let tcp_stream = TcpStream::connect((host, port))
            .map_err(|e| ElectrumError::Connection(format!("TCP connect: {}", e)))?;

        tcp_stream
            .set_read_timeout(Some(Duration::from_secs(30)))
            .map_err(|e| ElectrumError::Connection(format!("Set timeout: {}", e)))?;

        tcp_stream
            .set_write_timeout(Some(Duration::from_secs(30)))
            .map_err(|e| ElectrumError::Connection(format!("Set timeout: {}", e)))?;

        let tls_stream = StreamOwned::new(conn, tcp_stream);
        let stream = BufReader::new(tls_stream);

        let client = ElectrumClient {
            stream: Arc::new(Mutex::new(stream)),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
        };

        // Verify connection with server.version
        client.send_request("server.version", json!(["reactnative-lightening-wallet", "1.4"]))
            .map_err(|e| ElectrumError::Connection(format!("Version handshake failed: {}", e)))?;

        Ok(client)
    }

    pub fn send_request(&self, method: &str, params: Value) -> Result<Value, String> {
        let request_id = REQUEST_ID.fetch_add(1, Ordering::SeqCst);

        let (tx, rx) = channel();

        {
            let mut pending = self.pending_requests.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            pending.insert(request_id, tx);
        }

        {
            let mut stream = self.stream.lock()
                .map_err(|e| format!("Lock error: {}", e))?;

            let request = json!({
                "jsonrpc": "2.0",
                "method": method,
                "params": params,
                "id": request_id
            });

            let request_str = serde_json::to_string(&request)
                .map_err(|e| format!("Serialization error: {}", e))?;

            stream
                .get_mut()
                .write_all(format!("{}\n", request_str).as_bytes())
                .map_err(|e| format!("Write error: {}", e))?;

            stream
                .get_mut()
                .flush()
                .map_err(|e| format!("Flush error: {}", e))?;
        }

        // Try to read and dispatch responses
        for _ in 0..10 {
            match self.try_read_response() {
                Ok(_) => {
                    if let Ok(result) = rx.try_recv() {
                        let mut pending = self.pending_requests.lock().unwrap();
                        pending.remove(&request_id);
                        return result;
                    }
                }
                Err(_) => {
                    std::thread::sleep(Duration::from_millis(50));
                }
            }
        }

        let mut pending = self.pending_requests.lock().unwrap();
        pending.remove(&request_id);
        Err(format!("Timeout waiting for response to request {}", request_id))
    }

    fn try_read_response(&self) -> Result<(), String> {
        let mut stream = self.stream.lock()
            .map_err(|e| format!("Lock error: {}", e))?;

        let mut response_str = String::new();
        match stream.read_line(&mut response_str) {
            Ok(0) => return Err("Connection closed".to_string()),
            Ok(_) => {},
            Err(e) => return Err(format!("Read error: {}", e)),
        }

        let response_str = response_str.trim();
        if response_str.is_empty() {
            return Err("Empty response".to_string());
        }

        let response: Value = serde_json::from_str(response_str)
            .map_err(|e| format!("Invalid JSON: {}", e))?;

        let response_id = response.get("id")
            .and_then(|id| id.as_u64())
            .ok_or_else(|| "No ID in response".to_string())?;

        let sender = {
            let mut pending = self.pending_requests.lock().unwrap();
            pending.remove(&response_id)
        };

        if let Some(sender) = sender {
            if let Some(error) = response.get("error") {
                if !error.is_null() {
                    let _ = sender.send(Err(format!("Server error: {}", error)));
                    return Ok(());
                }
            }

            let result = response
                .get("result")
                .cloned()
                .ok_or_else(|| "No result field".to_string());

            let _ = sender.send(result);
        }

        Ok(())
    }

    // Public API methods
    pub fn get_header(&self, height: u32) -> Result<Value, ElectrumError> {
        self.send_request("blockchain.block.header", json!([height]))
            .map_err(|e| ElectrumError::Protocol(e))
    }

    pub fn get_balance(&self, script_hash: &str) -> Result<Value, ElectrumError> {
        self.send_request("blockchain.scripthash.get_balance", json!([script_hash]))
            .map_err(|e| ElectrumError::Protocol(e))
    }

    pub fn get_history(&self, script_hash: &str) -> Result<Value, ElectrumError> {
        self.send_request("blockchain.scripthash.get_history", json!([script_hash]))
            .map_err(|e| ElectrumError::Protocol(e))
    }

    pub fn get_transaction(&self, tx_hash: &str, verbose: bool) -> Result<Value, ElectrumError> {
        self.send_request("blockchain.transaction.get", json!([tx_hash, verbose]))
            .map_err(|e| ElectrumError::Protocol(e))
    }

    pub fn get_merkle(&self, tx_hash: &str, height: u32) -> Result<Value, ElectrumError> {
        self.send_request("blockchain.transaction.get_merkle", json!([tx_hash, height]))
            .map_err(|e| ElectrumError::Protocol(e))
    }

    pub fn broadcast_transaction(&self, raw_tx: &str) -> Result<Value, ElectrumError> {
        self.send_request("blockchain.transaction.broadcast", json!([raw_tx]))
            .map_err(|e| ElectrumError::Protocol(e))
    }

    pub fn subscribe_headers(&self) -> Result<Value, ElectrumError> {
        self.send_request("blockchain.headers.subscribe", json!([]))
            .map_err(|e| ElectrumError::Protocol(e))
    }

    pub fn ping(&self) -> Result<Value, ElectrumError> {
        self.send_request("server.ping", json!([]))
            .map_err(|e| ElectrumError::Protocol(e))
    }
}

pub fn get_default_peers(network: &str) -> Vec<Peer> {
    match network {
        "bitcoin" => vec![
            Peer {
                host: "electrum.blockstream.info".to_string(),
                ssl: Some(50002),
                tcp: Some(50001),
                protocol: Some("ssl".to_string()),
            },
            Peer {
                host: "blockstream.info".to_string(),
                ssl: Some(700),
                tcp: Some(110),
                protocol: Some("ssl".to_string()),
            },
        ],
        "testnet" | "bitcoinTestnet" => vec![
            Peer {
                host: "testnet.qtornado.com".to_string(),
                ssl: Some(51002),
                tcp: Some(51001),
                protocol: Some("ssl".to_string()),
            },
            Peer {
                host: "testnet.aranguren.org".to_string(),
                ssl: Some(51002),
                tcp: Some(51001),
                protocol: Some("ssl".to_string()),
            },
            Peer {
                host: "electrum.blockstream.info".to_string(),
                ssl: Some(60002),
                tcp: Some(60001),
                protocol: Some("ssl".to_string()),
            },
        ],
        _ => vec![],
    }
}
