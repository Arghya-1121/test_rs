use axum::{Json, extract::Path};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::account::Account;
use solana_sdk::{
    commitment_config::CommitmentConfig, native_token::LAMPORTS_PER_SOL, pubkey::Pubkey,
};
use solana_transaction_status_client_types::{
    TransactionDetails, UiConfirmedBlock, UiTransactionEncoding,
};
use std::str::FromStr;
use tokio::time::Duration;
use tokio::time::sleep;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum Re {
    Success { balance: f64 },
    Error { error: String },
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum AccountInfo {
    Success(Account),
    Error { error: String },
}

#[derive(serde::Serialize)]
#[serde(untagged)]
pub enum BlockResponse {
    Success(UiConfirmedBlock),
    Error { error: String },
}

pub async fn get_balance(Path(pubkey): Path<String>) -> Json<Re> {
    let client = RpcClient::new_with_commitment(
        "http://127.0.0.1:8899".to_string(),
        CommitmentConfig::confirmed(),
    );
    let pubkey = match Pubkey::from_str(&pubkey) {
        Ok(pk) => pk,
        Err(_) => {
            return Json(Re::Error {
                error: "Wrong Pub key".to_string(),
            });
        }
    };
    // Get balance from RPC
    let balance_result = client.get_balance(&pubkey).await;
    match balance_result {
        Ok(lamports) => {
            let sol_balance = (lamports as f64) / (LAMPORTS_PER_SOL as f64);
            println!("{:#?} SOL", sol_balance);

            Json(Re::Success {
                balance: sol_balance,
            })
        }
        Err(e) => Json(Re::Error {
            error: format!("RPC URL error: {}", e),
        }),
    }
}

pub async fn get_account_info(Path(pubkey): Path<String>) -> Json<AccountInfo> {
    let client = RpcClient::new_with_commitment(
        "http://127.0.0.1:8899".to_string(),
        CommitmentConfig::confirmed(),
    );
    let pubkey = match Pubkey::from_str(&pubkey) {
        Ok(pk) => pk,
        Err(e) => {
            return Json(AccountInfo::Error {
                error: format!("Wrong Pub key {}", e),
            });
        }
    };
    let account = client.get_account(&pubkey).await;
    match account {
        Ok(ac) => Json(AccountInfo::Success(ac)),
        Err(e) => Json(AccountInfo::Error {
            error: format!("RPC Url error: {}", e),
        }),
    }
}

pub async fn get_block(Path(slot_number): Path<u64>) -> Json<BlockResponse> {
    let client = RpcClient::new_with_commitment(
        String::from("https://api.devnet.solana.com"),
        CommitmentConfig::confirmed(),
    );

    let config = solana_client::rpc_config::RpcBlockConfig {
        encoding: Some(UiTransactionEncoding::Base58),
        transaction_details: Some(TransactionDetails::Full),
        rewards: None,
        commitment: Some(CommitmentConfig::finalized()),
        max_supported_transaction_version: Some(0),
    };

    let block = client.get_block_with_config(slot_number, config).await;
    match block {
        Ok(bol) => Json(BlockResponse::Success(bol)),
        Err(e) => Json(BlockResponse::Error {
            error: format!("RPC Error: {}", e),
        }),
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct RequestAirdrop {
    pubkey: String,
    amount: u64,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum Hash {
    Success { hash: String },
    Error { error: String },
}

pub async fn get_airdrop(Json(RA): Json<RequestAirdrop>) -> Json<Hash> {
    let client = RpcClient::new_with_commitment(
        String::from("http://127.0.0.1:8899"),
        CommitmentConfig::confirmed(),
    );
    let receiver = match Pubkey::from_str(&RA.pubkey) {
        Ok(pk) => pk,
        Err(e) => {
            return Json(Hash::Error {
                error: format!("Invalid pubkey: {}", e),
            });
        }
    };
    let lamports = RA.amount * LAMPORTS_PER_SOL;
    let transaction_signature = match client.request_airdrop(&receiver, lamports).await {
        Ok(sig) => sig,
        Err(e) => {
            return Json(Hash::Error {
                error: format!("Airdrop request failed: {}", e),
            });
        }
    };

    loop {
        match client.confirm_transaction(&transaction_signature).await {
            Ok(true) => break,
            Ok(false) => {
                sleep(Duration::from_millis(500)).await;
            }
            Err(e) => {
                return Json(Hash::Error {
                    error: format!("Error confirming transaction: {}", e),
                });
            }
        }
    }

    Json(Hash::Success {
        hash: transaction_signature.to_string(),
    })
}
