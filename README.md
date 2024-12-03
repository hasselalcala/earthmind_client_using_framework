# NEAR Event Listener Client

A robust client implementation for listening to and processing events on the NEAR Protocol blockchain, with support for multiple processing modes including Miner, Validator, and Aggregator roles.

## Overview

This project implements a specialized event listener client for the NEAR blockchain that can operate in different modes to process and respond to specific contract events. It's designed with a focus on reliability, proper transaction management, and efficient blockchain interaction.

## Features

- **Multiple Processing Modes**:
  - Miner: Handles mining-related events and commitments
  - Validator: Processes validation requests and provides verification
  - Aggregator: Aggregates and processes final results
  
- **Robust Transaction Management**:
  - Automatic nonce management
  - Transaction retry mechanisms
  - Proper error handling
  
- **Event Processing**:
  - Synchronized stage-based processing
  - Commit-reveal pattern implementation
  - Automatic stage progression

- **Network Support**:
  - Testnet and Mainnet compatibility
  - Configurable endpoints
  - Environment-specific contract addresses

## Prerequisites

- Rust (latest stable version)
- NEAR CLI
- Account on NEAR (Testnet or Mainnet)
- Full access keys for the account

## Installation

1. Clone the repository:

```
git clone https://github.com/hasselalcala/earthmind_client_using_framework
cd earthmind_client_using_framework
```

2. Build the project

```
cargo build --release
```

## Usage

The client can be run in different modes using command-line arguments:

```
cargo run -- --mode [miner|validator|aggregator] --account-id [your-account.near] --private-key [your-private-key] --network [testnet|mainnet]
```


### Example Commands

Run as a Miner:

```
cargo run -- --mode miner --account-id miner.testnet --private-key "ed25519:..." --network testnet
```

Run as a Validator:

```
cargo run -- --mode validator --account-id validator.testnet --private-key "ed25519:..." --network testnet
```


Run as an Aggregator:

```
cargo run -- --mode aggregator --account-id aggregator.testnet --private-key "ed25519:..." --network testnet
```


## Architecture

The project follows a modular architecture with several key components:

- **Event Listener**: Monitors blockchain events
- **Transaction Processor**: Handles different processing modes
- **Transaction Builder**: Constructs blockchain transactions
- **Query Builder**: Creates blockchain queries
- **Nonce Manager**: Manages transaction nonces
- **Transaction Sender**: Handles transaction submission

## Configuration

Key configuration constants are stored in `src/constants.rs`:

```
NEAR_RPC_TESTNET: RPC endpoint for testnet
NEAR_RPC_MAINNET: RPC endpoint for mainnet
ACCOUNT_TO_LISTEN: Contract account to monitor
FUNCTION_TO_LISTEN: Contract function to watch
```

## Development

### Running Tests

```
cargo test
```


