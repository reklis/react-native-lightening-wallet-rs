package com.reactnativelighteningwallet

import com.facebook.react.bridge.Promise
import com.facebook.react.bridge.ReactApplicationContext
import com.facebook.react.bridge.ReactContextBaseJavaModule
import com.facebook.react.bridge.ReactMethod

class LighteningWalletModule(reactContext: ReactApplicationContext) :
    ReactContextBaseJavaModule(reactContext) {

    override fun getName(): String {
        return "LighteningWalletModule"
    }

    companion object {
        init {
            System.loadLibrary("reactnative_lightening_wallet")
        }
    }

    // Native methods
    private external fun nativeInitialize(
        userId: String,
        mnemonic: String,
        network: String,
        dbPath: String
    ): String

    private external fun nativeGenerateMnemonic(): String

    private external fun nativeGetBalance(userId: String): String

    private external fun nativeSyncWallet(userId: String): String

    private external fun nativeGetReceivingAddress(userId: String): String

    private external fun nativeListPayments(
        userId: String,
        limit: Int,
        offset: Int
    ): String

    private external fun nativeGetEvents(userId: String): String

    private external fun nativeDisconnect(userId: String): String

    // React Native bridge methods
    @ReactMethod
    fun initialize(
        userId: String,
        mnemonic: String,
        network: String,
        dbPath: String,
        promise: Promise
    ) {
        try {
            val result = nativeInitialize(userId, mnemonic, network, dbPath)
            promise.resolve(result)
        } catch (e: Exception) {
            promise.reject("INIT_ERROR", e.message, e)
        }
    }

    @ReactMethod
    fun generateMnemonic(promise: Promise) {
        try {
            val result = nativeGenerateMnemonic()
            promise.resolve(result)
        } catch (e: Exception) {
            promise.reject("MNEMONIC_ERROR", e.message, e)
        }
    }

    @ReactMethod
    fun getBalance(userId: String, promise: Promise) {
        try {
            val result = nativeGetBalance(userId)
            promise.resolve(result)
        } catch (e: Exception) {
            promise.reject("BALANCE_ERROR", e.message, e)
        }
    }

    @ReactMethod
    fun syncWallet(userId: String, promise: Promise) {
        try {
            val result = nativeSyncWallet(userId)
            promise.resolve(result)
        } catch (e: Exception) {
            promise.reject("SYNC_ERROR", e.message, e)
        }
    }

    @ReactMethod
    fun getReceivingAddress(userId: String, promise: Promise) {
        try {
            val result = nativeGetReceivingAddress(userId)
            promise.resolve(result)
        } catch (e: Exception) {
            promise.reject("ADDRESS_ERROR", e.message, e)
        }
    }

    @ReactMethod
    fun listPayments(userId: String, limit: Int, offset: Int, promise: Promise) {
        try {
            val result = nativeListPayments(userId, limit, offset)
            promise.resolve(result)
        } catch (e: Exception) {
            promise.reject("PAYMENTS_ERROR", e.message, e)
        }
    }

    @ReactMethod
    fun getEvents(userId: String, promise: Promise) {
        try {
            val result = nativeGetEvents(userId)
            promise.resolve(result)
        } catch (e: Exception) {
            promise.reject("EVENTS_ERROR", e.message, e)
        }
    }

    @ReactMethod
    fun disconnect(userId: String, promise: Promise) {
        try {
            val result = nativeDisconnect(userId)
            promise.resolve(result)
        } catch (e: Exception) {
            promise.reject("DISCONNECT_ERROR", e.message, e)
        }
    }
}
