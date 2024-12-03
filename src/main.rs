use clap::Parser;
use near_crypto::InMemorySigner;
use near_event_listener::NearEventListener;
use near_jsonrpc_client::JsonRpcClient;
use std::sync::Arc;
use tokio::sync::Mutex;

mod cli;
mod constants;
mod nonce_manager;
mod processors;
mod qx_builder;
mod qx_sender;
mod tx_builder;
mod tx_sender;

use cli::{Cli, Modes, Networks};
use constants::{
    ACCOUNT_TO_LISTEN, DEFAULT_TIMEOUT, FUNCTION_TO_LISTEN, NEAR_RPC_MAINNET,
    NEAR_RPC_TESTNET,
};
use nonce_manager::NonceManager;
use processors::{Aggregator, Miner, TransactionProcessor, Validator};
use tx_builder::TxBuilder;
use tx_sender::TxSender;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let rpc_url = match cli.network {
        Networks::Testnet => NEAR_RPC_TESTNET,
        Networks::Mainnet => NEAR_RPC_MAINNET,
    };

    //initialize client
    let client = Arc::new(JsonRpcClient::connect(rpc_url));

    // Create signer
    let signer = InMemorySigner::from_secret_key(cli.account_id.clone(), cli.private_key.clone());

    // Initialize components
    let nonce_manager = Arc::new(NonceManager::new(client.clone(), Arc::new(signer.clone())));
    let tx_builder = Arc::new(Mutex::new(TxBuilder::new(signer, cli.network)));
    let tx_sender = Arc::new(TxSender::new(client.clone(), DEFAULT_TIMEOUT));

    // Create the processor based on the mode
    let processor: Arc<dyn TransactionProcessor> = match cli.mode {
        Modes::Miner => Arc::new(Miner::new(
            nonce_manager.clone(),
            tx_builder.clone(),
            tx_sender.clone(),
            cli.account_id.clone(),
        )),
        Modes::Validator => Arc::new(Validator::new(
            nonce_manager.clone(),
            tx_builder.clone(),
            tx_sender.clone(),
            cli.account_id.clone(),
        )),
        Modes::Aggregator => Arc::new(Aggregator::new(
            nonce_manager.clone(),
            tx_builder.clone(),
            tx_sender.clone(),
            cli.account_id,
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
