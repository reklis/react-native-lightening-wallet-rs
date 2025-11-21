mod wallet;
mod storage;
mod events;
mod coordinator;
mod ldk;

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use bitcoin::Network;
use serde::{Deserialize, Serialize};
use serde_json::json;

use coordinator::WalletCoordinator;

#[cfg(target_os = "android")]
use jni::JNIEnv;
#[cfg(target_os = "android")]
use jni::objects::{JClass, JString};
#[cfg(target_os = "android")]
use jni::sys::jstring;

// Global wallet instances
lazy_static::lazy_static! {
    static ref WALLETS: Arc<Mutex<HashMap<String, Arc<WalletCoordinator>>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    success: bool,
    data: Option<serde_json::Value>,
    error: Option<String>,
}

impl Response {
    fn success(data: serde_json::Value) -> String {
        let response = Response {
            success: true,
            data: Some(data),
            error: None,
        };
        serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string())
    }

    fn error(error: String) -> String {
        let response = Response {
            success: false,
            data: None,
            error: Some(error),
        };
        serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string())
    }
}

// Initialize wallet
fn initialize_wallet_impl(user_id: String, mnemonic: String, network: String, db_path: String) -> String {
    // Check if wallet is already initialized to prevent duplicate instances
    {
        let wallets = WALLETS.lock().unwrap();
        if wallets.contains_key(&user_id) {
            eprintln!("[INIT] Wallet already initialized for user {}, returning existing instance", user_id);
            return Response::success(json!({ "userId": user_id, "initialized": true, "existing": true }));
        }
    }

    let network = match network.as_str() {
        "bitcoin" => Network::Bitcoin,
        "testnet" | "bitcoinTestnet" => Network::Testnet,
        "regtest" => Network::Regtest,
        _ => return Response::error("Invalid network".to_string()),
    };

    eprintln!("[INIT] Creating new wallet for user {}", user_id);
    match WalletCoordinator::new(&db_path) {
        Ok(coordinator) => {
            match coordinator.initialize(&mnemonic, network) {
                Ok(_) => {
                    let mut wallets = WALLETS.lock().unwrap();
                    // Double-check in case another thread initialized while we were creating
                    if wallets.contains_key(&user_id) {
                        eprintln!("[INIT] Wallet was initialized by another thread, using that instance");
                        return Response::success(json!({ "userId": user_id, "initialized": true, "existing": true }));
                    }
                    wallets.insert(user_id.clone(), Arc::new(coordinator));
                    eprintln!("[INIT] Wallet initialized successfully for user {}", user_id);
                    Response::success(json!({ "userId": user_id, "initialized": true }))
                }
                Err(e) => Response::error(format!("Initialization failed: {}", e)),
            }
        }
        Err(e) => Response::error(format!("Failed to create coordinator: {}", e)),
    }
}

// Generate new mnemonic
fn generate_mnemonic_impl() -> String {
    let mnemonic = wallet::KeyManager::generate_mnemonic();
    Response::success(json!({ "mnemonic": mnemonic }))
}

// Get balance (using LDK-node's balance API)
fn get_balance_impl(user_id: String) -> String {
    let wallets = WALLETS.lock().unwrap();

    if let Some(coordinator) = wallets.get(&user_id) {
        let lightning_node = coordinator.get_lightning_node();

        // Get all balances in a single call to list_balances()
        match lightning_node.get_all_balances() {
            Ok((total_onchain, spendable_onchain, pending_broadcast, broadcast_awaiting, awaiting_threshold, lightning_balance)) => {
                // Calculate totals
                let pending_sweep_total = pending_broadcast + broadcast_awaiting + awaiting_threshold;
                let total_with_pending = total_onchain + pending_sweep_total;

                let balances = json!({
                    "onchain_confirmed": spendable_onchain,
                    "onchain_unconfirmed": total_onchain.saturating_sub(spendable_onchain),
                    "pending_sweep_balance": pending_sweep_total,
                    "pending_sweep_pending_broadcast": pending_broadcast,
                    "pending_sweep_broadcast_awaiting_confirmation": broadcast_awaiting,
                    "pending_sweep_awaiting_threshold_confirmations": awaiting_threshold,
                    "lightning_balance": lightning_balance,
                    "total": total_with_pending + lightning_balance
                });
                Response::success(balances)
            }
            Err(e) => Response::error(format!("Failed to get balance: {}", e)),
        }
    } else {
        Response::error("Wallet not initialized".to_string())
    }
}

// Sync wallet
fn sync_wallet_impl(user_id: String) -> String {
    let wallets = WALLETS.lock().unwrap();

    if let Some(coordinator) = wallets.get(&user_id) {
        // Sync blockchain data (payment sync happens automatically in background)
        match coordinator.sync() {
            Ok(_) => Response::success(json!({ "synced": true })),
            Err(e) => Response::error(format!("Sync failed: {}", e)),
        }
    } else {
        Response::error("Wallet not initialized".to_string())
    }
}

// Get receiving address (using LDK-node's onchain wallet)
fn get_receiving_address_impl(user_id: String) -> String {
    let wallets = WALLETS.lock().unwrap();

    if let Some(coordinator) = wallets.get(&user_id) {
        let lightning_node = coordinator.get_lightning_node();
        match lightning_node.get_onchain_address() {
            Ok(address) => Response::success(json!({ "address": address })),
            Err(e) => Response::error(format!("Failed to get address: {}", e)),
        }
    } else {
        Response::error("Wallet not initialized".to_string())
    }
}

// List payments
fn list_payments_impl(user_id: String, limit: Option<usize>, offset: Option<usize>) -> String {
    let wallets = WALLETS.lock().unwrap();

    if let Some(coordinator) = wallets.get(&user_id) {
        let db = coordinator.get_database();
        match db.list_payments(limit, offset) {
            Ok(payments) => Response::success(serde_json::to_value(payments).unwrap()),
            Err(e) => Response::error(format!("Failed to list payments: {}", e)),
        }
    } else {
        Response::error("Wallet not initialized".to_string())
    }
}

// Get events
fn get_events_impl(user_id: String) -> String {
    let wallets = WALLETS.lock().unwrap();

    if let Some(coordinator) = wallets.get(&user_id) {
        let emitter = coordinator.get_event_emitter();
        let events = emitter.drain_events();
        Response::success(serde_json::to_value(events).unwrap())
    } else {
        Response::error("Wallet not initialized".to_string())
    }
}

// Disconnect wallet
fn disconnect_wallet_impl(user_id: String) -> String {
    let mut wallets = WALLETS.lock().unwrap();

    if let Some(coordinator) = wallets.get(&user_id) {
        match coordinator.disconnect() {
            Ok(_) => {
                wallets.remove(&user_id);
                Response::success(json!({ "disconnected": true }))
            }
            Err(e) => Response::error(format!("Failed to disconnect: {}", e)),
        }
    } else {
        Response::error("Wallet not initialized".to_string())
    }
}

// Create Lightning invoice
fn create_invoice_impl(user_id: String, amount_sats: Option<u64>, description: Option<String>, expiry_secs: u32) -> String {
    let wallets = WALLETS.lock().unwrap();

    if let Some(coordinator) = wallets.get(&user_id) {
        let lightning_node = coordinator.get_lightning_node();
        let params = ldk::CreateInvoiceParams {
            amount_sats,
            description,
            expiry_secs,
        };
        match lightning_node.create_invoice(params) {
            Ok(invoice) => Response::success(serde_json::to_value(invoice).unwrap()),
            Err(e) => Response::error(format!("Failed to create invoice: {}", e)),
        }
    } else {
        Response::error("Wallet not initialized".to_string())
    }
}

// Decode Lightning invoice
fn decode_invoice_impl(bolt11: String) -> String {
    match ldk::LightningNode::decode_invoice(&bolt11) {
        Ok(invoice_info) => Response::success(serde_json::to_value(invoice_info).unwrap()),
        Err(e) => Response::error(format!("Failed to decode invoice: {}", e)),
    }
}

// Pay Lightning invoice
fn pay_invoice_impl(user_id: String, bolt11: String, amount_sats: Option<u64>) -> String {
    let wallets = WALLETS.lock().unwrap();

    if let Some(coordinator) = wallets.get(&user_id) {
        let lightning_node = coordinator.get_lightning_node();
        let params = ldk::PayInvoiceParams {
            bolt11,
            amount_sats,
        };
        match lightning_node.pay_invoice(params) {
            Ok(payment) => Response::success(serde_json::to_value(payment).unwrap()),
            Err(e) => Response::error(format!("Failed to pay invoice: {}", e)),
        }
    } else {
        Response::error("Wallet not initialized".to_string())
    }
}

// Send keysend payment (for Podcasting 2.0)
fn send_keysend_impl(user_id: String, destination_pubkey: String, amount_sats: u64, custom_records_json: Option<String>) -> String {
    let wallets = WALLETS.lock().unwrap();

    if let Some(coordinator) = wallets.get(&user_id) {
        let lightning_node = coordinator.get_lightning_node();

        // Parse custom records from JSON
        let custom_records = if let Some(json) = custom_records_json {
            match serde_json::from_str::<std::collections::HashMap<u64, Vec<u8>>>(&json) {
                Ok(records) => records,
                Err(e) => return Response::error(format!("Invalid custom records JSON: {}", e)),
            }
        } else {
            std::collections::HashMap::new()
        };

        let params = ldk::KeysendParams {
            destination_pubkey,
            amount_sats,
            custom_records,
        };

        match lightning_node.send_keysend(params) {
            Ok(payment) => Response::success(serde_json::to_value(payment).unwrap()),
            Err(e) => Response::error(format!("Failed to send keysend: {}", e)),
        }
    } else {
        Response::error("Wallet not initialized".to_string())
    }
}

// Estimate on-chain transaction fee
fn estimate_onchain_fee_impl(user_id: String, address: String, amount_sats: u64) -> String {
    let wallets = WALLETS.lock().unwrap();

    if let Some(_coordinator) = wallets.get(&user_id) {
        // Estimate transaction size
        // A typical P2WPKH transaction has:
        // - 1-2 inputs: ~68 vbytes each
        // - 2 outputs (recipient + change): ~31 vbytes each
        // - Base overhead: ~10.5 vbytes
        // Conservative estimate: assume 2 inputs
        let estimated_vbytes: f64 = 10.5 + (2.0 * 68.0) + (2.0 * 31.0); // ~208.5 vbytes

        // Use 1 sat/vbyte for regtest, could make this configurable
        let fee_rate: f64 = 1.0;
        let estimated_fee = (estimated_vbytes * fee_rate).ceil() as u64;

        Response::success(json!({
            "address": address,
            "amount_sats": amount_sats,
            "estimated_fee_sats": estimated_fee,
            "fee_rate_sat_per_vbyte": fee_rate,
            "estimated_vbytes": estimated_vbytes as u64
        }))
    } else {
        Response::error("Wallet not initialized".to_string())
    }
}

// Send on-chain Bitcoin transaction
fn send_onchain_impl(user_id: String, address: String, amount_sats: u64) -> String {
    let wallets = WALLETS.lock().unwrap();

    if let Some(coordinator) = wallets.get(&user_id) {
        let lightning_node = coordinator.get_lightning_node();
        let params = ldk::SendOnChainParams {
            address,
            amount_sats,
            fee_rate_sat_per_vbyte: 1, // Default to 1 sat/vbyte for regtest
        };
        match lightning_node.send_onchain(params) {
            Ok(txid) => Response::success(json!({ "txid": txid })),
            Err(e) => Response::error(format!("Failed to send on-chain: {}", e)),
        }
    } else {
        Response::error("Wallet not initialized".to_string())
    }
}

// Open Lightning channel
fn open_channel_impl(user_id: String, counterparty_node_id: String, channel_value_satoshis: u64, push_msat: u64, peer_address: Option<String>, peer_port: Option<u16>) -> String {
    let wallets = WALLETS.lock().unwrap();

    if let Some(coordinator) = wallets.get(&user_id) {
        let lightning_node = coordinator.get_lightning_node();
        let params = ldk::OpenChannelParams {
            counterparty_node_id,
            channel_value_satoshis,
            push_msat,
            is_public: false, // Default to private channels
            peer_address,
            peer_port,
        };
        match lightning_node.open_channel(params) {
            Ok(channel_id) => Response::success(json!({ "channel_id": channel_id })),
            Err(e) => Response::error(format!("Failed to open channel: {}", e)),
        }
    } else {
        Response::error("Wallet not initialized".to_string())
    }
}

// Close Lightning channel
fn close_channel_impl(user_id: String, channel_id: String, force: bool) -> String {
    let wallets = WALLETS.lock().unwrap();

    if let Some(coordinator) = wallets.get(&user_id) {
        let lightning_node = coordinator.get_lightning_node();
        let params = ldk::CloseChannelParams {
            channel_id,
            force,
            peer_address: None,
            peer_port: None,
        };
        match lightning_node.close_channel(params) {
            Ok(_) => Response::success(json!({ "closed": true })),
            Err(e) => Response::error(format!("Failed to close channel: {}", e)),
        }
    } else {
        Response::error("Wallet not initialized".to_string())
    }
}

// List Lightning channels
fn list_channels_impl(user_id: String) -> String {
    let wallets = WALLETS.lock().unwrap();

    if let Some(coordinator) = wallets.get(&user_id) {
        let lightning_node = coordinator.get_lightning_node();
        match lightning_node.list_channels() {
            Ok(channels) => Response::success(serde_json::to_value(channels).unwrap()),
            Err(e) => Response::error(format!("Failed to list channels: {}", e)),
        }
    } else {
        Response::error("Wallet not initialized".to_string())
    }
}

// Connect to peer
fn connect_peer_impl(user_id: String, node_id: String, address: String, port: u16) -> String {
    let wallets = WALLETS.lock().unwrap();

    if let Some(coordinator) = wallets.get(&user_id) {
        let lightning_node = coordinator.get_lightning_node();
        let params = ldk::ConnectPeerParams {
            node_id,
            address,
            port,
        };
        match lightning_node.connect_peer(params) {
            Ok(_) => Response::success(json!({ "connected": true })),
            Err(e) => Response::error(format!("Failed to connect peer: {}", e)),
        }
    } else {
        Response::error("Wallet not initialized".to_string())
    }
}

// Android JNI bindings
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_com_reactnativelighteningwallet_LighteningWalletModule_nativeInitialize(
    mut env: JNIEnv,
    _: JClass,
    user_id: JString,
    mnemonic: JString,
    network: JString,
    db_path: JString,
) -> jstring {
    let user_id: String = env.get_string(&user_id).unwrap().into();
    let mnemonic: String = env.get_string(&mnemonic).unwrap().into();
    let network: String = env.get_string(&network).unwrap().into();
    let db_path: String = env.get_string(&db_path).unwrap().into();

    let result = initialize_wallet_impl(user_id, mnemonic, network, db_path);
    env.new_string(result).unwrap().into_raw()
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_com_reactnativelighteningwallet_LighteningWalletModule_nativeGenerateMnemonic(
    env: JNIEnv,
    _: JClass,
) -> jstring {
    let result = generate_mnemonic_impl();
    env.new_string(result).unwrap().into_raw()
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_com_reactnativelighteningwallet_LighteningWalletModule_nativeGetBalance(
    mut env: JNIEnv,
    _: JClass,
    user_id: JString,
) -> jstring {
    let user_id: String = env.get_string(&user_id).unwrap().into();
    let result = get_balance_impl(user_id);
    env.new_string(result).unwrap().into_raw()
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_com_reactnativelighteningwallet_LighteningWalletModule_nativeSyncWallet(
    mut env: JNIEnv,
    _: JClass,
    user_id: JString,
) -> jstring {
    let user_id: String = env.get_string(&user_id).unwrap().into();
    let result = sync_wallet_impl(user_id);
    env.new_string(result).unwrap().into_raw()
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_com_reactnativelighteningwallet_LighteningWalletModule_nativeGetReceivingAddress(
    mut env: JNIEnv,
    _: JClass,
    user_id: JString,
) -> jstring {
    let user_id: String = env.get_string(&user_id).unwrap().into();
    let result = get_receiving_address_impl(user_id);
    env.new_string(result).unwrap().into_raw()
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_com_reactnativelighteningwallet_LighteningWalletModule_nativeListPayments(
    mut env: JNIEnv,
    _: JClass,
    user_id: JString,
    limit: i32,
    offset: i32,
) -> jstring {
    let user_id: String = env.get_string(&user_id).unwrap().into();

    let limit_opt = if limit > 0 { Some(limit as usize) } else { None };
    let offset_opt = if offset > 0 { Some(offset as usize) } else { None };

    let result = list_payments_impl(user_id, limit_opt, offset_opt);
    env.new_string(result).unwrap().into_raw()
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_com_reactnativelighteningwallet_LighteningWalletModule_nativeGetEvents(
    mut env: JNIEnv,
    _: JClass,
    user_id: JString,
) -> jstring {
    let user_id: String = env.get_string(&user_id).unwrap().into();
    let result = get_events_impl(user_id);
    env.new_string(result).unwrap().into_raw()
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_com_reactnativelighteningwallet_LighteningWalletModule_nativeDisconnect(
    mut env: JNIEnv,
    _: JClass,
    user_id: JString,
) -> jstring {
    let user_id: String = env.get_string(&user_id).unwrap().into();
    let result = disconnect_wallet_impl(user_id);
    env.new_string(result).unwrap().into_raw()
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_com_reactnativelighteningwallet_LighteningWalletModule_nativeCreateInvoice(
    mut env: JNIEnv,
    _: JClass,
    user_id: JString,
    amount_sats: i64,
    description: JString,
    expiry_secs: i32,
) -> jstring {
    let user_id: String = env.get_string(&user_id).unwrap().into();
    let amount_opt = if amount_sats > 0 { Some(amount_sats as u64) } else { None };
    let desc: String = env.get_string(&description).unwrap().into();
    let desc_opt = if desc.is_empty() { None } else { Some(desc) };

    let result = create_invoice_impl(user_id, amount_opt, desc_opt, expiry_secs as u32);
    env.new_string(result).unwrap().into_raw()
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_com_reactnativelighteningwallet_LighteningWalletModule_nativeDecodeInvoice(
    mut env: JNIEnv,
    _: JClass,
    bolt11: JString,
) -> jstring {
    let bolt11: String = env.get_string(&bolt11).unwrap().into();
    let result = decode_invoice_impl(bolt11);
    env.new_string(result).unwrap().into_raw()
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_com_reactnativelighteningwallet_LighteningWalletModule_nativePayInvoice(
    mut env: JNIEnv,
    _: JClass,
    user_id: JString,
    bolt11: JString,
    amount_sats: i64,
) -> jstring {
    let user_id: String = env.get_string(&user_id).unwrap().into();
    let bolt11: String = env.get_string(&bolt11).unwrap().into();
    let amount_opt = if amount_sats > 0 { Some(amount_sats as u64) } else { None };

    let result = pay_invoice_impl(user_id, bolt11, amount_opt);
    env.new_string(result).unwrap().into_raw()
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_com_reactnativelighteningwallet_LighteningWalletModule_nativeSendKeysend(
    mut env: JNIEnv,
    _: JClass,
    user_id: JString,
    destination_pubkey: JString,
    amount_sats: i64,
    custom_records_json: JString,
) -> jstring {
    let user_id: String = env.get_string(&user_id).unwrap().into();
    let destination_pubkey: String = env.get_string(&destination_pubkey).unwrap().into();
    let custom_records: String = env.get_string(&custom_records_json).unwrap().into();
    let custom_records_opt = if custom_records.is_empty() { None } else { Some(custom_records) };

    let result = send_keysend_impl(user_id, destination_pubkey, amount_sats as u64, custom_records_opt);
    env.new_string(result).unwrap().into_raw()
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_com_reactnativelighteningwallet_LighteningWalletModule_nativeEstimateOnchainFee(
    mut env: JNIEnv,
    _: JClass,
    user_id: JString,
    address: JString,
    amount_sats: i64,
) -> jstring {
    let user_id: String = env.get_string(&user_id).unwrap().into();
    let address: String = env.get_string(&address).unwrap().into();

    let result = estimate_onchain_fee_impl(user_id, address, amount_sats as u64);
    env.new_string(result).unwrap().into_raw()
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_com_reactnativelighteningwallet_LighteningWalletModule_nativeSendOnchain(
    mut env: JNIEnv,
    _: JClass,
    user_id: JString,
    address: JString,
    amount_sats: i64,
) -> jstring {
    let user_id: String = env.get_string(&user_id).unwrap().into();
    let address: String = env.get_string(&address).unwrap().into();

    let result = send_onchain_impl(user_id, address, amount_sats as u64);
    env.new_string(result).unwrap().into_raw()
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_com_reactnativelighteningwallet_LighteningWalletModule_nativeOpenChannel(
    mut env: JNIEnv,
    _: JClass,
    user_id: JString,
    counterparty_node_id: JString,
    channel_value_satoshis: i64,
    push_msat: i64,
    peer_address: JString,
    peer_port: i32,
) -> jstring {
    let user_id: String = env.get_string(&user_id).unwrap().into();
    let counterparty_node_id: String = env.get_string(&counterparty_node_id).unwrap().into();
    let address: String = env.get_string(&peer_address).unwrap().into();
    let address_opt = if address.is_empty() { None } else { Some(address) };
    let port_opt = if peer_port > 0 { Some(peer_port as u16) } else { None };

    let result = open_channel_impl(
        user_id,
        counterparty_node_id,
        channel_value_satoshis as u64,
        push_msat as u64,
        address_opt,
        port_opt
    );
    env.new_string(result).unwrap().into_raw()
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_com_reactnativelighteningwallet_LighteningWalletModule_nativeCloseChannel(
    mut env: JNIEnv,
    _: JClass,
    user_id: JString,
    channel_id: JString,
    force: bool,
) -> jstring {
    let user_id: String = env.get_string(&user_id).unwrap().into();
    let channel_id: String = env.get_string(&channel_id).unwrap().into();

    let result = close_channel_impl(user_id, channel_id, force);
    env.new_string(result).unwrap().into_raw()
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_com_reactnativelighteningwallet_LighteningWalletModule_nativeListChannels(
    mut env: JNIEnv,
    _: JClass,
    user_id: JString,
) -> jstring {
    let user_id: String = env.get_string(&user_id).unwrap().into();
    let result = list_channels_impl(user_id);
    env.new_string(result).unwrap().into_raw()
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_com_reactnativelighteningwallet_LighteningWalletModule_nativeConnectPeer(
    mut env: JNIEnv,
    _: JClass,
    user_id: JString,
    node_id: JString,
    address: JString,
    port: i32,
) -> jstring {
    let user_id: String = env.get_string(&user_id).unwrap().into();
    let node_id: String = env.get_string(&node_id).unwrap().into();
    let address: String = env.get_string(&address).unwrap().into();

    let result = connect_peer_impl(user_id, node_id, address, port as u16);
    env.new_string(result).unwrap().into_raw()
}

// iOS C FFI bindings
#[no_mangle]
pub extern "C" fn wallet_initialize(
    user_id: *const c_char,
    mnemonic: *const c_char,
    network: *const c_char,
    db_path: *const c_char,
) -> *mut c_char {
    let user_id = unsafe { CStr::from_ptr(user_id).to_str().unwrap() }.to_string();
    let mnemonic = unsafe { CStr::from_ptr(mnemonic).to_str().unwrap() }.to_string();
    let network = unsafe { CStr::from_ptr(network).to_str().unwrap() }.to_string();
    let db_path = unsafe { CStr::from_ptr(db_path).to_str().unwrap() }.to_string();

    let result = initialize_wallet_impl(user_id, mnemonic, network, db_path);
    CString::new(result).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn wallet_generate_mnemonic() -> *mut c_char {
    let result = generate_mnemonic_impl();
    CString::new(result).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn wallet_get_balance(user_id: *const c_char) -> *mut c_char {
    let user_id = unsafe { CStr::from_ptr(user_id).to_str().unwrap() }.to_string();
    let result = get_balance_impl(user_id);
    CString::new(result).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn wallet_sync(user_id: *const c_char) -> *mut c_char {
    let user_id = unsafe { CStr::from_ptr(user_id).to_str().unwrap() }.to_string();
    let result = sync_wallet_impl(user_id);
    CString::new(result).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn wallet_get_receiving_address(user_id: *const c_char) -> *mut c_char {
    let user_id = unsafe { CStr::from_ptr(user_id).to_str().unwrap() }.to_string();
    let result = get_receiving_address_impl(user_id);
    CString::new(result).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn wallet_list_payments(
    user_id: *const c_char,
    limit: i32,
    offset: i32,
) -> *mut c_char {
    let user_id = unsafe { CStr::from_ptr(user_id).to_str().unwrap() }.to_string();

    let limit_opt = if limit > 0 { Some(limit as usize) } else { None };
    let offset_opt = if offset > 0 { Some(offset as usize) } else { None };

    let result = list_payments_impl(user_id, limit_opt, offset_opt);
    CString::new(result).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn wallet_get_events(user_id: *const c_char) -> *mut c_char {
    let user_id = unsafe { CStr::from_ptr(user_id).to_str().unwrap() }.to_string();
    let result = get_events_impl(user_id);
    CString::new(result).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn wallet_disconnect(user_id: *const c_char) -> *mut c_char {
    let user_id = unsafe { CStr::from_ptr(user_id).to_str().unwrap() }.to_string();
    let result = disconnect_wallet_impl(user_id);
    CString::new(result).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn wallet_create_invoice(
    user_id: *const c_char,
    amount_sats: i64,
    description: *const c_char,
    expiry_secs: i32,
) -> *mut c_char {
    let user_id = unsafe { CStr::from_ptr(user_id).to_str().unwrap() }.to_string();
    let amount_opt = if amount_sats > 0 { Some(amount_sats as u64) } else { None };
    let desc = unsafe { CStr::from_ptr(description).to_str().unwrap() }.to_string();
    let desc_opt = if desc.is_empty() { None } else { Some(desc) };

    let result = create_invoice_impl(user_id, amount_opt, desc_opt, expiry_secs as u32);
    CString::new(result).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn wallet_pay_invoice(
    user_id: *const c_char,
    bolt11: *const c_char,
    amount_sats: i64,
) -> *mut c_char {
    let user_id = unsafe { CStr::from_ptr(user_id).to_str().unwrap() }.to_string();
    let bolt11 = unsafe { CStr::from_ptr(bolt11).to_str().unwrap() }.to_string();
    let amount_opt = if amount_sats > 0 { Some(amount_sats as u64) } else { None };

    let result = pay_invoice_impl(user_id, bolt11, amount_opt);
    CString::new(result).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn wallet_send_keysend(
    user_id: *const c_char,
    destination_pubkey: *const c_char,
    amount_sats: i64,
    custom_records_json: *const c_char,
) -> *mut c_char {
    let user_id = unsafe { CStr::from_ptr(user_id).to_str().unwrap() }.to_string();
    let destination_pubkey = unsafe { CStr::from_ptr(destination_pubkey).to_str().unwrap() }.to_string();
    let custom_records = unsafe { CStr::from_ptr(custom_records_json).to_str().unwrap() }.to_string();
    let custom_records_opt = if custom_records.is_empty() { None } else { Some(custom_records) };

    let result = send_keysend_impl(user_id, destination_pubkey, amount_sats as u64, custom_records_opt);
    CString::new(result).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn wallet_open_channel(
    user_id: *const c_char,
    counterparty_node_id: *const c_char,
    channel_value_satoshis: i64,
    push_msat: i64,
    peer_address: *const c_char,
    peer_port: i32,
) -> *mut c_char {
    let user_id = unsafe { CStr::from_ptr(user_id).to_str().unwrap() }.to_string();
    let counterparty_node_id = unsafe { CStr::from_ptr(counterparty_node_id).to_str().unwrap() }.to_string();
    let address = unsafe { CStr::from_ptr(peer_address).to_str().unwrap() }.to_string();
    let address_opt = if address.is_empty() { None } else { Some(address) };
    let port_opt = if peer_port > 0 { Some(peer_port as u16) } else { None };

    let result = open_channel_impl(
        user_id,
        counterparty_node_id,
        channel_value_satoshis as u64,
        push_msat as u64,
        address_opt,
        port_opt
    );
    CString::new(result).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn wallet_close_channel(
    user_id: *const c_char,
    channel_id: *const c_char,
    force: bool,
) -> *mut c_char {
    let user_id = unsafe { CStr::from_ptr(user_id).to_str().unwrap() }.to_string();
    let channel_id = unsafe { CStr::from_ptr(channel_id).to_str().unwrap() }.to_string();

    let result = close_channel_impl(user_id, channel_id, force);
    CString::new(result).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn wallet_list_channels(user_id: *const c_char) -> *mut c_char {
    let user_id = unsafe { CStr::from_ptr(user_id).to_str().unwrap() }.to_string();
    let result = list_channels_impl(user_id);
    CString::new(result).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn wallet_connect_peer(
    user_id: *const c_char,
    node_id: *const c_char,
    address: *const c_char,
    port: i32,
) -> *mut c_char {
    let user_id = unsafe { CStr::from_ptr(user_id).to_str().unwrap() }.to_string();
    let node_id = unsafe { CStr::from_ptr(node_id).to_str().unwrap() }.to_string();
    let address = unsafe { CStr::from_ptr(address).to_str().unwrap() }.to_string();

    let result = connect_peer_impl(user_id, node_id, address, port as u16);
    CString::new(result).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn wallet_free_string(s: *mut c_char) {
    unsafe {
        if s.is_null() {
            return;
        }
        let _ = CString::from_raw(s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_mnemonic() {
        let result = generate_mnemonic_impl();
        assert!(result.contains("mnemonic"));
    }
}
