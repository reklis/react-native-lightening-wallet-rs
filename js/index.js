"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g = Object.create((typeof Iterator === "function" ? Iterator : Object).prototype);
    return g.next = verb(0), g["throw"] = verb(1), g["return"] = verb(2), typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (g && (g = 0, op[0] && (_ = 0)), _) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.LighteningWallet = exports.LighteningWalletAPI = void 0;
var react_native_1 = require("react-native");
var LINKING_ERROR = "The package 'reactnative-lightening-wallet-rs' doesn't seem to be linked. Make sure: \n\n" +
    react_native_1.Platform.select({ ios: "- You have run 'pod install'\n", default: '' }) +
    '- You rebuilt the app after installing the package\n' +
    '- You are not using Expo Go\n';
// Module name is different on iOS vs Android
var moduleName = react_native_1.Platform.OS === 'ios' ? 'LighteningWallet' : 'LighteningWalletModule';
var LighteningWallet = react_native_1.NativeModules[moduleName]
    ? react_native_1.NativeModules[moduleName]
    : new Proxy({}, {
        get: function () {
            throw new Error(LINKING_ERROR);
        },
    });
// Helper to parse responses
function parseResponse(jsonString) {
    try {
        var response = JSON.parse(jsonString);
        if (response.success && response.data) {
            return response.data;
        }
        else {
            throw new Error(response.error || 'Unknown error');
        }
    }
    catch (error) {
        throw new Error("Failed to parse response: ".concat(error));
    }
}
// API
var LighteningWalletAPI = /** @class */ (function () {
    function LighteningWalletAPI() {
        this.userId = null;
        this.eventEmitter = new react_native_1.NativeEventEmitter(LighteningWallet);
    }
    /**
     * Initialize a wallet for a user
     */
    LighteningWalletAPI.prototype.initialize = function (userId, mnemonic, network, dbPath) {
        return __awaiter(this, void 0, void 0, function () {
            var methodName, result, data;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0:
                        methodName = react_native_1.Platform.OS === 'ios' ? 'initialize' : 'nativeInitialize';
                        return [4 /*yield*/, LighteningWallet[methodName](userId, mnemonic, network, dbPath)];
                    case 1:
                        result = _a.sent();
                        data = parseResponse(result);
                        this.userId = userId;
                        return [2 /*return*/, data];
                }
            });
        });
    };
    /**
     * Generate a new BIP39 mnemonic phrase
     */
    LighteningWalletAPI.generateMnemonic = function () {
        return __awaiter(this, void 0, void 0, function () {
            var methodName, result, data;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0:
                        methodName = react_native_1.Platform.OS === 'ios' ? 'generateMnemonic' : 'nativeGenerateMnemonic';
                        return [4 /*yield*/, LighteningWallet[methodName]()];
                    case 1:
                        result = _a.sent();
                        data = parseResponse(result);
                        return [2 /*return*/, data.mnemonic];
                }
            });
        });
    };
    /**
     * Get the wallet balance
     */
    LighteningWalletAPI.prototype.getBalance = function () {
        return __awaiter(this, void 0, void 0, function () {
            var methodName, result;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0:
                        if (!this.userId) {
                            throw new Error('Wallet not initialized');
                        }
                        methodName = react_native_1.Platform.OS === 'ios' ? 'getBalance' : 'nativeGetBalance';
                        return [4 /*yield*/, LighteningWallet[methodName](this.userId)];
                    case 1:
                        result = _a.sent();
                        return [2 /*return*/, parseResponse(result)];
                }
            });
        });
    };
    /**
     * Sync the wallet with the blockchain
     */
    LighteningWalletAPI.prototype.sync = function () {
        return __awaiter(this, void 0, void 0, function () {
            var methodName, result;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0:
                        if (!this.userId) {
                            throw new Error('Wallet not initialized');
                        }
                        methodName = react_native_1.Platform.OS === 'ios' ? 'syncWallet' : 'nativeSyncWallet';
                        return [4 /*yield*/, LighteningWallet[methodName](this.userId)];
                    case 1:
                        result = _a.sent();
                        return [2 /*return*/, parseResponse(result)];
                }
            });
        });
    };
    /**
     * Get a receiving address
     */
    LighteningWalletAPI.prototype.getReceivingAddress = function () {
        return __awaiter(this, void 0, void 0, function () {
            var methodName, result, data;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0:
                        if (!this.userId) {
                            throw new Error('Wallet not initialized');
                        }
                        methodName = react_native_1.Platform.OS === 'ios' ? 'getReceivingAddress' : 'nativeGetReceivingAddress';
                        return [4 /*yield*/, LighteningWallet[methodName](this.userId)];
                    case 1:
                        result = _a.sent();
                        data = parseResponse(result);
                        return [2 /*return*/, data.address];
                }
            });
        });
    };
    /**
     * List payment history
     */
    LighteningWalletAPI.prototype.listPayments = function (limit, offset) {
        return __awaiter(this, void 0, void 0, function () {
            var methodName, result;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0:
                        if (!this.userId) {
                            throw new Error('Wallet not initialized');
                        }
                        methodName = react_native_1.Platform.OS === 'ios' ? 'listPayments' : 'nativeListPayments';
                        return [4 /*yield*/, LighteningWallet[methodName](this.userId, limit || 0, offset || 0)];
                    case 1:
                        result = _a.sent();
                        return [2 /*return*/, parseResponse(result)];
                }
            });
        });
    };
    /**
     * Get pending wallet events
     */
    LighteningWalletAPI.prototype.getEvents = function () {
        return __awaiter(this, void 0, void 0, function () {
            var methodName, result;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0:
                        if (!this.userId) {
                            throw new Error('Wallet not initialized');
                        }
                        methodName = react_native_1.Platform.OS === 'ios' ? 'getEvents' : 'nativeGetEvents';
                        return [4 /*yield*/, LighteningWallet[methodName](this.userId)];
                    case 1:
                        result = _a.sent();
                        return [2 /*return*/, parseResponse(result)];
                }
            });
        });
    };
    /**
     * Subscribe to wallet events
     */
    LighteningWalletAPI.prototype.onEvent = function (callback) {
        var subscription = this.eventEmitter.addListener('WalletEvent', callback);
        return function () { return subscription.remove(); };
    };
    /**
     * Disconnect and cleanup the wallet
     */
    LighteningWalletAPI.prototype.disconnect = function () {
        return __awaiter(this, void 0, void 0, function () {
            var methodName, result, data;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0:
                        if (!this.userId) {
                            throw new Error('Wallet not initialized');
                        }
                        methodName = react_native_1.Platform.OS === 'ios' ? 'disconnect' : 'nativeDisconnect';
                        return [4 /*yield*/, LighteningWallet[methodName](this.userId)];
                    case 1:
                        result = _a.sent();
                        data = parseResponse(result);
                        this.userId = null;
                        return [2 /*return*/, data];
                }
            });
        });
    };
    /**
     * Create a Lightning invoice
     */
    LighteningWalletAPI.prototype.createInvoice = function (amountSats, description, expirySecs) {
        return __awaiter(this, void 0, void 0, function () {
            var methodName, result;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0:
                        if (!this.userId) {
                            throw new Error('Wallet not initialized');
                        }
                        methodName = react_native_1.Platform.OS === 'ios' ? 'createInvoice' : 'nativeCreateInvoice';
                        return [4 /*yield*/, LighteningWallet[methodName](this.userId, amountSats || 0, description || '', expirySecs || 3600)];
                    case 1:
                        result = _a.sent();
                        return [2 /*return*/, parseResponse(result)];
                }
            });
        });
    };
    /**
     * Pay a Lightning invoice
     */
    LighteningWalletAPI.prototype.payInvoice = function (bolt11, amountSats) {
        return __awaiter(this, void 0, void 0, function () {
            var methodName, result;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0:
                        if (!this.userId) {
                            throw new Error('Wallet not initialized');
                        }
                        methodName = react_native_1.Platform.OS === 'ios' ? 'payInvoice' : 'nativePayInvoice';
                        return [4 /*yield*/, LighteningWallet[methodName](this.userId, bolt11, amountSats || 0)];
                    case 1:
                        result = _a.sent();
                        return [2 /*return*/, parseResponse(result)];
                }
            });
        });
    };
    /**
     * Send a keysend payment (for Podcasting 2.0)
     */
    LighteningWalletAPI.prototype.sendKeysend = function (destinationPubkey, amountSats, customRecords) {
        return __awaiter(this, void 0, void 0, function () {
            var customRecordsJson, methodName, result;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0:
                        if (!this.userId) {
                            throw new Error('Wallet not initialized');
                        }
                        customRecordsJson = customRecords
                            ? JSON.stringify(Object.fromEntries(Object.entries(customRecords).map(function (_a) {
                                var k = _a[0], v = _a[1];
                                return [k, Array.from(v)];
                            })))
                            : '';
                        methodName = react_native_1.Platform.OS === 'ios' ? 'sendKeysend' : 'nativeSendKeysend';
                        return [4 /*yield*/, LighteningWallet[methodName](this.userId, destinationPubkey, amountSats, customRecordsJson)];
                    case 1:
                        result = _a.sent();
                        return [2 /*return*/, parseResponse(result)];
                }
            });
        });
    };
    /**
     * Open a Lightning channel
     */
    LighteningWalletAPI.prototype.openChannel = function (counterpartyNodeId, channelValueSatoshis, pushMsat, peerAddress, peerPort) {
        return __awaiter(this, void 0, void 0, function () {
            var methodName, result;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0:
                        if (!this.userId) {
                            throw new Error('Wallet not initialized');
                        }
                        methodName = react_native_1.Platform.OS === 'ios' ? 'openChannel' : 'nativeOpenChannel';
                        return [4 /*yield*/, LighteningWallet[methodName](this.userId, counterpartyNodeId, channelValueSatoshis, pushMsat || 0, peerAddress || '', peerPort || 0)];
                    case 1:
                        result = _a.sent();
                        return [2 /*return*/, parseResponse(result)];
                }
            });
        });
    };
    /**
     * Close a Lightning channel
     */
    LighteningWalletAPI.prototype.closeChannel = function (channelId_1) {
        return __awaiter(this, arguments, void 0, function (channelId, force) {
            var methodName, result;
            if (force === void 0) { force = false; }
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0:
                        if (!this.userId) {
                            throw new Error('Wallet not initialized');
                        }
                        methodName = react_native_1.Platform.OS === 'ios' ? 'closeChannel' : 'nativeCloseChannel';
                        return [4 /*yield*/, LighteningWallet[methodName](this.userId, channelId, force)];
                    case 1:
                        result = _a.sent();
                        return [2 /*return*/, parseResponse(result)];
                }
            });
        });
    };
    /**
     * List all Lightning channels
     */
    LighteningWalletAPI.prototype.listChannels = function () {
        return __awaiter(this, void 0, void 0, function () {
            var methodName, result;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0:
                        if (!this.userId) {
                            throw new Error('Wallet not initialized');
                        }
                        methodName = react_native_1.Platform.OS === 'ios' ? 'listChannels' : 'nativeListChannels';
                        return [4 /*yield*/, LighteningWallet[methodName](this.userId)];
                    case 1:
                        result = _a.sent();
                        return [2 /*return*/, parseResponse(result)];
                }
            });
        });
    };
    /**
     * Connect to a Lightning peer
     */
    LighteningWalletAPI.prototype.connectPeer = function (nodeId, address, port) {
        return __awaiter(this, void 0, void 0, function () {
            var methodName, result;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0:
                        if (!this.userId) {
                            throw new Error('Wallet not initialized');
                        }
                        methodName = react_native_1.Platform.OS === 'ios' ? 'connectPeer' : 'nativeConnectPeer';
                        return [4 /*yield*/, LighteningWallet[methodName](this.userId, nodeId, address, port)];
                    case 1:
                        result = _a.sent();
                        return [2 /*return*/, parseResponse(result)];
                }
            });
        });
    };
    return LighteningWalletAPI;
}());
exports.LighteningWalletAPI = LighteningWalletAPI;
exports.LighteningWallet = LighteningWalletAPI;
// Export default instance
exports.default = new LighteningWalletAPI();
