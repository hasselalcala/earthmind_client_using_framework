use crate::constants::ACCOUNT_TO_LISTEN;
use crate::qx_builder::QueryBuilder;
use crate::qx_sender::QuerySender;

use async_trait::async_trait;
use near_event_listener::EventLog;
use near_jsonrpc_client::JsonRpcClient;
use near_jsonrpc_primitives::types::transactions::RpcTransactionResponse;
use near_primitives::views::FinalExecutionOutcomeViewEnum;
use std::sync::Arc;

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
        tx_sender: Arc<JsonRpcClient>,
        event_data: EventLog,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        println!("Get Stage");

        let start_time = event_data.data[0]["start_time"]
            .as_u64()
            .unwrap_or_default();

        let query = QueryBuilder::new(ACCOUNT_TO_LISTEN.to_string())
            .with_method_name("get_stage")
            .with_args(serde_json::json!({
                "start_time": start_time,
            }))
            .build();

        let query_sender = QuerySender::new(tx_sender);
        let stage = query_sender.send_query(query).await?;

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
