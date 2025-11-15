package com.reactnativelighteningwallet;

import android.util.Log;

import androidx.annotation.NonNull;

import com.facebook.react.bridge.Promise;
import com.facebook.react.bridge.ReactApplicationContext;
import com.facebook.react.bridge.ReactContextBaseJavaModule;
import com.facebook.react.bridge.ReactMethod;
import com.facebook.react.module.annotations.ReactModule;

@ReactModule(name = LighteningWalletModule.NAME)
public class LighteningWalletModule extends ReactContextBaseJavaModule {
    public static final String NAME = "LighteningWalletModule";
    private static final String TAG = "LighteningWallet";

    static {
        try {
            System.loadLibrary("reactnative_lightening_wallet");
            Log.d(TAG, "Successfully loaded reactnative_lightening_wallet library");
        } catch (UnsatisfiedLinkError e) {
            Log.e(TAG, "Failed to load reactnative_lightening_wallet library", e);
            throw e;
        }
    }

    // Native method declarations - these match the JNI implementations in Rust
    private static native String nativeInitialize(String userId, String mnemonic, String network, String dbPath);
    private static native String nativeGenerateMnemonic();
    private static native String nativeSyncWallet(String userId);
    private static native String nativeGetBalance(String userId);
    private static native String nativeGetReceivingAddress(String userId);
    private static native String nativeGetEvents(String userId);
    private static native String nativeCreateInvoice(String userId, long amountSats, String description, int expirySecs);
    private static native String nativePayInvoice(String userId, String bolt11, long amountSats);
    private static native String nativeSendKeysend(String userId, String destinationPubkey, long amountSats, String customRecordsJson);
    private static native String nativeListPayments(String userId, int limit, int offset);
    private static native String nativeOpenChannel(String userId, String counterpartyNodeId, long channelValueSatoshis, long pushMsat, String peerAddress, int peerPort);
    private static native String nativeCloseChannel(String userId, String channelId, boolean force);
    private static native String nativeListChannels(String userId);
    private static native String nativeConnectPeer(String userId, String nodeId, String address, int port);
    private static native void nativeDisconnect(String userId);

    public LighteningWalletModule(ReactApplicationContext reactContext) {
        super(reactContext);
    }

    @Override
    @NonNull
    public String getName() {
        return NAME;
    }

    @ReactMethod
    public void initialize(String userId, String mnemonic, String network, String dbPath, Promise promise) {
        try {
            String result = nativeInitialize(userId, mnemonic, network, dbPath);
            promise.resolve(result);
        } catch (Exception e) {
            Log.e(TAG, "initialize error", e);
            promise.reject("INITIALIZE_ERROR", e.getMessage(), e);
        }
    }

    @ReactMethod
    public void generateMnemonic(Promise promise) {
        try {
            String result = nativeGenerateMnemonic();
            promise.resolve(result);
        } catch (Exception e) {
            Log.e(TAG, "generateMnemonic error", e);
            promise.reject("GENERATE_MNEMONIC_ERROR", e.getMessage(), e);
        }
    }

    @ReactMethod
    public void sync(String userId, Promise promise) {
        try {
            String result = nativeSyncWallet(userId);
            promise.resolve(result);
        } catch (Exception e) {
            Log.e(TAG, "sync error", e);
            promise.reject("SYNC_ERROR", e.getMessage(), e);
        }
    }

    @ReactMethod
    public void getEvents(String userId, Promise promise) {
        try {
            String result = nativeGetEvents(userId);
            promise.resolve(result);
        } catch (Exception e) {
            Log.e(TAG, "getEvents error", e);
            promise.reject("GET_EVENTS_ERROR", e.getMessage(), e);
        }
    }

    @ReactMethod
    public void getBalance(String userId, Promise promise) {
        try {
            String result = nativeGetBalance(userId);
            promise.resolve(result);
        } catch (Exception e) {
            Log.e(TAG, "getBalance error", e);
            promise.reject("GET_BALANCE_ERROR", e.getMessage(), e);
        }
    }

    @ReactMethod
    public void getReceivingAddress(String userId, Promise promise) {
        try {
            String result = nativeGetReceivingAddress(userId);
            promise.resolve(result);
        } catch (Exception e) {
            Log.e(TAG, "getReceivingAddress error", e);
            promise.reject("GET_ADDRESS_ERROR", e.getMessage(), e);
        }
    }

    @ReactMethod
    public void createInvoice(String userId, double amountSats, String description, double expirySecs, Promise promise) {
        try {
            long amount = amountSats < 0 ? -1 : (long) amountSats;
            int expiry = (int) expirySecs;
            String result = nativeCreateInvoice(userId, amount, description, expiry);
            promise.resolve(result);
        } catch (Exception e) {
            Log.e(TAG, "createInvoice error", e);
            promise.reject("CREATE_INVOICE_ERROR", e.getMessage(), e);
        }
    }

    @ReactMethod
    public void payInvoice(String userId, String bolt11, double amountSats, Promise promise) {
        try {
            long amount = amountSats < 0 ? -1 : (long) amountSats;
            String result = nativePayInvoice(userId, bolt11, amount);
            promise.resolve(result);
        } catch (Exception e) {
            Log.e(TAG, "payInvoice error", e);
            promise.reject("PAY_INVOICE_ERROR", e.getMessage(), e);
        }
    }

    @ReactMethod
    public void sendKeysend(String userId, String destinationPubkey, double amountSats, String customRecordsJson, Promise promise) {
        try {
            long amount = (long) amountSats;
            String result = nativeSendKeysend(userId, destinationPubkey, amount, customRecordsJson);
            promise.resolve(result);
        } catch (Exception e) {
            Log.e(TAG, "sendKeysend error", e);
            promise.reject("SEND_KEYSEND_ERROR", e.getMessage(), e);
        }
    }

    @ReactMethod
    public void listPayments(String userId, double limit, double offset, Promise promise) {
        try {
            int limitInt = (int) limit;
            int offsetInt = (int) offset;
            String result = nativeListPayments(userId, limitInt, offsetInt);
            promise.resolve(result);
        } catch (Exception e) {
            Log.e(TAG, "listPayments error", e);
            promise.reject("LIST_PAYMENTS_ERROR", e.getMessage(), e);
        }
    }

    @ReactMethod
    public void openChannel(String userId, String counterpartyNodeId, double channelValueSatoshis, double pushMsat, String peerAddress, double peerPort, Promise promise) {
        try {
            long channelValue = (long) channelValueSatoshis;
            long push = (long) pushMsat;
            int port = (int) peerPort;
            String result = nativeOpenChannel(userId, counterpartyNodeId, channelValue, push, peerAddress, port);
            promise.resolve(result);
        } catch (Exception e) {
            Log.e(TAG, "openChannel error", e);
            promise.reject("OPEN_CHANNEL_ERROR", e.getMessage(), e);
        }
    }

    @ReactMethod
    public void closeChannel(String userId, String channelId, boolean force, Promise promise) {
        try {
            String result = nativeCloseChannel(userId, channelId, force);
            promise.resolve(result);
        } catch (Exception e) {
            Log.e(TAG, "closeChannel error", e);
            promise.reject("CLOSE_CHANNEL_ERROR", e.getMessage(), e);
        }
    }

    @ReactMethod
    public void listChannels(String userId, Promise promise) {
        try {
            String result = nativeListChannels(userId);
            promise.resolve(result);
        } catch (Exception e) {
            Log.e(TAG, "listChannels error", e);
            promise.reject("LIST_CHANNELS_ERROR", e.getMessage(), e);
        }
    }

    @ReactMethod
    public void connectPeer(String userId, String nodeId, String address, double port, Promise promise) {
        try {
            int portInt = (int) port;
            String result = nativeConnectPeer(userId, nodeId, address, portInt);
            promise.resolve(result);
        } catch (Exception e) {
            Log.e(TAG, "connectPeer error", e);
            promise.reject("CONNECT_PEER_ERROR", e.getMessage(), e);
        }
    }

    @ReactMethod
    public void disconnect(String userId, Promise promise) {
        try {
            nativeDisconnect(userId);
            promise.resolve(null);
        } catch (Exception e) {
            Log.e(TAG, "disconnect error", e);
            promise.reject("DISCONNECT_ERROR", e.getMessage(), e);
        }
    }
}
