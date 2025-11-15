import { NativeModules, NativeEventEmitter, Platform } from 'react-native';

const LINKING_ERROR =
  `The package 'reactnative-lightening-wallet-rs' doesn't seem to be linked. Make sure: \n\n` +
  Platform.select({ ios: "- You have run 'pod install'\n", default: '' }) +
  '- You rebuilt the app after installing the package\n' +
  '- You are not using Expo Go\n';

// Module name is different on iOS vs Android
const moduleName = Platform.OS === 'ios' ? 'LighteningWallet' : 'LighteningWalletModule';

const LighteningWallet = NativeModules[moduleName]
  ? NativeModules[moduleName]
  : new Proxy(
      {},
      {
        get() {
          throw new Error(LINKING_ERROR);
        },
      }
    );

// Types
export interface WalletBalance {
  onchain_confirmed: number;
  onchain_unconfirmed: number;
  lightning_balance: number;
  lightning_receivable: number;
  claimable_balance: number;
  total: number;
  pending: number;
}

export interface Payment {
  id?: number;
  payment_hash: string;
  payment_type: 'sent' | 'received' | 'onchain_sent' | 'onchain_received';
  amount_sats: number;
  fee_sats?: number;
  status: 'pending' | 'completed' | 'failed';
  timestamp: number;
  description?: string;
  destination?: string;
  txid?: string;
  preimage?: string;
  bolt11?: string;
}

export interface ChannelInfo {
  channel_id: string;
  counterparty_node_id: string;
  channel_value_sats: number;
  balance_sats: number;
  outbound_capacity_sats: number;
  inbound_capacity_sats: number;
  is_usable: boolean;
  is_public: boolean;
  is_ready: boolean;
  is_closing: boolean;
  confirmations_required?: number;
}

export interface InvoiceInfo {
  bolt11: string;
  payment_hash: string;
  amount_sats?: number;
  description?: string;
  created_at: number;
  expires_at: number;
}

export type WalletEvent =
  | { type: 'BalanceUpdated'; onchain_confirmed: number; onchain_unconfirmed: number; lightning_balance: number; total: number }
  | { type: 'PaymentReceived'; payment_hash: string; amount_sats: number; description?: string }
  | { type: 'PaymentSent'; payment_hash: string; amount_sats: number; fee_sats?: number; destination?: string }
  | { type: 'PaymentFailed'; payment_hash: string; reason: string }
  | { type: 'ChannelOpened'; channel_id: string; counterparty: string; capacity_sats: number }
  | { type: 'ChannelClosed'; channel_id: string; reason: string }
  | { type: 'SyncStarted' }
  | { type: 'SyncCompleted'; duration_ms: number }
  | { type: 'SyncFailed'; error: string }
  | { type: 'TransactionConfirmed'; txid: string; confirmations: number }
  | { type: 'Error'; message: string };

interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}

// Helper to parse responses
function parseResponse<T>(jsonString: string): T {
  try {
    const response: ApiResponse<T> = JSON.parse(jsonString);
    if (response.success && response.data) {
      return response.data;
    } else {
      throw new Error(response.error || 'Unknown error');
    }
  } catch (error) {
    throw new Error(`Failed to parse response: ${error}`);
  }
}

// API
export class LighteningWalletAPI {
  private userId: string | null = null;
  private eventEmitter = new NativeEventEmitter(LighteningWallet);

  /**
   * Initialize a wallet for a user
   */
  async initialize(
    userId: string,
    mnemonic: string,
    network: 'bitcoin' | 'testnet',
    dbPath: string
  ): Promise<{ userId: string; initialized: boolean }> {
    const methodName = Platform.OS === 'ios' ? 'initialize' : 'nativeInitialize';
    const result = await LighteningWallet[methodName](
      userId,
      mnemonic,
      network,
      dbPath
    );
    const data = parseResponse<{ userId: string; initialized: boolean }>(result);
    this.userId = userId;
    return data;
  }

  /**
   * Generate a new BIP39 mnemonic phrase
   */
  static async generateMnemonic(): Promise<string> {
    const methodName = Platform.OS === 'ios' ? 'generateMnemonic' : 'nativeGenerateMnemonic';
    const result = await LighteningWallet[methodName]();
    const data = parseResponse<{ mnemonic: string }>(result);
    return data.mnemonic;
  }

  /**
   * Get the wallet balance
   */
  async getBalance(): Promise<WalletBalance> {
    if (!this.userId) {
      throw new Error('Wallet not initialized');
    }
    const methodName = Platform.OS === 'ios' ? 'getBalance' : 'nativeGetBalance';
    const result = await LighteningWallet[methodName](this.userId);
    return parseResponse<WalletBalance>(result);
  }

  /**
   * Sync the wallet with the blockchain
   */
  async sync(): Promise<{ synced: boolean }> {
    if (!this.userId) {
      throw new Error('Wallet not initialized');
    }
    const methodName = Platform.OS === 'ios' ? 'syncWallet' : 'nativeSyncWallet';
    const result = await LighteningWallet[methodName](this.userId);
    return parseResponse<{ synced: boolean }>(result);
  }

  /**
   * Get a receiving address
   */
  async getReceivingAddress(): Promise<string> {
    if (!this.userId) {
      throw new Error('Wallet not initialized');
    }
    const methodName = Platform.OS === 'ios' ? 'getReceivingAddress' : 'nativeGetReceivingAddress';
    const result = await LighteningWallet[methodName](this.userId);
    const data = parseResponse<{ address: string }>(result);
    return data.address;
  }

  /**
   * List payment history
   */
  async listPayments(limit?: number, offset?: number): Promise<Payment[]> {
    if (!this.userId) {
      throw new Error('Wallet not initialized');
    }
    const methodName = Platform.OS === 'ios' ? 'listPayments' : 'nativeListPayments';
    const result = await LighteningWallet[methodName](
      this.userId,
      limit || 0,
      offset || 0
    );
    return parseResponse<Payment[]>(result);
  }

  /**
   * Get pending wallet events
   */
  async getEvents(): Promise<WalletEvent[]> {
    if (!this.userId) {
      throw new Error('Wallet not initialized');
    }
    const methodName = Platform.OS === 'ios' ? 'getEvents' : 'nativeGetEvents';
    const result = await LighteningWallet[methodName](this.userId);
    return parseResponse<WalletEvent[]>(result);
  }

  /**
   * Subscribe to wallet events
   */
  onEvent(callback: (event: WalletEvent) => void): () => void {
    const subscription = this.eventEmitter.addListener(
      'WalletEvent',
      callback
    );
    return () => subscription.remove();
  }

  /**
   * Disconnect and cleanup the wallet
   */
  async disconnect(): Promise<{ disconnected: boolean }> {
    if (!this.userId) {
      throw new Error('Wallet not initialized');
    }
    const methodName = Platform.OS === 'ios' ? 'disconnect' : 'nativeDisconnect';
    const result = await LighteningWallet[methodName](this.userId);
    const data = parseResponse<{ disconnected: boolean }>(result);
    this.userId = null;
    return data;
  }

  /**
   * Create a Lightning invoice
   */
  async createInvoice(
    amountSats?: number,
    description?: string,
    expirySecs?: number
  ): Promise<InvoiceInfo> {
    if (!this.userId) {
      throw new Error('Wallet not initialized');
    }
    const methodName = Platform.OS === 'ios' ? 'createInvoice' : 'nativeCreateInvoice';
    const result = await LighteningWallet[methodName](
      this.userId,
      amountSats || 0,
      description || '',
      expirySecs || 3600
    );
    return parseResponse<InvoiceInfo>(result);
  }

  /**
   * Pay a Lightning invoice
   */
  async payInvoice(bolt11: string, amountSats?: number): Promise<Payment> {
    if (!this.userId) {
      throw new Error('Wallet not initialized');
    }
    const methodName = Platform.OS === 'ios' ? 'payInvoice' : 'nativePayInvoice';
    const result = await LighteningWallet[methodName](
      this.userId,
      bolt11,
      amountSats || 0
    );
    return parseResponse<Payment>(result);
  }

  /**
   * Send a keysend payment (for Podcasting 2.0)
   */
  async sendKeysend(
    destinationPubkey: string,
    amountSats: number,
    customRecords?: Record<number, Uint8Array>
  ): Promise<Payment> {
    if (!this.userId) {
      throw new Error('Wallet not initialized');
    }

    // Convert custom records to JSON format
    const customRecordsJson = customRecords
      ? JSON.stringify(
          Object.fromEntries(
            Object.entries(customRecords).map(([k, v]) => [k, Array.from(v)])
          )
        )
      : '';

    const methodName = Platform.OS === 'ios' ? 'sendKeysend' : 'nativeSendKeysend';
    const result = await LighteningWallet[methodName](
      this.userId,
      destinationPubkey,
      amountSats,
      customRecordsJson
    );
    return parseResponse<Payment>(result);
  }

  /**
   * Open a Lightning channel
   */
  async openChannel(
    counterpartyNodeId: string,
    channelValueSatoshis: number,
    pushMsat?: number,
    peerAddress?: string,
    peerPort?: number
  ): Promise<{ channel_id: string }> {
    if (!this.userId) {
      throw new Error('Wallet not initialized');
    }
    const methodName = Platform.OS === 'ios' ? 'openChannel' : 'nativeOpenChannel';
    const result = await LighteningWallet[methodName](
      this.userId,
      counterpartyNodeId,
      channelValueSatoshis,
      pushMsat || 0,
      peerAddress || '',
      peerPort || 0
    );
    return parseResponse<{ channel_id: string }>(result);
  }

  /**
   * Close a Lightning channel
   */
  async closeChannel(channelId: string, force: boolean = false): Promise<{ closed: boolean }> {
    if (!this.userId) {
      throw new Error('Wallet not initialized');
    }
    const methodName = Platform.OS === 'ios' ? 'closeChannel' : 'nativeCloseChannel';
    const result = await LighteningWallet[methodName](
      this.userId,
      channelId,
      force
    );
    return parseResponse<{ closed: boolean }>(result);
  }

  /**
   * List all Lightning channels
   */
  async listChannels(): Promise<ChannelInfo[]> {
    if (!this.userId) {
      throw new Error('Wallet not initialized');
    }
    const methodName = Platform.OS === 'ios' ? 'listChannels' : 'nativeListChannels';
    const result = await LighteningWallet[methodName](this.userId);
    return parseResponse<ChannelInfo[]>(result);
  }

  /**
   * Connect to a Lightning peer
   */
  async connectPeer(
    nodeId: string,
    address: string,
    port: number
  ): Promise<{ connected: boolean }> {
    if (!this.userId) {
      throw new Error('Wallet not initialized');
    }
    const methodName = Platform.OS === 'ios' ? 'connectPeer' : 'nativeConnectPeer';
    const result = await LighteningWallet[methodName](
      this.userId,
      nodeId,
      address,
      port
    );
    return parseResponse<{ connected: boolean }>(result);
  }
}

// Export default instance
export default new LighteningWalletAPI();

// Export class for multiple instances
export { LighteningWalletAPI as LighteningWallet };

// Export types
export type { WalletBalance, Payment, ChannelInfo, InvoiceInfo, WalletEvent };
