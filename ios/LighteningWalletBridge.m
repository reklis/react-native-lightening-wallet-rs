#import "LighteningWalletBridge.h"
#import "LighteningWallet-Bridging-Header.h"

@implementation LighteningWalletBridge

+ (NSString *)initialize:(NSString *)userId
                mnemonic:(NSString *)mnemonic
                 network:(NSString *)network
                  dbPath:(NSString *)dbPath {
    const char *userIdPtr = [userId UTF8String];
    const char *mnemonicPtr = [mnemonic UTF8String];
    const char *networkPtr = [network UTF8String];
    const char *dbPathPtr = [dbPath UTF8String];

    char *result = wallet_initialize(userIdPtr, mnemonicPtr, networkPtr, dbPathPtr);
    NSString *resultStr = result ? [NSString stringWithUTF8String:result] : nil;
    if (result) wallet_free_string(result);
    return resultStr;
}

+ (NSString *)generateMnemonic {
    char *result = wallet_generate_mnemonic();
    NSString *resultStr = result ? [NSString stringWithUTF8String:result] : nil;
    if (result) wallet_free_string(result);
    return resultStr;
}

+ (NSString *)getBalance:(NSString *)userId {
    const char *userIdPtr = [userId UTF8String];
    char *result = wallet_get_balance(userIdPtr);
    NSString *resultStr = result ? [NSString stringWithUTF8String:result] : nil;
    if (result) wallet_free_string(result);
    return resultStr;
}

+ (NSString *)sync:(NSString *)userId {
    const char *userIdPtr = [userId UTF8String];
    char *result = wallet_sync(userIdPtr);
    NSString *resultStr = result ? [NSString stringWithUTF8String:result] : nil;
    if (result) wallet_free_string(result);
    return resultStr;
}

+ (NSString *)getReceivingAddress:(NSString *)userId {
    const char *userIdPtr = [userId UTF8String];
    char *result = wallet_get_receiving_address(userIdPtr);
    NSString *resultStr = result ? [NSString stringWithUTF8String:result] : nil;
    if (result) wallet_free_string(result);
    return resultStr;
}

+ (NSString *)listPayments:(NSString *)userId
                     limit:(int32_t)limit
                    offset:(int32_t)offset {
    const char *userIdPtr = [userId UTF8String];
    char *result = wallet_list_payments(userIdPtr, limit, offset);
    NSString *resultStr = result ? [NSString stringWithUTF8String:result] : nil;
    if (result) wallet_free_string(result);
    return resultStr;
}

+ (NSString *)getEvents:(NSString *)userId {
    const char *userIdPtr = [userId UTF8String];
    char *result = wallet_get_events(userIdPtr);
    NSString *resultStr = result ? [NSString stringWithUTF8String:result] : nil;
    if (result) wallet_free_string(result);
    return resultStr;
}

+ (NSString *)disconnect:(NSString *)userId {
    const char *userIdPtr = [userId UTF8String];
    char *result = wallet_disconnect(userIdPtr);
    NSString *resultStr = result ? [NSString stringWithUTF8String:result] : nil;
    if (result) wallet_free_string(result);
    return resultStr;
}

+ (NSString *)createInvoice:(NSString *)userId
                 amountSats:(uint64_t)amountSats
                description:(NSString *)description
                expirySecs:(uint32_t)expirySecs {
    const char *userIdPtr = [userId UTF8String];
    const char *descPtr = [description UTF8String];
    char *result = wallet_create_invoice(userIdPtr, amountSats, descPtr, expirySecs);
    NSString *resultStr = result ? [NSString stringWithUTF8String:result] : nil;
    if (result) wallet_free_string(result);
    return resultStr;
}

+ (NSString *)payInvoice:(NSString *)userId
                  bolt11:(NSString *)bolt11
              amountSats:(uint64_t)amountSats {
    const char *userIdPtr = [userId UTF8String];
    const char *bolt11Ptr = [bolt11 UTF8String];
    char *result = wallet_pay_invoice(userIdPtr, bolt11Ptr, amountSats);
    NSString *resultStr = result ? [NSString stringWithUTF8String:result] : nil;
    if (result) wallet_free_string(result);
    return resultStr;
}

+ (NSString *)sendKeysend:(NSString *)userId
                   pubkey:(NSString *)pubkey
               amountSats:(uint64_t)amountSats
           customRecords:(NSString *)customRecords {
    const char *userIdPtr = [userId UTF8String];
    const char *pubkeyPtr = [pubkey UTF8String];
    const char *recordsPtr = customRecords ? [customRecords UTF8String] : NULL;
    char *result = wallet_send_keysend(userIdPtr, pubkeyPtr, amountSats, recordsPtr);
    NSString *resultStr = result ? [NSString stringWithUTF8String:result] : nil;
    if (result) wallet_free_string(result);
    return resultStr;
}

+ (NSString *)openChannel:(NSString *)userId
                   nodeId:(NSString *)nodeId
    channelValueSatoshis:(uint64_t)channelValueSatoshis
                 pushMsat:(uint64_t)pushMsat
                  address:(NSString *)address
                 peerPort:(uint16_t)peerPort {
    const char *userIdPtr = [userId UTF8String];
    const char *nodeIdPtr = [nodeId UTF8String];
    const char *addressPtr = [address UTF8String];
    char *result = wallet_open_channel(userIdPtr, nodeIdPtr, channelValueSatoshis, pushMsat, addressPtr, peerPort);
    NSString *resultStr = result ? [NSString stringWithUTF8String:result] : nil;
    if (result) wallet_free_string(result);
    return resultStr;
}

+ (NSString *)closeChannel:(NSString *)userId
                 channelId:(NSString *)channelId
                     force:(BOOL)force {
    const char *userIdPtr = [userId UTF8String];
    const char *channelIdPtr = [channelId UTF8String];
    char *result = wallet_close_channel(userIdPtr, channelIdPtr, force);
    NSString *resultStr = result ? [NSString stringWithUTF8String:result] : nil;
    if (result) wallet_free_string(result);
    return resultStr;
}

+ (NSString *)listChannels:(NSString *)userId {
    const char *userIdPtr = [userId UTF8String];
    char *result = wallet_list_channels(userIdPtr);
    NSString *resultStr = result ? [NSString stringWithUTF8String:result] : nil;
    if (result) wallet_free_string(result);
    return resultStr;
}

+ (NSString *)connectPeer:(NSString *)userId
                   nodeId:(NSString *)nodeId
                  address:(NSString *)address
                     port:(uint16_t)port {
    const char *userIdPtr = [userId UTF8String];
    const char *nodeIdPtr = [nodeId UTF8String];
    const char *addressPtr = [address UTF8String];
    char *result = wallet_connect_peer(userIdPtr, nodeIdPtr, addressPtr, port);
    NSString *resultStr = result ? [NSString stringWithUTF8String:result] : nil;
    if (result) wallet_free_string(result);
    return resultStr;
}

@end
