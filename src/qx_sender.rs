use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use std::error::Error;
use std::sync::Arc;

pub struct QuerySender {
    client: Arc<JsonRpcClient>,
}

impl QuerySender {
    pub fn new(client: Arc<JsonRpcClient>) -> Self {
        Self { client }
    }

    pub async fn send_query(
        &self,
        request: methods::query::RpcQueryRequest,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let response = self.client.call(request).await?;

        if let QueryResponseKind::CallResult(result) = response.kind {
            let result_str = String::from_utf8(result.result)?;
            Ok(result_str)
        } else {
            Err("Unexpected response kind".into())
        }
    }
}
