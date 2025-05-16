use did::utils::msg_caller;

use crate::storage::config::Config;
use crate::utils::trap;

#[ic_cdk::inspect_message]
pub fn inspect() {
    let method = ic_cdk::api::msg_method_name();

    match method.as_str() {
        "request_file"
        | "get_requests"
        | "upload_file_atomic"
        | "share_file"
        | "share_file_with_users"
        | "revoke_file_sharing"
        | "get_allowed_users"
        | "get_shared_files" => {
            if msg_caller() != Config::get_owner() {
                trap("Only the owner can call this method");
            }
        }
        _ => {}
    }

    ic_cdk::api::accept_message();
}
