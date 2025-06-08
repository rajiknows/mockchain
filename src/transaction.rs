use std::time::{SystemTime, UNIX_EPOCH};

use log::warn;
use secp256k1::{PublicKey, Secp256k1};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::FAUCET_MOCKCHAIN_ADDRESS;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub timestamp: u64,
    pub signature: Vec<u8>,
}

impl Transaction {
    pub fn new(from: &str, to: &str, amount: u64) -> Self {
        Self {
            from: from.to_string(),
            to: to.to_string(),
            amount,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs(),
            signature: Vec::new(),
        }
    }

    pub fn get_message_to_sign(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(
            serde_json::to_string(&(&self.from, &self.to, self.amount, self.timestamp))
                .unwrap()
                .as_bytes(),
        );
        hasher.finalize().to_vec()
    }

    pub fn verify(&self) -> bool {
        // Skip verification for faucet transactions
        if self.from == FAUCET_MOCKCHAIN_ADDRESS {
            return true;
        }

        let secp = Secp256k1::new();

        let public_key_bytes = match hex::decode(&self.from) {
            Ok(bytes) => bytes,
            Err(e) => {
                warn!("Failed to decode public key: {}", e);
                return false;
            }
        };

        let public_key = match PublicKey::from_slice(&public_key_bytes) {
            Ok(key) => key,
            Err(e) => {
                warn!("Invalid public key: {}", e);
                return false;
            }
        };

        if let Ok(sig) = secp256k1::ecdsa::Signature::from_compact(&self.signature) {
            let message = self.get_message_to_sign();
            if let Ok(msg) = secp256k1::Message::from_slice(&message) {
                return secp.verify_ecdsa(&msg, &sig, &public_key).is_ok();
            }
        }
        false
    }
}
