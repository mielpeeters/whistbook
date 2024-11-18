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

use std::ops::Deref;
use std::sync::Arc;
use surrealdb::engine::any::Any;
use surrealdb::Surreal;
use tokio::net::TcpListener;

pub struct Db(Arc<Surreal<Any>>);

impl axum::extract::FromRef<Db> for () {
    fn from_ref(_: &Db) -> Self {}
}

impl Deref for Db {
    type Target = Arc<Surreal<Any>>;

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

    let db: Db = Db(Arc::new(db::get_db().await?));
    db::init_db(db.clone()).await?;

    let app = routes::router(db.clone()).await;

    let port = std::env::var("PORT").unwrap_or("8080".into());
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    println!("Listening on port {}", port);

    let domain = std::env::var("DOMAIN").unwrap_or("https://whist.mielpeeters.be".into());
    println!("Deploying on {domain}");
    qr2term::print_qr(&domain).unwrap();

    axum::serve(listener, app).await?;
    return Ok(());
}
