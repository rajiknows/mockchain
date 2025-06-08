use std::sync::{Arc, Mutex};

use pow::ProofOfWork;

use crate::{block::Block, transaction::Transaction, Blockchain};

mod pow;
// Consensus trait defines how blocks are produced and validated
pub trait Consensus: Send + Sync {
    fn generate_block(
        &self,
        index: u64,
        transactions: Vec<Transaction>,
        previous_hash: String,
    ) -> Block;
    fn validate_block(&self, block: &Block, previous_hash: &str) -> bool;
    fn start(&self, blockchain: Arc<Mutex<Blockchain>>);
    fn name(&self) -> &str;
}

// Available consensus types
#[derive(Debug)]
pub enum ConsensusType {
    ProofOfWorkType { difficulty: usize },
    ProofOfStakeType { min_stake: u64 },
}

impl ConsensusType {
    pub fn create_consensus(&self) -> Box<dyn Consensus> {
        match self {
            ConsensusType::ProofOfWorkType { difficulty } => {
                Box::new(ProofOfWork::new(*difficulty))
            }
            ConsensusType::ProofOfStakeType { min_stake: _ } => todo!(),
        }
    }
}
