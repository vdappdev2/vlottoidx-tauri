mod rpc;
mod commands;

use commands::{
    AppState, greet,
    get_info, get_wallet_info, get_mining_info, z_get_total_balance, get_blockchain_info,
    list_identities, get_identity, get_identity_content, verify_message, verify_hash, list_currencies, get_currency, get_currency_converters, get_currency_state, get_offers, list_open_offers, close_offers, make_offer, take_offer,
    discover_chains, get_discovered_chains, connect_to_chain, test_and_connect_manual, store_credentials,
    load_credentials, clear_credentials, list_stored_chains, get_expected_config_paths,
    get_block_count, list_address_groupings, estimate_conversion, convert_currency, send_currency,
    convert_to_verusidx, register_name_commitment, register_identity, revoke_identity, recover_identity, update_identity, set_identity_timelock,
    list_transactions, get_currency_balance, get_new_address, get_addresses_by_account,
    z_get_new_address, z_list_addresses, z_get_operation_status, define_currency, send_raw_transaction,
    send_ticket_to_graveyard
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();
    
    // Add app configuration
    builder = builder
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_info,
            get_wallet_info,
            get_mining_info,
            z_get_total_balance,
            get_blockchain_info,
            list_identities,
            get_identity,
            get_identity_content,
            verify_message,
            verify_hash,
            list_currencies,
            get_currency,
            get_currency_converters,
            get_currency_state,
            get_offers,
            list_open_offers,
            close_offers,
            make_offer,
            take_offer,
            discover_chains,
            get_discovered_chains,
            connect_to_chain,
            test_and_connect_manual,
            store_credentials,
            load_credentials,
            clear_credentials,
            list_stored_chains,
            get_expected_config_paths,
            get_block_count,
            list_address_groupings,
            estimate_conversion,
            convert_currency,
            send_currency,
            convert_to_verusidx,
            register_name_commitment,
            register_identity,
            revoke_identity,
            recover_identity,
            update_identity,
            set_identity_timelock,
            list_transactions,
            get_currency_balance,
            get_new_address,
            get_addresses_by_account,
            z_get_new_address,
            z_list_addresses,
            z_get_operation_status,
            define_currency,
            send_raw_transaction,
            send_ticket_to_graveyard
        ]);
    
    // Add the opener plugin
    builder = builder.plugin(tauri_plugin_opener::init());
    
    
    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
