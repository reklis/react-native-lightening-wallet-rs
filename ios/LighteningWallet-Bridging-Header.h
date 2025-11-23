#ifndef LighteningWallet_Bridging_Header_h
#define LighteningWallet_Bridging_Header_h

#include <stdint.h>

// Rust FFI function declarations
char* wallet_initialize(const char* user_id, const char* mnemonic, const char* network, const char* db_path);
char* wallet_generate_mnemonic(void);
char* wallet_get_balance(const char* user_id);
char* wallet_sync(const char* user_id);
char* wallet_get_receiving_address(const char* user_id);
char* wallet_list_payments(const char* user_id, int32_t limit, int32_t offset);
char* wallet_get_events(const char* user_id);
char* wallet_disconnect(const char* user_id);
char* wallet_create_invoice(const char* user_id, uint64_t amount_sats, const char* description, uint32_t expiry_secs);
char* wallet_pay_invoice(const char* user_id, const char* bolt11, uint64_t amount_sats);
char* wallet_send_keysend(const char* user_id, const char* pubkey, uint64_t amount_sats, const char* custom_records);
char* wallet_open_channel(const char* user_id, const char* node_id, uint64_t channel_value_satoshis, uint64_t push_msat, const char* address, uint16_t peer_port);
char* wallet_close_channel(const char* user_id, const char* channel_id, _Bool force);
char* wallet_list_channels(const char* user_id);
char* wallet_connect_peer(const char* user_id, const char* node_id, const char* address, uint16_t port);
void wallet_free_string(char* s);

#endif /* LighteningWallet_Bridging_Header_h */
