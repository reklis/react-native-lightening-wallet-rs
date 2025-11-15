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
export type WalletEvent = {
    type: 'BalanceUpdated';
    onchain_confirmed: number;
    onchain_unconfirmed: number;
    lightning_balance: number;
    total: number;
} | {
    type: 'PaymentReceived';
    payment_hash: string;
    amount_sats: number;
    description?: string;
} | {
    type: 'PaymentSent';
    payment_hash: string;
    amount_sats: number;
    fee_sats?: number;
    destination?: string;
} | {
    type: 'PaymentFailed';
    payment_hash: string;
    reason: string;
} | {
    type: 'ChannelOpened';
    channel_id: string;
    counterparty: string;
    capacity_sats: number;
} | {
    type: 'ChannelClosed';
    channel_id: string;
    reason: string;
} | {
    type: 'SyncStarted';
} | {
    type: 'SyncCompleted';
    duration_ms: number;
} | {
    type: 'SyncFailed';
    error: string;
} | {
    type: 'TransactionConfirmed';
    txid: string;
    confirmations: number;
} | {
    type: 'Error';
    message: string;
};
export declare class LighteningWalletAPI {
    private userId;
    private eventEmitter;
    /**
     * Initialize a wallet for a user
     */
    initialize(userId: string, mnemonic: string, network: 'bitcoin' | 'testnet', dbPath: string): Promise<{
        userId: string;
        initialized: boolean;
    }>;
    /**
     * Generate a new BIP39 mnemonic phrase
     */
    static generateMnemonic(): Promise<string>;
    /**
     * Get the wallet balance
     */
    getBalance(): Promise<WalletBalance>;
    /**
     * Sync the wallet with the blockchain
     */
    sync(): Promise<{
        synced: boolean;
    }>;
    /**
     * Get a receiving address
     */
    getReceivingAddress(): Promise<string>;
    /**
     * List payment history
     */
    listPayments(limit?: number, offset?: number): Promise<Payment[]>;
    /**
     * Get pending wallet events
     */
    getEvents(): Promise<WalletEvent[]>;
    /**
     * Subscribe to wallet events
     */
    onEvent(callback: (event: WalletEvent) => void): () => void;
    /**
     * Disconnect and cleanup the wallet
     */
    disconnect(): Promise<{
        disconnected: boolean;
    }>;
    /**
     * Create a Lightning invoice
     */
    createInvoice(amountSats?: number, description?: string, expirySecs?: number): Promise<InvoiceInfo>;
    /**
     * Pay a Lightning invoice
     */
    payInvoice(bolt11: string, amountSats?: number): Promise<Payment>;
    /**
     * Send a keysend payment (for Podcasting 2.0)
     */
    sendKeysend(destinationPubkey: string, amountSats: number, customRecords?: Record<number, Uint8Array>): Promise<Payment>;
    /**
     * Open a Lightning channel
     */
    openChannel(counterpartyNodeId: string, channelValueSatoshis: number, pushMsat?: number, peerAddress?: string, peerPort?: number): Promise<{
        channel_id: string;
    }>;
    /**
     * Close a Lightning channel
     */
    closeChannel(channelId: string, force?: boolean): Promise<{
        closed: boolean;
    }>;
    /**
     * List all Lightning channels
     */
    listChannels(): Promise<ChannelInfo[]>;
    /**
     * Connect to a Lightning peer
     */
    connectPeer(nodeId: string, address: string, port: number): Promise<{
        connected: boolean;
    }>;
}
declare const _default: LighteningWalletAPI;
export default _default;
export { LighteningWalletAPI as LighteningWallet };
