#import <Foundation/Foundation.h>

@interface LighteningWalletBridge : NSObject

+ (NSString *)initialize:(NSString *)userId
                mnemonic:(NSString *)mnemonic
                 network:(NSString *)network
                  dbPath:(NSString *)dbPath;

+ (NSString *)generateMnemonic;

+ (NSString *)getBalance:(NSString *)userId;

+ (NSString *)sync:(NSString *)userId;

+ (NSString *)getReceivingAddress:(NSString *)userId;

+ (NSString *)listPayments:(NSString *)userId
                     limit:(int32_t)limit
                    offset:(int32_t)offset;

+ (NSString *)getEvents:(NSString *)userId;

+ (NSString *)disconnect:(NSString *)userId;

+ (NSString *)createInvoice:(NSString *)userId
                 amountSats:(uint64_t)amountSats
                description:(NSString *)description
                expirySecs:(uint32_t)expirySecs;

+ (NSString *)payInvoice:(NSString *)userId
                  bolt11:(NSString *)bolt11
              amountSats:(uint64_t)amountSats;

+ (NSString *)sendKeysend:(NSString *)userId
                   pubkey:(NSString *)pubkey
               amountSats:(uint64_t)amountSats
           customRecords:(NSString *)customRecords;

+ (NSString *)openChannel:(NSString *)userId
                   nodeId:(NSString *)nodeId
    channelValueSatoshis:(uint64_t)channelValueSatoshis
                 pushMsat:(uint64_t)pushMsat
                  address:(NSString *)address
                 peerPort:(uint16_t)peerPort;

+ (NSString *)closeChannel:(NSString *)userId
                 channelId:(NSString *)channelId
                     force:(BOOL)force;

+ (NSString *)listChannels:(NSString *)userId;

+ (NSString *)connectPeer:(NSString *)userId
                   nodeId:(NSString *)nodeId
                  address:(NSString *)address
                     port:(uint16_t)port;

@end
