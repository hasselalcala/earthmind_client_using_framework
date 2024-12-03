use near_sdk::AccountId;

pub fn generate_validator_answer() -> Vec<AccountId> {
    let value = vec![
        "hasserualcala.testnet".parse().unwrap(),
        "miner2.near".parse().unwrap(),
        "miner3.near".parse().unwrap(),
        "miner4.near".parse().unwrap(),
        "miner5.near".parse().unwrap(),
        "miner6.near".parse().unwrap(),
        "miner7.near".parse().unwrap(),
        "miner8.near".parse().unwrap(),
        "miner9.near".parse().unwrap(),
        "miner10.near".parse().unwrap(),
    ];
    value
}
