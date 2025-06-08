use std::collections::HashMap;

use log::info;
use rand::seq::IteratorRandom;
use secp256k1::Secp256k1;

use crate::block::Block;

use super::Consensus;

pub struct ProofOfStake {
    validators: HashMap<String, u64>, // address -> stake
}

impl ProofOfStake {
    pub fn new() -> Self {
        Self {
            validators: HashMap::new(),
        }
    }

    pub fn register_stake(&mut self, validator_address: String, stake: u64) {
        *self.validators.entry(validator_address).or_default() += stake;
    }

    fn select_validator(&self) -> Option<String> {
        let total_stake: u64 = self.validators.values().sum();
        if total_stake == 0 {
            return None;
        }

        let mut rng = rand::thread_rng();
        // more the stake more is the chance of getting selected
        self.validators
            .iter()
            .flat_map(|(addr, stake)| std::iter::repeat(addr.clone()).take(*stake as usize)) // duplicate the validator n times where n is the stake of the validators
            .choose(&mut rng)
    }
}

impl Consensus for ProofOfStake {
    fn name(&self) -> &str {
        "proof of stake"
    }

    fn generate_block(
        &self,
        index: u64,
        transactions: Vec<crate::transaction::Transaction>,
        previous_hash: String,
    ) -> crate::block::Block {
        let miner = self.select_validator().unwrap_or_default();
        let mut block = Block::new(index, transactions, previous_hash);
        block.miner = miner;
        block.hash = block.calculate_hash();
        block
    }

    fn validate_block(&self, block: &crate::block::Block, previous_hash: &str) -> bool {
        if block.previous_hash != previous_hash {
            return false;
        }

        if block.hash != block.calculate_hash() {
            return false;
        }
        true
    }
    fn start(&self, blockchain: std::sync::Arc<std::sync::Mutex<crate::Blockchain>>) {
        tokio::spawn(async move {
            let secp = Secp256k1::new();
            let (_, miner_key) = secp.generate_keypair(&mut rand::thread_rng());
            info!(
                "PoS mining with address: {}",
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
