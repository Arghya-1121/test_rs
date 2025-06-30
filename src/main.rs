use axum::{
    Router,
    routing::{get, post},
};
mod router;
use crate::router::routes::get_block;
use router::routes::{get_account_info, get_airdrop, get_balance};

#[tokio::main]
async fn main() {
    //setup the router
    let router = Router::new()
        .route("/getBalance/{pubkey}", get(get_balance))
        .route("/getAccountInfo/{pubkey}", get(get_account_info))
        .route("/getBlock/{slot_number}", get(get_block))
        .route("/requestAirdrop", post(get_airdrop));

    // Get port from environment variable (Render provides this)
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let address = format!("0.0.0.0:{}", port);

    let listener = tokio::net::TcpListener::bind(&address).await.unwrap();

    //start the server
    println!("The server starts now at {}", address);
    axum::serve(listener, router).await.unwrap();
}

