#import <React/RCTBridgeModule.h>

@interface RCT_EXTERN_MODULE(LighteningWallet, NSObject)

RCT_EXTERN_METHOD(initialize:(NSString *)userId
                  mnemonic:(NSString *)mnemonic
                  network:(NSString *)network
                  dbPath:(NSString *)dbPath
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(generateMnemonic:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(getBalance:(NSString *)userId
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(syncWallet:(NSString *)userId
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(getReceivingAddress:(NSString *)userId
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(listPayments:(NSString *)userId
                  limit:(int32_t)limit
                  offset:(int32_t)offset
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(getEvents:(NSString *)userId
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(disconnect:(NSString *)userId
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(createInvoice:(NSString *)userId
                  amountSats:(int64_t)amountSats
                  description:(NSString *)description
                  expirySecs:(int32_t)expirySecs
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(payInvoice:(NSString *)userId
                  bolt11:(NSString *)bolt11
                  amountSats:(int64_t)amountSats
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(sendKeysend:(NSString *)userId
                  destinationPubkey:(NSString *)destinationPubkey
                  amountSats:(int64_t)amountSats
                  customRecordsJson:(NSString *)customRecordsJson
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(openChannel:(NSString *)userId
                  counterpartyNodeId:(NSString *)counterpartyNodeId
                  channelValueSatoshis:(int64_t)channelValueSatoshis
                  pushMsat:(int64_t)pushMsat
                  peerAddress:(NSString *)peerAddress
                  peerPort:(int32_t)peerPort
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(closeChannel:(NSString *)userId
                  channelId:(NSString *)channelId
                  force:(BOOL)force
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(listChannels:(NSString *)userId
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(connectPeer:(NSString *)userId
                  nodeId:(NSString *)nodeId
                  address:(NSString *)address
                  port:(int32_t)port
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)

@end
