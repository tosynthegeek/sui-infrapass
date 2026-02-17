use sui_json_rpc_types::SuiTransactionBlockResponse;

pub mod address;
pub mod coin;
pub mod config;
pub mod constants;

pub fn handle_response(resp: &SuiTransactionBlockResponse) {
    match resp.status_ok() {
        Some(true) => {
            println!("Transaction succeeded");
        }
        Some(false) => {
            println!("Transaction failed");
        }
        None => {
            println!("No execution status returned");
        }
    }
}
