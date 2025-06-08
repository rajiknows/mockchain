use std::{
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use log::info;
use secp256k1::Secp256k1;
use tonic::{Request, Response, Status};

use crate::{
    blockchain::{
        blockchain_service_server::BlockchainService, BalanceRequest, BalanceResponse,
        FaucetRequest, FaucetResponse, RpcTransaction, TransactionResponse,
    },
    transaction::Transaction,
    Blockchain, FAUCET_MOCKCHAIN_ADDRESS,
};

pub struct BlockchainServer {
    pub blockchain: Arc<Mutex<Blockchain>>,
}

impl BlockchainServer {
    pub fn new(blockchain: Blockchain) -> Self {
        Self {
            blockchain: Arc::new(Mutex::new(blockchain)),
        }
    }
}

#[tonic::async_trait]
impl BlockchainService for BlockchainServer {
    async fn submit_transaction(
        &self,
        request: Request<RpcTransaction>,
    ) -> Result<Response<TransactionResponse>, Status> {
        let tx = request.into_inner();

        let transaction = Transaction {
            from: tx.from,
            to: tx.to,
            amount: tx.amount,
            timestamp: tx.timestamp,
            signature: tx.signature,
        };

        let mut chain = self.blockchain.lock().unwrap();
        let success = chain.add_transaction(transaction);

        Ok(Response::new(TransactionResponse {
            success,
            message: if success {
                "Transaction accepted".into()
            } else {
                "Transaction failed".into()
            },
        }))
    }

    async fn get_balance(
        &self,
        request: Request<BalanceRequest>,
    ) -> Result<Response<BalanceResponse>, Status> {
        let address = request.into_inner().address;
        let chain = self.blockchain.lock().unwrap();
        let balance = chain.get_balance(&address);

        Ok(Response::new(BalanceResponse { balance }))
    }

    async fn request_faucet(
        &self,
        request: Request<FaucetRequest>,
    ) -> Result<Response<FaucetResponse>, Status> {
        let address = request.into_inner().address;
        info!("Faucet request for address: {}", address);

        // Create a faucet transaction
        let faucet_amount = 1000; // Amount for testing

        // Create a system transaction to fund the account
        let transaction = Transaction {
            from: FAUCET_MOCKCHAIN_ADDRESS.to_string(), // Special faucet address
            to: address.clone(),
            amount: faucet_amount,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs(),
            signature: vec![], // No signature needed for faucet
        };

        let mut chain = self.blockchain.lock().unwrap();
        let success = chain.add_transaction(transaction);

        // Immediately try to mine a block with this transaction
        let secp = Secp256k1::new();
        let (_, faucet_key) = secp.generate_keypair(&mut rand::thread_rng());

        if let Some(block) = chain.mine_pending_transactions(&faucet_key) {
            info!("Created faucet block with hash {}", block.hash);

            Ok(Response::new(FaucetResponse {
                success: true,
                amount: faucet_amount,
                message: "Faucet funds sent successfully".to_string(),
            }))
        } else {
            Ok(Response::new(FaucetResponse {
                success,
                amount: if success { faucet_amount } else { 0 },
                message: if success {
                    "Faucet funds queued for next block".to_string()
                } else {
                    "Failed to process faucet request".to_string()
                },
            }))
        }
    }
}
