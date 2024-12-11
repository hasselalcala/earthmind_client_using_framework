use near_event_listener::EventLog;
use async_trait::async_trait;
use near_sdk::AccountId;
use serde_json::json;
use near_crypto::SecretKey;
use tokio::time::{sleep, Duration};

use near_tx_qx_builder::{NearTxSender, NearQxSender};

use super::utils;
use super::TransactionProcessor;
use crate::constants::ACCOUNT_TO_LISTEN;

pub struct Validator {
    account_id: AccountId,
    private_key: SecretKey,
    rpc_url: String,
}

impl Validator {
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
impl TransactionProcessor for Validator {
    async fn process_transaction(
        &self,
        event_data: EventLog,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        println!("Validator Processor");
        println!("Validator Event Data: {:?}", event_data);

        let commit_attempts = 30;
        let reveal_attempts = 30;
        let mut committed = false;

        for _attempt in 0..commit_attempts {
            //Get stage to synchronize
            let stage_result = self
                .get_stage(self.rpc_url.clone(), self.account_id.clone(), event_data.clone())
                .await?;
            let stage = stage_result.trim_matches('"').to_string();
            println!("Current Stage: {:?}", stage);

            if stage == "CommitValidators" {
                match self.commit(event_data.clone()).await {
                    Ok(_) => {
                        committed = true;
                        break;
                    }
                    Err(e) => {
                        println!("Failed to commit by validator: {}", e);
                        return Err(e);
                    }
                }
            } else if stage == "RevealValidators" || stage == "Ended" {
                println!("Commit stage passed without committing, skipping transaction.");
                return Ok(false);
            } else {
                println!("Waiting for CommitValidators stage...");
                sleep(Duration::from_secs(10)).await;
            }
        }

        if !committed {
            println!("Failed to reach CommitValidators stage, skipping transaction.");
            return Ok(false);
        }

        for _attempt in 0..reveal_attempts {
            let stage_result = self
                .get_stage(self.rpc_url.clone(), self.account_id.clone(), event_data.clone())
                .await?;
            let stage = stage_result.trim_matches('"').to_string();
            println!("Current Stage: {:?}", stage);

            if stage == "RevealValidators" {
                match self.reveal(event_data.clone()).await {
                    Ok(_) => {
                        return Ok(true);
                    }
                    Err(e) => {
                        println!("Failed to reveal by validator: {}", e);
                        return Err(e);
                    }
                }
            } else if stage == "Ended" {
                println!("RevealValidators stage has ended...");
                return Ok(false);
            } else {
                println!("Waiting for RevealValidators stage...");
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
        println!("Validator Commit");

        let request_id = event_data.data[0]["request_id"]
            .as_str()
            .unwrap_or_default()
            .to_string();

        let qx_sender = NearQxSender::builder(&self.rpc_url)
            .account_receiver(ACCOUNT_TO_LISTEN)
            .method_name("get_list_miners_that_commit_and_reveal")
            .args(json!({"request_id": request_id}))
            .build()?;

        let participant_miners = qx_sender.send_query().await?;

        println!("PARTICIPANT MINERS: {:?}", participant_miners);

        let answer = utils::generate_validator_answer();

        let qx_sender = NearQxSender::builder(&self.rpc_url)
        .account_receiver(ACCOUNT_TO_LISTEN)
        .method_name("hash_validator_answer")
        .args(json!({"validator": self.account_id.to_string(),
                 "request_id": request_id,
                 "answer": answer,
                 "message": "This are the best miners"}))
        .build()?;

        let query_result = qx_sender.send_query().await?;

        let tx_sender = NearTxSender::builder(&self.rpc_url)
            .account_sender(self.account_id.as_str())
            .account_receiver(ACCOUNT_TO_LISTEN)
            .use_private_key(&self.private_key.to_string())
            .method_name("commit_by_validator")
            .args(json!({"request_id": request_id,
                 "answer": query_result}))
            .build()?;

        let tx_response = tx_sender.send_transaction().await?;
        let log_tx = self.extract_logs(&tx_response);

        println!("COMMIT_VALIDATOR_LOG: {:?}", log_tx);

        Ok(())
    }

    async fn reveal(
        &self,
        event_data: EventLog,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("Reveal by validator");

        let request_id = event_data.data[0]["request_id"]
            .as_str()
            .unwrap_or_default()
            .to_string();

        let tx_sender = NearTxSender::builder(&self.rpc_url)
            .account_sender(self.account_id.as_str())
            .account_receiver(ACCOUNT_TO_LISTEN)
            .use_private_key(&self.private_key.to_string())
            .method_name("reveal_by_validator")
            .args(json!({"request_id": request_id,
                 "request_id": request_id,
                 "answer": utils::generate_validator_answer(),
                 "message": "This are the best miners"}))
            .build()?;

        let tx_response = tx_sender.send_transaction().await?;
        let log_tx = self.extract_logs(&tx_response);
        println!("REVEAL_VALIDATOR_LOG: {:?}", log_tx);

        Ok(())
    }
}
