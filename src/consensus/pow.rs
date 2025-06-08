use std::sync::{Arc, Mutex};

use log::info;
use secp256k1::Secp256k1;

use crate::{block::Block, transaction::Transaction, Blockchain};

use super::Consensus;

// Proof of Work implementation
pub struct ProofOfWork {
    difficulty: usize,
}

impl ProofOfWork {
    pub fn new(difficulty: usize) -> Self {
        Self { difficulty }
    }
}

impl Consensus for ProofOfWork {
    fn name(&self) -> &str {
        "Proof of Work"
    }

    fn generate_block(
        &self,
        index: u64,
        transactions: Vec<Transaction>,
        previous_hash: String,
    ) -> Block {
        let mut block = Block::new(index, transactions, previous_hash);

        let target = "0".repeat(self.difficulty);
        while !block.hash.starts_with(&target) {
            block.nonce += 1;
            block.hash = block.calculate_hash();
        }

        block
    }

    fn validate_block(&self, block: &Block, previous_hash: &str) -> bool {
        if block.previous_hash != previous_hash {
            return false;
        }

        if block.hash != block.calculate_hash() {
            return false;
        }

        block.hash.starts_with(&"0".repeat(self.difficulty))
    }

    fn start(&self, blockchain: Arc<Mutex<Blockchain>>) {
        tokio::spawn(async move {
            let secp = Secp256k1::new();
            let (_, miner_key) = secp.generate_keypair(&mut rand::thread_rng());
            info!(
                "PoW mining with address: {}",
                hex::encode(miner_key.serialize())
            );

            loop {
                {
                    let mut chain = blockchain.lock().unwrap();
                    if chain.transaction_pool.len() > 10 {
                        if let Some(block) = chain.mine_pending_transactions(&miner_key) {
                            info!("Mined block {} with hash {}", block.index, block.hash);
                        }
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            }
        });
    }
}
