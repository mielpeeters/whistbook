/// One-time migration binary: copies all data from SurrealDB into the new SQLite database.
///
/// Run with the same .env file as the main app, but with DB_ENDPOINT pointing at the
/// live SurrealDB instance and DB_PATH pointing at the destination SQLite file.
///
/// Usage:
///   cargo run --bin migrate
///
/// After a successful run you can remove surrealdb from Cargo.toml and delete this file.

use std::collections::HashMap;

use serde::Deserialize;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::SqlitePool;
use surrealdb::engine::any::{self, Any};
use surrealdb::opt::auth::Root;
use surrealdb::opt::Config;
use surrealdb::Surreal;

const NS: &str = "whistbook";
const DB: &str = "whistbook";

#[derive(Deserialize, Debug)]
struct SurrealLogin {
    id: String,
    email: String,
    pw: String,
}

#[derive(Deserialize, Debug)]
struct SurrealGame {
    id: String,
    game: serde_json::Value,
}

#[derive(Deserialize, Debug)]
struct SurrealPlays {
    login_id: String,
    game_id: String,
    alias: String,
}

fn load_env() {
    // Try .env first, fall back to .env.dev
    for path in &[".env", ".env.dev"] {
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, val)) = line.split_once('=') {
                    std::env::set_var(key.trim(), val.trim());
                }
            }
            break;
        }
    }
}

fn env(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("missing env var: {key}"))
}

/// Extract the raw ID part from a SurrealDB record reference string.
/// e.g. "login:abc123" -> "abc123", "game:xyz789" -> "xyz789"
fn surreal_id(record_ref: &str) -> &str {
    record_ref
        .split_once(':')
        .map(|(_, id)| id)
        .unwrap_or(record_ref)
}

async fn connect_surreal() -> Surreal<Any> {
    let endpoint = env("DB_ENDPOINT");
    let root = Root {
        username: "root",
        password: "root",
    };
    let config = Config::new().user(root);
    let db = any::connect((endpoint, config))
        .await
        .expect("failed to connect to SurrealDB");
    db.signin(root).await.expect("SurrealDB signin failed");
    db.use_ns(NS).use_db(DB).await.expect("use_ns/use_db failed");
    db
}

async fn connect_sqlite() -> SqlitePool {
    let path = env("DB_PATH");
    let opts = SqliteConnectOptions::new()
        .filename(&path)
        .create_if_missing(true)
        .foreign_keys(true);
    let pool = SqlitePool::connect_with(opts)
        .await
        .expect("failed to connect to SQLite");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("SQLite migration failed");
    pool
}

#[tokio::main]
async fn main() {
    load_env();

    println!("Connecting to SurrealDB...");
    let surreal = connect_surreal().await;

    println!("Connecting to SQLite...");
    let sqlite = connect_sqlite().await;

    // --- Fetch from SurrealDB ---

    let logins: Vec<SurrealLogin> = surreal
        .query("SELECT id.id() as id, email, pw FROM login")
        .await
        .expect("query logins failed")
        .take(0)
        .expect("take logins failed");

    let games: Vec<SurrealGame> = surreal
        .query("SELECT id.id() as id, game FROM game")
        .await
        .expect("query games failed")
        .take(0)
        .expect("take games failed");

    let plays: Vec<SurrealPlays> = surreal
        .query("SELECT record::id(in) as login_id, record::id(out) as game_id, alias FROM plays")
        .await
        .expect("query plays failed")
        .take(0)
        .expect("take plays failed");

    println!(
        "Found: {} logins, {} games, {} plays",
        logins.len(),
        games.len(),
        plays.len()
    );

    // --- Migrate into SQLite (single transaction) ---

    let mut tx = sqlite.begin().await.expect("begin transaction failed");

    // login_id_map: surreal string id -> sqlite i64
    let mut login_id_map: HashMap<String, i64> = HashMap::new();
    for login in &logins {
        let result = sqlx::query("INSERT INTO login (email, pw) VALUES (?, ?)")
            .bind(&login.email)
            .bind(&login.pw)
            .execute(&mut *tx)
            .await
            .unwrap_or_else(|e| panic!("insert login {} failed: {e}", login.email));
        login_id_map.insert(login.id.clone(), result.last_insert_rowid());
    }

    // game_id_map: surreal string id -> sqlite i64
    let mut game_id_map: HashMap<String, i64> = HashMap::new();
    for game in &games {
        let json = serde_json::to_string(&game.game).expect("re-serialize game failed");
        let result = sqlx::query("INSERT INTO game (game) VALUES (?)")
            .bind(&json)
            .execute(&mut *tx)
            .await
            .unwrap_or_else(|e| panic!("insert game {} failed: {e}", game.id));
        game_id_map.insert(game.id.clone(), result.last_insert_rowid());
    }

    let mut plays_count = 0usize;
    for play in &plays {
        let login_id = login_id_map
            .get(&play.login_id)
            .copied()
            .unwrap_or_else(|| panic!("no sqlite login id for surreal id '{}'", play.login_id));

        let game_id = game_id_map
            .get(&play.game_id)
            .copied()
            .unwrap_or_else(|| panic!("no sqlite game id for surreal id '{}'", play.game_id));

        sqlx::query("INSERT OR IGNORE INTO plays (login_id, game_id, alias) VALUES (?, ?, ?)")
            .bind(login_id)
            .bind(game_id)
            .bind(&play.alias)
            .execute(&mut *tx)
            .await
            .unwrap_or_else(|e| panic!("insert plays failed: {e}"));

        plays_count += 1;
    }

    tx.commit().await.expect("commit failed");

    println!(
        "Migration complete: {} logins, {} games, {} plays",
        logins.len(),
        games.len(),
        plays_count
    );
}
