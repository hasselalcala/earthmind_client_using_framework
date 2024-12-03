set dotenv-load

run_miner:
    rm -rf data
    cargo run -- --mode miner --account-id "$MINER_ACCOUNT" --private-key "$MINER_SECRET_KEY" --network testnet

run_validator:
    rm -rf data
    cargo run -- --mode validator --account-id "$VALIDATOR_ACCOUNT" --private-key "$VALIDATOR_SECRET_KEY" --network testnet

run_aggregator:
    rm -rf data
    cargo run -- --mode aggregator --account-id "$AGGREGATOR_ACCOUNT" --private-key "$AGGREGATOR_SECRET_KEY" --network testnet
