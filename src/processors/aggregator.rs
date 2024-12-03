use crate::nonce_manager::NonceManager;
use crate::tx_builder::TxBuilder;
use crate::tx_sender::TxSender;
use near_event_listener::EventLog;

use async_trait::async_trait;
use near_jsonrpc_client::methods;
use near_primitives::views::TxExecutionStatus;
use near_sdk::AccountId;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use super::TransactionProcessor;

pub struct Aggregator {
    nonce_manager: Arc<NonceManager>,
    tx_builder: Arc<Mutex<TxBuilder>>,
    tx_sender: Arc<TxSender>,
    account_id: AccountId,
}

impl Aggregator {
    pub fn new(
        nonce_manager: Arc<NonceManager>,
        tx_builder: Arc<Mutex<TxBuilder>>,
        tx_sender: Arc<TxSender>,
        account_id: AccountId,
    ) -> Self {
        Self {
            nonce_manager,
            tx_builder,
            tx_sender,
            account_id,
        }
    }
}

#[async_trait]
impl TransactionProcessor for Aggregator {
    async fn process_transaction(
        &self,
        event_data: EventLog,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        println!("Aggregator Processor");
        println!("Event Data: {:?}", event_data);

        let aggregator_attempts = 30;

        for _attempt in 0..aggregator_attempts {
            // Get stage to synchronize
            let stage_result = self
                .get_stage(self.tx_sender.client.clone(), event_data.clone())
                .await?;
            let stage = stage_result.trim_matches('"').to_string();
            println!("Current Stage: {:?}", stage);

            if stage == "Ended" {
                match self::obtain_top_ten(self, event_data).await {
                    Ok(_) => {
                        return Ok(true);
                    }
                    Err(e) => {
                        println!("Failed to obtain top ten voters: {}", e);
                        return Err(e);
                    }
                }
            } else {
                println!("Waiting for Ended stage...");
                sleep(Duration::from_secs(10)).await;
            }
        }

        println!(
            "Failed to reach Ended stage after {} attempts",
            aggregator_attempts
        );
        Ok(false)
    }

    async fn commit(
        &self,
        _event_data: EventLog,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    async fn reveal(
        &self,
        _event_data: EventLog,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}
pub async fn obtain_top_ten(
    aggregator: &Aggregator,
    event_data: EventLog,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Obtaining top ten voters");

    let (nonce, block_hash) = aggregator.nonce_manager.get_nonce_and_tx_hash().await?;

    let mut tx_builder = aggregator.tx_builder.lock().await;

    let request_id = event_data.data[0]["request_id"]
        .as_str()
        .unwrap_or_default()
        .to_string();

    let (tx, _) = tx_builder
        .with_method_name("get_top_10_voters")
        .with_args(serde_json::json!({
            "request_id": request_id,
        }))
        .build(nonce, block_hash);

    let signer = &tx_builder.signer;

    let request = methods::send_tx::RpcSendTransactionRequest {
        signed_transaction: tx.sign(signer),
        wait_until: TxExecutionStatus::Final,
    };

    let tx_response = aggregator.tx_sender.send_transaction(request).await?;
    let log_tx = aggregator.extract_logs(&tx_response);
    println!("TOP_TEN LOG: {:?}", log_tx);

    Ok(())
}
