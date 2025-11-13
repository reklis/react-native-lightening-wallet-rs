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
void wallet_free_string(char* s);

#endif /* LighteningWallet_Bridging_Header_h */
