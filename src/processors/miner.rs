use near_event_listener::EventLog;
use near_tx_qx_builder::{NearTxSender, NearQxSender};
use async_trait::async_trait;
use near_sdk::AccountId;
use near_crypto::SecretKey;
use serde_json::json;
use tokio::time::{sleep, Duration};

use crate::constants::ACCOUNT_TO_LISTEN;
use super::TransactionProcessor;

pub struct Miner {
    account_id: AccountId,
    private_key: SecretKey, 
    rpc_url: String,
}

impl Miner {
    pub fn new(
        account_id: AccountId,
        private_key: SecretKey,
        rpc_url: String,
    ) -> Self {
        Self {
            account_id,
            private_key,
            rpc_url,
        }
    }
}

#[async_trait]
impl TransactionProcessor for Miner {
    async fn process_transaction(
        &self,
        event_data: EventLog,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        println!("Miner Processor");
        println!("Miner Event Data: {:?}", event_data);

        let commit_attempts = 30;
        let reveal_attempts = 30;
        let mut committed = false;

        // Wait for CommitMiners stage
        for _attempt in 0..commit_attempts {
            let stage_result = self
                .get_stage(self.rpc_url.clone(), self.account_id.clone(), event_data.clone())
                .await?;
            let stage = stage_result.trim_matches('"').to_string();
            println!("Current Stage: {:?}", stage);

            if stage == "CommitMiners" {
                match self.commit(event_data.clone()).await {
                    Ok(_) => {
                        committed = true;
                        break;
                    }
                    Err(e) => {
                        println!("Failed to commit by miner: {}", e);
                        return Err(e);
                    }
                }
            } else if stage == "RevealMiners"
                || stage == "CommitValidators"
                || stage == "RevealValidators"
                || stage == "Ended"
            {
                println!("Commit stage passed without committing, skipping transaction.");
                return Ok(false);
            } else {
                println!("Waiting for CommitMiners stage...");
                sleep(Duration::from_secs(10)).await;
            }
        }

        if !committed {
            println!("Failed to reach CommitMiners stage, skipping transaction.");
            return Ok(false);
        }

        // Wait for RevealMiners stage
        for _attempt in 0..reveal_attempts {
            let stage_result = self
                .get_stage(self.rpc_url.clone(), self.account_id.clone(), event_data.clone())
                .await?;
            let stage = stage_result.trim_matches('"').to_string();
            println!("Current Stage: {:?}", stage);

            if stage == "RevealMiners" {
                match self.reveal(event_data.clone()).await {
                    Ok(_) => {
                        return Ok(true);
                    }
                    Err(e) => {
                        println!("Failed to reveal by miner: {}", e);
                        return Err(e);
                    }
                }
            } else if stage == "CommitValidators" || stage == "RevealValidators" || stage == "Ended"
            {
                println!("RevealMiner stage has ended");
                return Ok(false);
            } else {
                println!("Waiting for RevealMiners stage...");
                sleep(Duration::from_secs(10)).await;
            }
        }

        println!("Failed to reach appropriate stages after multiple attempts.");
        Ok(false)
    }

    async fn commit(
        &self,
        event_data: EventLog,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("Miner Commit");

        let request_id = event_data.data[0]["request_id"]
            .as_str()
            .unwrap_or_default()
            .to_string();

        let query_sender = NearQxSender::builder(&self.rpc_url)
            .account_receiver(ACCOUNT_TO_LISTEN)
            .method_name("hash_miner_answer")
            .args(json!({"miner": self.account_id.to_string(),
                    "request_id": request_id,
                    "answer": true,
                    "message": "It's the best option"
            }))
            .build()?;

        let query_result = query_sender.send_query().await?;
        let answer_hash = query_result.trim_matches('"');

        let tx_sender = NearTxSender::builder(&self.rpc_url)
            .account_sender(self.account_id.as_str())
            .account_receiver(ACCOUNT_TO_LISTEN)
            .use_private_key(&self.private_key.to_string())
            .method_name("commit_by_miner")
            .args(json!({
                "request_id": request_id,
                "answer": answer_hash}))
            .build()?;

        let tx_response = tx_sender.send_transaction().await?;
        let log_tx = self.extract_logs(&tx_response);

        println!("COMMIT_MINER_LOG: {:?}", log_tx);

        Ok(())
    }

    async fn reveal(
        &self,
        event_data: EventLog,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("Reveal by miner");

        let request_id = event_data.data[0]["request_id"]
            .as_str()
            .unwrap_or_default()
            .to_string();

        let tx_sender = NearTxSender::builder(&self.rpc_url)
            .account_sender(self.account_id.as_str())
            .account_receiver(ACCOUNT_TO_LISTEN)
            .use_private_key(&self.private_key.to_string())
            .method_name("reveal_by_miner")
            .args(json!({
                "request_id": request_id,
                "answer": true,
                "message" : "It's the best option"}))
            .build()?;

        let tx_response = tx_sender.send_transaction().await?;
        let log_tx = self.extract_logs(&tx_response);
        println!("REVEAL_MINER_LOG: {:?}", log_tx);

        Ok(())
    }
}
