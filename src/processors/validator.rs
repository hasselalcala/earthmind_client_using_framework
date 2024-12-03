//use crate::block_streamer::extract_logs;
use crate::constants::ACCOUNT_TO_LISTEN;
use near_event_listener::EventLog;

use crate::nonce_manager::NonceManager;
use crate::qx_builder::QueryBuilder;
use crate::qx_sender::QuerySender;
use crate::tx_builder::TxBuilder;
use crate::tx_sender::TxSender;

use async_trait::async_trait;
use near_jsonrpc_client::methods;
use near_primitives::views::TxExecutionStatus;
use near_sdk::AccountId;

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use super::utils;
use super::TransactionProcessor;

pub struct Validator {
    nonce_manager: Arc<NonceManager>,
    tx_builder: Arc<Mutex<TxBuilder>>,
    tx_sender: Arc<TxSender>,
    account_id: AccountId,
}

impl Validator {
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
                .get_stage(self.tx_sender.client.clone(), event_data.clone())
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
                .get_stage(self.tx_sender.client.clone(), event_data.clone())
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

        let query = QueryBuilder::new(ACCOUNT_TO_LISTEN.to_string())
            .with_method_name("get_list_miners_that_commit_and_reveal")
            .with_args(serde_json::json!({
                "request_id": request_id
            }))
            .build();

        let query_sender = QuerySender::new(self.tx_sender.client.clone());
        let participant_miners = query_sender.send_query(query).await?;

        println!("PARTICIPANT MINERS: {:?}", participant_miners);

        let answer = utils::generate_validator_answer();

        let query = QueryBuilder::new(ACCOUNT_TO_LISTEN.to_string())
            .with_method_name("hash_validator_answer")
            .with_args(serde_json::json!({
                "validator": self.account_id.to_string(),
                "request_id": request_id,
                "answer": answer,
                "message": "This are the best miners",
            }))
            .build();

        let query_sender = QuerySender::new(self.tx_sender.client.clone());
        let query_result = query_sender.send_query(query).await?;

        let (nonce, block_hash) = self.nonce_manager.get_nonce_and_tx_hash().await?;

        let mut tx_builder = self.tx_builder.lock().await;

        let (tx, _) = tx_builder
            .with_method_name("commit_by_validator")
            .with_args(serde_json::json!({
                "request_id": request_id,
                "answer": query_result,
            }))
            .build(nonce, block_hash);

        let signer = &tx_builder.signer;

        let request = methods::send_tx::RpcSendTransactionRequest {
            signed_transaction: tx.sign(signer),
            wait_until: TxExecutionStatus::Final,
        };

        let tx_response = self.tx_sender.send_transaction(request).await?;
        let log_tx = self.extract_logs(&tx_response);

        println!("COMMIT_VALIDATOR_LOG: {:?}", log_tx);

        Ok(())
    }

    async fn reveal(
        &self,
        event_data: EventLog,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("Reveal by validator");

        let (nonce, block_hash) = self.nonce_manager.get_nonce_and_tx_hash().await?;

        let mut tx_builder = self.tx_builder.lock().await;

        let request_id = event_data.data[0]["request_id"]
            .as_str()
            .unwrap_or_default()
            .to_string();

        let (tx, _) = tx_builder
            .with_method_name("reveal_by_validator")
            .with_args(serde_json::json!({
                "request_id": request_id,
                "answer": utils::generate_validator_answer(),
                "message": "This are the best miners",
            }))
            .build(nonce, block_hash);

        let signer = &tx_builder.signer;

        let request = methods::send_tx::RpcSendTransactionRequest {
            signed_transaction: tx.sign(signer),
            wait_until: TxExecutionStatus::Final,
        };

        let tx_response = self.tx_sender.send_transaction(request).await?;
        let log_tx = self.extract_logs(&tx_response);
        println!("REVEAL_VALIDATOR_LOG: {:?}", log_tx);

        Ok(())
    }
}
