#![feature(let_chains)]
#![feature(duration_constructors)]
pub mod auth;
mod db;
mod embed;
pub mod error;
mod routes;
pub mod telegram;
mod template;
pub mod whist;

use std::sync::Arc;
use surrealdb::engine::any::Any;
use surrealdb::Surreal;
use tokio::net::TcpListener;

type Db = Arc<Surreal<Any>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let db: Db = Arc::new(db::get_db().await?);
    db::init_db(db.clone()).await?;

    let app = routes::router(Arc::clone(&db)).await;

    let port = std::env::var("PORT").unwrap_or("8080".into());
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    println!("Listening on port {}", port);

    let domain = std::env::var("DOMAIN").unwrap_or("https://whist.mielpeeters.be".into());
    println!("Deploying on {domain}");
    qr2term::print_qr(&domain).unwrap();

    axum::serve(listener, app).await?;
    return Ok(());
}
