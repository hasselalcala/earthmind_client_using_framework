use clap::Parser;
use near_event_listener::NearEventListener;
use std::sync::Arc;

mod cli;
mod constants;
mod processors;

use cli::{Cli, Modes, Networks};
use constants::{
    ACCOUNT_TO_LISTEN, FUNCTION_TO_LISTEN, NEAR_RPC_MAINNET,
    NEAR_RPC_TESTNET,
};

use processors::{Aggregator, Miner, TransactionProcessor, Validator};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let rpc_url = match cli.network {
        Networks::Testnet => NEAR_RPC_TESTNET,
        Networks::Mainnet => NEAR_RPC_MAINNET,
    };


    let processor: Arc<dyn TransactionProcessor> = match cli.mode {
        Modes::Miner => Arc::new(Miner::new(
            cli.account_id.clone(),
            cli.private_key.clone(),
            rpc_url.to_string(),
        )),
        Modes::Validator => Arc::new(Validator::new(
            cli.account_id.clone(),
            cli.private_key.clone(),
            rpc_url.to_string(),
        )),
        Modes::Aggregator => Arc::new(Aggregator::new(
            cli.account_id.clone(),
            cli.private_key.clone(),
            rpc_url.to_string(),
        )),
    };

    let mut listener = NearEventListener::builder(rpc_url)
        .account_id(ACCOUNT_TO_LISTEN)
        .method_name(FUNCTION_TO_LISTEN)
        .last_processed_block(181088453)
        .build()?;

    listener
        .start(move |event_log| {
            println!("Standard: {}", event_log.standard);
            println!("Version: {}", event_log.version);
            println!("Event: {}", event_log.event);
            println!("Data: {}", event_log.data);

            let processor = processor.clone();
            tokio::spawn(async move {
                if let Err(e) = processor.process_transaction(event_log).await {
                    eprintln!("Error processing transaction: {}", e);
                }
            });
        })
        .await?;

    Ok(())
}
