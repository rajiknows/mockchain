use blockchain::blockchain_service_server::BlockchainServiceServer;
use log::{info, warn};
use secp256k1::PublicKey;
use std::collections::VecDeque;
use std::sync::Arc;
use tonic::transport::Server;

pub mod blockchain {
    tonic::include_proto!("blockchain");
}

use rpc::BlockchainServer;

mod block;
mod consensus;
mod rpc;
mod transaction;

use block::Block;
use consensus::{Consensus, ConsensusType};
use transaction::Transaction;
const FAUCET_MOCKCHAIN_ADDRESS: &str = "FAUCET_MOCKCHAIN_ADDRESS";

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub transaction_pool: VecDeque<Transaction>,
    consensus: Box<dyn Consensus>,
}

impl Blockchain {
    pub fn new(consensus: Box<dyn Consensus>) -> Self {
        let genesis_block = consensus.generate_block(0, Vec::new(), String::from("0"));
        info!(
            "Creating new blockchain with {} consensus",
            consensus.name()
        );

        Self {
            chain: vec![genesis_block],
            transaction_pool: VecDeque::new(),
            consensus,
        }
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> bool {
        // Allow transactions from the faucet without verification
        if transaction.from == FAUCET_MOCKCHAIN_ADDRESS {
            info!(
                "Adding faucet transaction to pool: FAUCET -> {}, amount: {}",
                transaction.to, transaction.amount
            );
            self.transaction_pool.push_back(transaction);
            return true;
        }

        if !transaction.verify() {
            warn!("Transaction verification failed");
            return false;
        }

        if !self.check_balance(&transaction.from, transaction.amount) {
            warn!("Insufficient balance for transaction");
            return false;
        }

        info!(
            "Adding transaction to pool: {} -> {}, amount: {}",
            transaction.from, transaction.to, transaction.amount
        );
        self.transaction_pool.push_back(transaction);
        true
    }

    pub fn mine_pending_transactions(&mut self, miner_key: &PublicKey) -> Option<Block> {
        if self.transaction_pool.is_empty() {
            return None;
        }

        let transactions: Vec<Transaction> = self.transaction_pool.drain(..).collect();
        let previous_block = self.chain.last()?;

        let mut block = self.consensus.generate_block(
            previous_block.index + 1,
            transactions,
            previous_block.hash.clone(),
        );

        block.miner = hex::encode(miner_key.serialize());
        self.chain.push(block.clone());
        Some(block)
    }

    pub fn get_balance(&self, address: &str) -> u64 {
        let mut balance = 0;
        for block in &self.chain {
            for tx in &block.transactions {
                if tx.to == address {
                    balance += tx.amount;
                }
                if tx.from == address {
                    balance = balance.saturating_sub(tx.amount);
                }
            }
            if block.miner == address {
                balance += 50; // Mining reward
            }
        }
        balance
    }

    pub fn check_balance(&self, address: &str, amount: u64) -> bool {
        let balance = self.get_balance(address);
        balance >= amount
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    // Choose consensus mechanism (could come from args/config)
    let consensus_type = ConsensusType::ProofOfWorkType { difficulty: 3 };
    let consensus = consensus_type.create_consensus();

    info!("Blockchain node starting...");
    let blockchain = Blockchain::new(consensus);
    let server = BlockchainServer::new(blockchain);

    // Start consensus mechanism
    server
        .blockchain
        .lock()
        .unwrap()
        .consensus
        .start(Arc::clone(&server.blockchain));

    let addr = "[::1]:50051".parse()?;
    info!("Starting gRPC server on {}", addr);

    Server::builder()
        .add_service(BlockchainServiceServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}
