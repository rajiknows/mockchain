# MockChain: A Flexible Rust Blockchain Implementation

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.54+-orange.svg)](https://www.rust-lang.org/)
[![gRPC](https://img.shields.io/badge/gRPC-Protocol-blue.svg)](https://grpc.io/)
[![Consensus](https://img.shields.io/badge/Consensus-Pluggable-green.svg)]()
[![Status](https://img.shields.io/badge/Status-Experimental-red.svg)]()
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](#contributing)

## Overview

MockChain is a modular blockchain implementation written in Rust that demonstrates fundamental blockchain concepts while providing a practical framework for experimentation. This project features a pluggable consensus mechanism, transaction management, and a gRPC API for external interaction.

## Key Features

### Pluggable Consensus Mechanisms

The blockchain supports different consensus algorithms through a trait-based plugin system:

- **Proof of Work (PoW)**: A hashrate-based consensus where miners compete to solve computational puzzles
- **Proof of Stake (PoS)**: A consensus mechanism that selects validators based on their economic stake

The consensus system is designed to be extensible:

```rust
pub trait Consensus: Send + Sync {
    fn generate_block(&self, index: u64, transactions: Vec<Transaction>, previous_hash: String) -> Block;
    fn validate_block(&self, block: &Block, previous_hash: &str) -> bool;
    fn start(&self, blockchain: Arc<Mutex<Blockchain>>);
    fn name(&self) -> &str;
}
```

### Secure Transactions

Transactions are cryptographically secured using:

- **ECDSA Signatures**: Using the secp256k1 curve (the same as Bitcoin)
- **SHA-256 Hashing**: For transaction and block integrity

Each transaction contains:
- Sender address (public key)
- Recipient address
- Amount
- Timestamp
- Digital signature

### gRPC API Service

The blockchain exposes a gRPC interface for client applications, defined in protobuf:

- `submit_transaction`: Send tokens from one address to another
- `get_balance`: Query an address's current balance
- `request_faucet`: Request test tokens for development

### Block Structure

Each block contains:

- Block index
- Timestamp
- List of transactions
- Previous block's hash
- Current block's hash
- Nonce (used in PoW)
- Miner's address

### Development Features

- **Test Faucet**: Easily obtain test tokens for development
- **Async Runtime**: Built on tokio for concurrent operation
- **Structured Logging**: Comprehensive logging for troubleshooting

## Getting Started

### Prerequisites

- Rust (1.54.0+)
- Protobuf compiler (`protoc`)

### Installation

1. Clone the repository:
   ```
   git clone https://github.com/0xsourav/mockchain.git
   cd mockchain
   ```

2. Build the project:
   ```
   cargo build --release
   ```

### Running the Node

Start a blockchain node with default settings:

```
RUST_LOG=info cargo run --release
```

### Configuration Options

You can choose different consensus mechanisms by modifying the following line in `main.rs`:

```rust
// For Proof of Work with difficulty 3
let consensus_type = ConsensusType::ProofOfWork { difficulty: 3 };

// For Proof of Stake with minimum stake of 1000
// let consensus_type = ConsensusType::ProofOfStake { min_stake: 1000 };
```

## Client Interaction

### Official Wallet Client: Mockallet

[Mockallet](https://github.com/0xsouravm/mockchain-wallet-rs) is the official wallet implementation for this blockchain. It provides a user-friendly way to interact with the MockChain network.

Features of Mockallet:
- Key management (generation and storage)
- Balance checking
- Transaction creation and signing
- Faucet interaction for test tokens

Check out the [Mockallet repository](https://github.com/0xsouravm/mockchain-wallet-rs) for installation and usage instructions.

### Using the gRPC API Directly

The blockchain node exposes a gRPC server on `[::1]:50051` by default.

#### Example: Requesting Test Tokens

```rust
// Example gRPC client code
let mut client = BlockchainServiceClient::connect("http://[::1]:50051").await?;

let request = Request::new(FaucetRequest {
    address: "your_public_key_here".to_string(),
});

let response = client.request_faucet(request).await?;
println!("Response: {:?}", response);
```

#### Example: Submitting a Transaction

```rust
let mut client = BlockchainServiceClient::connect("http://[::1]:50051").await?;

// Create and sign a transaction
let tx = ProtoTransaction {
    from: sender_public_key,
    to: recipient_address,
    amount: 100,
    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    signature: signature_bytes,
};

let request = Request::new(tx);
let response = client.submit_transaction(request).await?;
```

## Architecture

The system is designed with the following components:

- **Block**: Contains transactions and chain metadata
- **Transaction**: Represents a transfer of value with cryptographic proof
- **Blockchain**: Manages the chain state and transaction pool
- **Consensus**: Pluggable algorithms for block creation and validation
- **BlockchainServer**: gRPC service implementation

## Technical Details

### Transaction Verification

Transactions undergo multiple verification steps:
1. Signature verification using the sender's public key
2. Balance check to ensure the sender has sufficient funds
3. Block validation by consensus rules

### Mining Process

For Proof of Work consensus:
1. The miner collects pending transactions from the pool
2. A candidate block is created with these transactions
3. The nonce is incremented until the block hash meets difficulty requirements
4. The valid block is added to the chain
5. The miner receives a reward of 50 tokens

## Ecosystem

### Core Components
- **MockChain**: This blockchain implementation
- **[Mockallet](https://github.com/0xsouravm/mockchain-wallet-rs)**: Official wallet implementation for interacting with the blockchain

## Future Improvements

- Peer-to-peer network communication
- Merkle tree implementation for transaction verification
- Support for smart contracts
- Enhanced wallet integration

## License

[MIT License](LICENSE)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Priority Contribution Areas

1. **gRPC API Enhancements**:
   - Implement a service for retrieving transaction history by address
   - Create an endpoint for querying the full blockchain state
   - Add block explorer functionality via gRPC

2. **Consensus Mechanisms**:
   - Complete the Proof of Stake implementation (currently stubbed)
   - Add educational implementations of other consensus algorithms:
     - Delegated Proof of Stake (DPoS)
     - Practical Byzantine Fault Tolerance (PBFT)
     - Proof of Authority (PoA)
   - Improve documentation on consensus pluggability

3. **Testing and Documentation**:
   - Create comprehensive test suites for different consensus mechanisms
   - Develop educational examples demonstrating blockchain fundamentals
   - Document performance characteristics under different consensus models

When contributing, please follow the existing code style and include appropriate tests for your changes.