pub mod auth;
pub mod config;
mod db;
mod embed;
pub mod error;
mod routes;
pub mod telegram;
mod template;
pub mod whist;

pub use config::{config, config_bytes};

use sqlx::SqlitePool;
use std::ops::Deref;
use std::sync::Arc;
use tokio::net::TcpListener;

pub struct Db(Arc<SqlitePool>);

impl axum::extract::FromRef<Db> for () {
    fn from_ref(_: &Db) -> Self {}
}

impl Deref for Db {
    type Target = Arc<SqlitePool>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Clone for Db {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let db: Db = Db(Arc::new(db::create_pool().await?));

    let app = routes::router(db.clone()).await;

    let listener = TcpListener::bind(format!("0.0.0.0:{}", config("PORT").unwrap())).await?;
    println!("Listening on port {}", config("PORT").unwrap());

    println!("Deploying on {}", config("DOMAIN").unwrap());
    qr2term::print_qr(config("DOMAIN").unwrap()).unwrap();

    axum::serve(listener, app).await?;
    return Ok(());
}
