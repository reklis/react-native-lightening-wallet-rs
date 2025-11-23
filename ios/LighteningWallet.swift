import Foundation
import React

@objc(LighteningWallet)
class LighteningWallet: NSObject {

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
        DispatchQueue.global(qos: .userInitiated).async {
            if let result = LighteningWalletBridge.initialize(userId, mnemonic: mnemonic, network: network, dbPath: dbPath) {
                DispatchQueue.main.async {
                    resolve(result)
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
        DispatchQueue.global(qos: .userInitiated).async {
            if let result = LighteningWalletBridge.generateMnemonic() {
                DispatchQueue.main.async {
                    resolve(result)
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
        DispatchQueue.global(qos: .userInitiated).async {
            if let result = LighteningWalletBridge.getBalance(userId) {
                DispatchQueue.main.async {
                    resolve(result)
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
        DispatchQueue.global(qos: .userInitiated).async {
            if let result = LighteningWalletBridge.sync(userId) {
                DispatchQueue.main.async {
                    resolve(result)
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
        DispatchQueue.global(qos: .userInitiated).async {
            if let result = LighteningWalletBridge.getReceivingAddress(userId) {
                DispatchQueue.main.async {
                    resolve(result)
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
        DispatchQueue.global(qos: .userInitiated).async {
            if let result = LighteningWalletBridge.listPayments(userId, limit: limit, offset: offset) {
                DispatchQueue.main.async {
                    resolve(result)
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
        DispatchQueue.global(qos: .userInitiated).async {
            if let result = LighteningWalletBridge.getEvents(userId) {
                DispatchQueue.main.async {
                    resolve(result)
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
        DispatchQueue.global(qos: .userInitiated).async {
            if let result = LighteningWalletBridge.disconnect(userId) {
                DispatchQueue.main.async {
                    resolve(result)
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
        DispatchQueue.global(qos: .userInitiated).async {
            if let result = LighteningWalletBridge.createInvoice(userId, amountSats: UInt64(amountSats), description: description, expirySecs: UInt32(expirySecs)) {
                DispatchQueue.main.async {
                    resolve(result)
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
        DispatchQueue.global(qos: .userInitiated).async {
            if let result = LighteningWalletBridge.payInvoice(userId, bolt11: bolt11, amountSats: UInt64(amountSats)) {
                DispatchQueue.main.async {
                    resolve(result)
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
        DispatchQueue.global(qos: .userInitiated).async {
            if let result = LighteningWalletBridge.sendKeysend(userId, pubkey: destinationPubkey, amountSats: UInt64(amountSats), customRecords: customRecordsJson) {
                DispatchQueue.main.async {
                    resolve(result)
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
        DispatchQueue.global(qos: .userInitiated).async {
            if let result = LighteningWalletBridge.openChannel(userId, nodeId: counterpartyNodeId, channelValueSatoshis: UInt64(channelValueSatoshis), pushMsat: UInt64(pushMsat), address: peerAddress, peerPort: UInt16(peerPort)) {
                DispatchQueue.main.async {
                    resolve(result)
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
        DispatchQueue.global(qos: .userInitiated).async {
            if let result = LighteningWalletBridge.closeChannel(userId, channelId: channelId, force: force) {
                DispatchQueue.main.async {
                    resolve(result)
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
        DispatchQueue.global(qos: .userInitiated).async {
            if let result = LighteningWalletBridge.listChannels(userId) {
                DispatchQueue.main.async {
                    resolve(result)
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
        DispatchQueue.global(qos: .userInitiated).async {
            if let result = LighteningWalletBridge.connectPeer(userId, nodeId: nodeId, address: address, port: UInt16(port)) {
                DispatchQueue.main.async {
                    resolve(result)
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
