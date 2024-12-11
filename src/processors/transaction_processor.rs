use near_jsonrpc_primitives::types::transactions::RpcTransactionResponse;
use near_primitives::views::FinalExecutionOutcomeViewEnum;
use serde_json::json;
use near_sdk::AccountId;

use near_tx_qx_builder::NearQxSender;
use async_trait::async_trait;
use near_event_listener::EventLog;
use crate::constants::ACCOUNT_TO_LISTEN;

#[async_trait]
pub trait TransactionProcessor: Send + Sync {
    async fn process_transaction(
        &self,
        event_data: EventLog,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;

    async fn commit(
        &self,
        event_data: EventLog,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    async fn reveal(
        &self,
        event_data: EventLog,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    // Default methods to all the implementations to synchronize
    async fn get_stage(
        &self,
        rpc_url: String,
        account_id_sender: AccountId,
        event_data: EventLog,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        println!("Get Stage");

        let start_time = event_data.data[0]["start_time"]
            .as_u64()
            .unwrap_or_default();

        let query = NearQxSender::builder(&rpc_url)
            .account_sender(&account_id_sender.as_str())
            .account_receiver(ACCOUNT_TO_LISTEN) 
            .method_name("get_stage")
            .args(json!({"start_time": start_time}))
            .build()?;

        let stage = query.send_query().await?;

        Ok(stage)
    }

    fn extract_logs(&self, response: &RpcTransactionResponse) -> Vec<String> {
        let mut logs = Vec::new();

        if let Some(final_outcome_enum) = &response.final_execution_outcome {
            match final_outcome_enum {
                FinalExecutionOutcomeViewEnum::FinalExecutionOutcome(final_outcome) => {
                    logs.extend(final_outcome.transaction_outcome.outcome.logs.clone());

                    for receipt_outcome in &final_outcome.receipts_outcome {
                        logs.extend(receipt_outcome.outcome.logs.clone());
                    }
                }
                FinalExecutionOutcomeViewEnum::FinalExecutionOutcomeWithReceipt(
                    final_outcome_with_receipt,
                ) => {
                    println!("Non-handled case: {:?}", final_outcome_with_receipt);
                }
            }
        }

        logs
    }
}
