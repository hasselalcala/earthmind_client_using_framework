use super::TransactionProcessor;
use async_trait::async_trait;
use near_crypto::SecretKey;
use near_event_listener::EventLog;
use near_sdk::AccountId;
use near_tx_qx_builder::NearTxSender;
use serde_json::json;
use tokio::time::{sleep, Duration};

use crate::constants::ACCOUNT_TO_LISTEN;

pub struct Aggregator {
    account_id: AccountId,
    private_key: SecretKey,
    rpc_url: String,
}

impl Aggregator {
    pub fn new(account_id: AccountId, private_key: SecretKey, rpc_url: String) -> Self {
        Self {
            account_id,
            private_key,
            rpc_url,
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
                .get_stage(
                    self.rpc_url.clone(),
                    self.account_id.clone(),
                    event_data.clone(),
                )
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

    let request_id = event_data.data[0]["request_id"]
        .as_str()
        .unwrap_or_default()
        .to_string();

    let tx_sender = NearTxSender::builder(&aggregator.rpc_url)
        .account_sender(aggregator.account_id.as_str())
        .account_receiver(ACCOUNT_TO_LISTEN)
        .use_private_key(&aggregator.private_key.to_string())
        .method_name("get_top_10_voters")
        .args(json!({"request_id": request_id}))
        .build()?;

    let tx_response = tx_sender.send_transaction().await?;
    let log_tx = aggregator.extract_logs(&tx_response);
    println!("TOP_TEN LOG: {:?}", log_tx);

    Ok(())
}
