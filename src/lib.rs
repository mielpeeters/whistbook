pub mod auth;
pub mod config;
pub mod db;
pub mod embed;
pub mod error;
pub mod rating;
pub mod telegram;
pub mod template;
pub mod whist;

pub use config::{config, config_bytes};

use sqlx::SqlitePool;
use std::ops::Deref;
use std::sync::Arc;

pub struct Db(pub Arc<SqlitePool>);

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
