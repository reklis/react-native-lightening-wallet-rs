import Foundation

@objc(LighteningWallet)
class LighteningWallet: NSObject {

    // MARK: - Native C Functions (imported from Rust via bridging header)

    // MARK: - Helper Methods

    private func stringFromCString(_ cString: UnsafeMutablePointer<CChar>?) -> String? {
        guard let cString = cString else { return nil }
        let string = String(cString: cString)
        wallet_free_string(cString)
        return string
    }

    // MARK: - React Native Methods

    @objc(initialize:mnemonic:network:dbPath:resolver:rejecter:)
    func initialize(
        userId: String,
        mnemonic: String,
        network: String,
        dbPath: String,
        resolve: @escaping RCTPromiseResolveBlock,
        reject: @escaping RCTPromiseRejectBlock
    ) {
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            guard let self = self else { return }

            let result = userId.withCString { userIdPtr in
                mnemonic.withCString { mnemonicPtr in
                    network.withCString { networkPtr in
                        dbPath.withCString { dbPathPtr in
                            self.wallet_initialize(userIdPtr, mnemonicPtr, networkPtr, dbPathPtr)
                        }
                    }
                }
            }

            if let jsonString = self.stringFromCString(result) {
                DispatchQueue.main.async {
                    resolve(jsonString)
                }
            } else {
                DispatchQueue.main.async {
                    reject("INIT_ERROR", "Failed to initialize wallet", nil)
                }
            }
        }
    }

    @objc(generateMnemonic:rejecter:)
    func generateMnemonic(
        resolve: @escaping RCTPromiseResolveBlock,
        reject: @escaping RCTPromiseRejectBlock
    ) {
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            guard let self = self else { return }

            let result = self.wallet_generate_mnemonic()

            if let jsonString = self.stringFromCString(result) {
                DispatchQueue.main.async {
                    resolve(jsonString)
                }
            } else {
                DispatchQueue.main.async {
                    reject("MNEMONIC_ERROR", "Failed to generate mnemonic", nil)
                }
            }
        }
    }

    @objc(getBalance:resolver:rejecter:)
    func getBalance(
        userId: String,
        resolve: @escaping RCTPromiseResolveBlock,
        reject: @escaping RCTPromiseRejectBlock
    ) {
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            guard let self = self else { return }

            let result = userId.withCString { userIdPtr in
                self.wallet_get_balance(userIdPtr)
            }

            if let jsonString = self.stringFromCString(result) {
                DispatchQueue.main.async {
                    resolve(jsonString)
                }
            } else {
                DispatchQueue.main.async {
                    reject("BALANCE_ERROR", "Failed to get balance", nil)
                }
            }
        }
    }

    @objc(syncWallet:resolver:rejecter:)
    func syncWallet(
        userId: String,
        resolve: @escaping RCTPromiseResolveBlock,
        reject: @escaping RCTPromiseRejectBlock
    ) {
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            guard let self = self else { return }

            let result = userId.withCString { userIdPtr in
                self.wallet_sync(userIdPtr)
            }

            if let jsonString = self.stringFromCString(result) {
                DispatchQueue.main.async {
                    resolve(jsonString)
                }
            } else {
                DispatchQueue.main.async {
                    reject("SYNC_ERROR", "Failed to sync wallet", nil)
                }
            }
        }
    }

    @objc(getReceivingAddress:resolver:rejecter:)
    func getReceivingAddress(
        userId: String,
        resolve: @escaping RCTPromiseResolveBlock,
        reject: @escaping RCTPromiseRejectBlock
    ) {
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            guard let self = self else { return }

            let result = userId.withCString { userIdPtr in
                self.wallet_get_receiving_address(userIdPtr)
            }

            if let jsonString = self.stringFromCString(result) {
                DispatchQueue.main.async {
                    resolve(jsonString)
                }
            } else {
                DispatchQueue.main.async {
                    reject("ADDRESS_ERROR", "Failed to get address", nil)
                }
            }
        }
    }

    @objc(listPayments:limit:offset:resolver:rejecter:)
    func listPayments(
        userId: String,
        limit: Int32,
        offset: Int32,
        resolve: @escaping RCTPromiseResolveBlock,
        reject: @escaping RCTPromiseRejectBlock
    ) {
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            guard let self = self else { return }

            let result = userId.withCString { userIdPtr in
                self.wallet_list_payments(userIdPtr, limit, offset)
            }

            if let jsonString = self.stringFromCString(result) {
                DispatchQueue.main.async {
                    resolve(jsonString)
                }
            } else {
                DispatchQueue.main.async {
                    reject("PAYMENTS_ERROR", "Failed to list payments", nil)
                }
            }
        }
    }

    @objc(getEvents:resolver:rejecter:)
    func getEvents(
        userId: String,
        resolve: @escaping RCTPromiseResolveBlock,
        reject: @escaping RCTPromiseRejectBlock
    ) {
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            guard let self = self else { return }

            let result = userId.withCString { userIdPtr in
                self.wallet_get_events(userIdPtr)
            }

            if let jsonString = self.stringFromCString(result) {
                DispatchQueue.main.async {
                    resolve(jsonString)
                }
            } else {
                DispatchQueue.main.async {
                    reject("EVENTS_ERROR", "Failed to get events", nil)
                }
            }
        }
    }

    @objc(disconnect:resolver:rejecter:)
    func disconnect(
        userId: String,
        resolve: @escaping RCTPromiseResolveBlock,
        reject: @escaping RCTPromiseRejectBlock
    ) {
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            guard let self = self else { return }

            let result = userId.withCString { userIdPtr in
                self.wallet_disconnect(userIdPtr)
            }

            if let jsonString = self.stringFromCString(result) {
                DispatchQueue.main.async {
                    resolve(jsonString)
                }
            } else {
                DispatchQueue.main.async {
                    reject("DISCONNECT_ERROR", "Failed to disconnect", nil)
                }
            }
        }
    }

    @objc
    static func requiresMainQueueSetup() -> Bool {
        return false
    }
}
