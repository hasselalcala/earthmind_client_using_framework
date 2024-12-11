use near_sdk::{env, AccountId};

pub fn calculate_request_id(sender_account: AccountId, message: String) -> String {
    let concatenated_answer = format!("{}{}", sender_account, message);
    let new_request_id = env::keccak256(concatenated_answer.as_bytes());
    let new_request_id_hex = hex::encode(new_request_id);
    return new_request_id_hex;
}
