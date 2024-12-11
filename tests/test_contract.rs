use anyhow::Ok;
use common::utils::calculate_request_id;
use near_workspaces::types::NearToken;
use serde_json::json;
use dotenv::dotenv;
use std::env;

pub mod common;

fn get_wasm_filepath() -> String {
    dotenv().ok();
    env::var("EARTHMIND_WASM_FILEPATH").expect("EARTHMIND_WASM_FILEPATH must be set in .env file")
}

#[tokio::test]
async fn test_earthmind_contract() -> anyhow::Result<()> {
    let wasm_filepath = get_wasm_filepath();
    let worker = near_workspaces::sandbox().await?;
    let wasm = std::fs::read(wasm_filepath)?;

    let contract = worker.dev_deploy(&wasm).await?;

    let outcome = contract.call("new").transact().await?;

    println!("new_contract outcome: {:#?}", outcome);
    assert!(outcome.is_success());

    //@dev Register a protocol
    let protocol_account = worker.dev_create_account().await?;
    println!("protocol acount created: {:?}", protocol_account);

    let registration_fee = NearToken::from_near(10);

    let outcome = protocol_account
                                           .call(contract.id(), "register_protocol")
                                           .args_json(json!({"culture":"Governance decision", "modules":["TextPrompting", "ObjectRecognition"]}))
                                           .deposit(registration_fee)
                                           .transact()
                                           .await?;

    println!("register protocol outcome: {:#?}", outcome);

    let expected = format!(
        r#"EVENT_JSON:{{"standard":"emip001","version":"1.0.0","event":"register_protocol","data":[{{"account":"{}"}}]}}"#,
        protocol_account.id()
    );
    let logs = outcome.logs().join("\n");

    assert_eq!(expected, logs);

    // @dev verify that protocol es registered
    let account = protocol_account.id();
    let result = contract
        .call("is_protocol_registered")
        .args_json(json!({"account":account}))
        .transact()
        .await?;

    println!("is_protocol_registered outcome: {:#?}", result);
    assert!(result.is_success());
    Ok(())
}

#[tokio::test]
async fn test_register_miner() -> anyhow::Result<()> {
    let wasm_filepath = get_wasm_filepath();
    let worker = near_workspaces::sandbox().await?;
    let wasm = std::fs::read(wasm_filepath)?;

    let contract = worker.dev_deploy(&wasm).await?;
    let contract_deploy_outcome = contract.call("new").transact().await?;

    assert!(contract_deploy_outcome.is_success());

    // @dev register miner to earthmind protocol
    let miner_account = worker.dev_create_account().await?;
    let miner_deposit = NearToken::from_near(1);

    let register_miner_outcome = miner_account
        .call(contract.id(), "register_miner")
        .deposit(miner_deposit)
        .transact()
        .await?;

    println!("register_miner_outcome: {:#?}", register_miner_outcome);
    assert!(register_miner_outcome.is_success());

    let expected = format!(
        r#"EVENT_JSON:{{"standard":"emip001","version":"1.0.0","event":"register_miner","data":[{{"miner":"{}"}}]}}"#,
        miner_account.id()
    );
    let logs = register_miner_outcome.logs().join("\n");

    assert_eq!(expected, logs);

    // @dev verify miner is registered
    let miner_account = miner_account.id();
    let is_miner_registered_outcome = contract
        .call("is_miner_registered")
        .args_json(json!({"miner_id":miner_account}))
        .transact()
        .await?;

    assert!(is_miner_registered_outcome.is_success());

    Ok(())
}

#[tokio::test]
async fn test_request_governance_decision() -> anyhow::Result<()> {
    let wasm_filepath = get_wasm_filepath();
    let worker = near_workspaces::sandbox().await?;
    let wasm = std::fs::read(wasm_filepath)?;

    let contract = worker.dev_deploy(&wasm).await?;

    let _contract_deploy_outcome = contract.call("new").transact().await?;

    let protocol_account = worker.dev_create_account().await?;
    let registration_fee = NearToken::from_near(10);

    let _register_protocol_outcome = protocol_account
                                           .call(contract.id(), "register_protocol")
                                           .args_json(json!({"culture":"Governance decision", "modules":["TextPrompting", "ObjectRecognition"]}))
                                           .deposit(registration_fee)
                                           .transact()
                                           .await?;

    let request_governance_decision_outcome = protocol_account
        .call(contract.id(), "request_governance_decision")
        .args_json(json!({"message":"Should we change the rules?"}))
        .transact()
        .await?;

    println!(
        "request_governance_decision_outcome: {:#?}",
        request_governance_decision_outcome
    );
    assert!(request_governance_decision_outcome.is_success());

    let request_id = calculate_request_id(
        protocol_account.id().clone(),
        "Should we change the rules?".to_string(),
    );
    let expected = format!(
        r#"EVENT_JSON:{{"standard":"emip001","version":"1.0.0","event":"register_request","data":[{{"request_id":"{}"}}]}}"#,
        request_id
    );
    let logs = request_governance_decision_outcome.logs().join("\n");
    assert_eq!(expected, logs);

    Ok(())
}

