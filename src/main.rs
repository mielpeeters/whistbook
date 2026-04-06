// Re-export lib items so routes.rs can use crate:: paths unchanged
pub use whistbook::{auth, config, db, embed, error, rating, telegram, template, whist};
pub use whistbook::{config as config_fn, config_bytes};
pub use whistbook::Db;

mod routes;

use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let db = Db(Arc::new(db::create_pool().await?));

    let app = routes::router(db.clone()).await;

    let listener = TcpListener::bind(format!("0.0.0.0:{}", whistbook::config("PORT").unwrap())).await?;
    println!("Listening on port {}", whistbook::config("PORT").unwrap());

    println!("Deploying on {}", whistbook::config("DOMAIN").unwrap());
    qr2term::print_qr(whistbook::config("DOMAIN").unwrap()).unwrap();

    axum::serve(listener, app).await?;
    return Ok(());
}
