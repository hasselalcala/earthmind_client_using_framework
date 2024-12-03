use near_jsonrpc_client::methods;
use near_primitives::types::{BlockReference, Finality, FunctionArgs};
use near_primitives::views::QueryRequest;
use serde_json::Value;

pub struct QueryBuilder {
    account_id: String,
    method_name: String,
    args: Value,
    block_reference: BlockReference,
}

impl QueryBuilder {
    pub fn new(account_id: String) -> Self {
        Self {
            account_id,
            method_name: String::new(),
            args: Value::Null,
            block_reference: BlockReference::Finality(Finality::Final),
        }
    }

    pub fn with_method_name(&mut self, method_name: &str) -> &mut Self {
        self.method_name = method_name.to_string();
        self
    }

    pub fn with_args(&mut self, args: Value) -> &mut Self {
        self.args = args;
        self
    }

    pub fn build(&self) -> methods::query::RpcQueryRequest {
        methods::query::RpcQueryRequest {
            block_reference: self.block_reference.clone(),
            request: QueryRequest::CallFunction {
                account_id: self.account_id.parse().unwrap(),
                method_name: self.method_name.clone(),
                args: FunctionArgs::from(serde_json::to_string(&self.args).unwrap().into_bytes()),
            },
        }
    }
}
