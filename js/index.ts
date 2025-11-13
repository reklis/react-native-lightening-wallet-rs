import { NativeModules, NativeEventEmitter, Platform } from 'react-native';

const LINKING_ERROR =
  `The package 'reactnative-lightening-wallet-rs' doesn't seem to be linked. Make sure: \n\n` +
  Platform.select({ ios: "- You have run 'pod install'\n", default: '' }) +
  '- You rebuilt the app after installing the package\n' +
  '- You are not using Expo Go\n';

const LighteningWallet = NativeModules.LighteningWalletModule
  ? NativeModules.LighteningWalletModule
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
    const result = await LighteningWallet.nativeInitialize(
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
    const result = await LighteningWallet.nativeGenerateMnemonic();
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
    const result = await LighteningWallet.nativeGetBalance(this.userId);
    return parseResponse<WalletBalance>(result);
  }

  /**
   * Sync the wallet with the blockchain
   */
  async sync(): Promise<{ synced: boolean }> {
    if (!this.userId) {
      throw new Error('Wallet not initialized');
    }
    const result = await LighteningWallet.nativeSyncWallet(this.userId);
    return parseResponse<{ synced: boolean }>(result);
  }

  /**
   * Get a receiving address
   */
  async getReceivingAddress(): Promise<string> {
    if (!this.userId) {
      throw new Error('Wallet not initialized');
    }
    const result = await LighteningWallet.nativeGetReceivingAddress(this.userId);
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
    const result = await LighteningWallet.nativeListPayments(
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
    const result = await LighteningWallet.nativeGetEvents(this.userId);
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
    const result = await LighteningWallet.nativeDisconnect(this.userId);
    const data = parseResponse<{ disconnected: boolean }>(result);
    this.userId = null;
    return data;
  }
}

// Export default instance
export default new LighteningWalletAPI();

// Export class for multiple instances
export { LighteningWalletAPI as LighteningWallet };
