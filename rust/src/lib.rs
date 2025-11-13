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
    let network = match network.as_str() {
        "bitcoin" => Network::Bitcoin,
        "testnet" | "bitcoinTestnet" => Network::Testnet,
        _ => return Response::error("Invalid network".to_string()),
    };

    match WalletCoordinator::new(&db_path) {
        Ok(coordinator) => {
            match coordinator.initialize(&mnemonic, network) {
                Ok(_) => {
                    let mut wallets = WALLETS.lock().unwrap();
                    wallets.insert(user_id.clone(), Arc::new(coordinator));
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
        match lightning_node.get_total_onchain_balance() {
            Ok(onchain_balance) => {
                let balances = json!({
                    "onchain_confirmed": onchain_balance,
                    "onchain_unconfirmed": 0, // LDK doesn't expose this separately
                    "lightning_balance": 0, // TODO: Calculate from channels
                    "total": onchain_balance
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
