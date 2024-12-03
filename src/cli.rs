use clap::{Parser, ValueEnum};
use near_crypto::SecretKey;
use near_sdk::AccountId;

#[derive(Parser)]
#[command(name = "earthmind_client_listener")]
#[command(about = "A Near block listener with different modes", long_about = None)]
pub struct Cli {
    #[arg(long, value_enum)]
    pub mode: Modes,

    #[arg(long)]
    pub account_id: AccountId,

    #[arg(long)]
    pub private_key: SecretKey,

    #[arg(long, default_value = "testnet")]
    pub network: Networks,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Modes {
    Miner,
    Validator,
    Aggregator,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Networks {
    Testnet,
    Mainnet,
}
