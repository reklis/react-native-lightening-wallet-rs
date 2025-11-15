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

    @objc(createInvoice:amountSats:description:expirySecs:resolver:rejecter:)
    func createInvoice(
        userId: String,
        amountSats: Int64,
        description: String,
        expirySecs: Int32,
        resolve: @escaping RCTPromiseResolveBlock,
        reject: @escaping RCTPromiseRejectBlock
    ) {
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            guard let self = self else { return }

            let result = userId.withCString { userIdPtr in
                description.withCString { descPtr in
                    self.wallet_create_invoice(userIdPtr, amountSats, descPtr, expirySecs)
                }
            }

            if let jsonString = self.stringFromCString(result) {
                DispatchQueue.main.async {
                    resolve(jsonString)
                }
            } else {
                DispatchQueue.main.async {
                    reject("INVOICE_ERROR", "Failed to create invoice", nil)
                }
            }
        }
    }

    @objc(payInvoice:bolt11:amountSats:resolver:rejecter:)
    func payInvoice(
        userId: String,
        bolt11: String,
        amountSats: Int64,
        resolve: @escaping RCTPromiseResolveBlock,
        reject: @escaping RCTPromiseRejectBlock
    ) {
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            guard let self = self else { return }

            let result = userId.withCString { userIdPtr in
                bolt11.withCString { bolt11Ptr in
                    self.wallet_pay_invoice(userIdPtr, bolt11Ptr, amountSats)
                }
            }

            if let jsonString = self.stringFromCString(result) {
                DispatchQueue.main.async {
                    resolve(jsonString)
                }
            } else {
                DispatchQueue.main.async {
                    reject("PAYMENT_ERROR", "Failed to pay invoice", nil)
                }
            }
        }
    }

    @objc(sendKeysend:destinationPubkey:amountSats:customRecordsJson:resolver:rejecter:)
    func sendKeysend(
        userId: String,
        destinationPubkey: String,
        amountSats: Int64,
        customRecordsJson: String,
        resolve: @escaping RCTPromiseResolveBlock,
        reject: @escaping RCTPromiseRejectBlock
    ) {
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            guard let self = self else { return }

            let result = userId.withCString { userIdPtr in
                destinationPubkey.withCString { pubkeyPtr in
                    customRecordsJson.withCString { recordsPtr in
                        self.wallet_send_keysend(userIdPtr, pubkeyPtr, amountSats, recordsPtr)
                    }
                }
            }

            if let jsonString = self.stringFromCString(result) {
                DispatchQueue.main.async {
                    resolve(jsonString)
                }
            } else {
                DispatchQueue.main.async {
                    reject("KEYSEND_ERROR", "Failed to send keysend", nil)
                }
            }
        }
    }

    @objc(openChannel:counterpartyNodeId:channelValueSatoshis:pushMsat:peerAddress:peerPort:resolver:rejecter:)
    func openChannel(
        userId: String,
        counterpartyNodeId: String,
        channelValueSatoshis: Int64,
        pushMsat: Int64,
        peerAddress: String,
        peerPort: Int32,
        resolve: @escaping RCTPromiseResolveBlock,
        reject: @escaping RCTPromiseRejectBlock
    ) {
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            guard let self = self else { return }

            let result = userId.withCString { userIdPtr in
                counterpartyNodeId.withCString { nodeIdPtr in
                    peerAddress.withCString { addressPtr in
                        self.wallet_open_channel(userIdPtr, nodeIdPtr, channelValueSatoshis, pushMsat, addressPtr, peerPort)
                    }
                }
            }

            if let jsonString = self.stringFromCString(result) {
                DispatchQueue.main.async {
                    resolve(jsonString)
                }
            } else {
                DispatchQueue.main.async {
                    reject("CHANNEL_ERROR", "Failed to open channel", nil)
                }
            }
        }
    }

    @objc(closeChannel:channelId:force:resolver:rejecter:)
    func closeChannel(
        userId: String,
        channelId: String,
        force: Bool,
        resolve: @escaping RCTPromiseResolveBlock,
        reject: @escaping RCTPromiseRejectBlock
    ) {
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            guard let self = self else { return }

            let result = userId.withCString { userIdPtr in
                channelId.withCString { channelIdPtr in
                    self.wallet_close_channel(userIdPtr, channelIdPtr, force)
                }
            }

            if let jsonString = self.stringFromCString(result) {
                DispatchQueue.main.async {
                    resolve(jsonString)
                }
            } else {
                DispatchQueue.main.async {
                    reject("CHANNEL_ERROR", "Failed to close channel", nil)
                }
            }
        }
    }

    @objc(listChannels:resolver:rejecter:)
    func listChannels(
        userId: String,
        resolve: @escaping RCTPromiseResolveBlock,
        reject: @escaping RCTPromiseRejectBlock
    ) {
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            guard let self = self else { return }

            let result = userId.withCString { userIdPtr in
                self.wallet_list_channels(userIdPtr)
            }

            if let jsonString = self.stringFromCString(result) {
                DispatchQueue.main.async {
                    resolve(jsonString)
                }
            } else {
                DispatchQueue.main.async {
                    reject("CHANNEL_ERROR", "Failed to list channels", nil)
                }
            }
        }
    }

    @objc(connectPeer:nodeId:address:port:resolver:rejecter:)
    func connectPeer(
        userId: String,
        nodeId: String,
        address: String,
        port: Int32,
        resolve: @escaping RCTPromiseResolveBlock,
        reject: @escaping RCTPromiseRejectBlock
    ) {
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            guard let self = self else { return }

            let result = userId.withCString { userIdPtr in
                nodeId.withCString { nodeIdPtr in
                    address.withCString { addressPtr in
                        self.wallet_connect_peer(userIdPtr, nodeIdPtr, addressPtr, port)
                    }
                }
            }

            if let jsonString = self.stringFromCString(result) {
                DispatchQueue.main.async {
                    resolve(jsonString)
                }
            } else {
                DispatchQueue.main.async {
                    reject("PEER_ERROR", "Failed to connect to peer", nil)
                }
            }
        }
    }

    @objc
    static func requiresMainQueueSetup() -> Bool {
        return false
    }
}
