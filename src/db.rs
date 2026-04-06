use crate::error::{Error, LoginErr};
use crate::template::{IdGame, LinkedPlayer};
use crate::whist::{Game, Players};
use crate::{auth, Db};

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use password_hash::{rand_core::OsRng, SaltString};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Row, SqlitePool};

pub async fn create_pool() -> Result<SqlitePool, Error> {
    let path = crate::config("DB_PATH")?;
    let opts = SqliteConnectOptions::new()
        .filename(&path)
        .create_if_missing(true)
        .foreign_keys(true);
    let pool = SqlitePool::connect_with(opts).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

pub async fn check_login(db: Db, email: &str, pw: &str) -> Result<bool, Error> {
    let row = sqlx::query("SELECT pw FROM login WHERE email = ?")
        .bind(email)
        .fetch_optional(&**db)
        .await?;

    match row {
        None => Ok(false),
        Some(r) => {
            let pw_hash: String = r.try_get("pw")?;
            let parsed = PasswordHash::new(&pw_hash)
                .map_err(|_| Error::LoginErr(LoginErr::WrongCreds))?;
            Ok(Argon2::default()
                .verify_password(pw.as_bytes(), &parsed)
                .is_ok())
        }
    }
}

pub async fn set_login(db: Db, email: &str, pw: &str) -> Result<(), Error> {
    auth::check_pw(pw).map_err(Error::LoginErr)?;

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(pw.as_bytes(), &salt)
        .map_err(|_| Error::LoginErr(LoginErr::WrongCreds))?
        .to_string();

    sqlx::query("INSERT INTO login (email, pw) VALUES (?, ?)")
        .bind(email)
        .bind(&hash)
        .execute(&**db)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref de) = e {
                if de.is_unique_violation() {
                    return Error::LoginAlreadyExists(email.to_string());
                }
            }
            Error::SqlxError(e)
        })?;

    Ok(())
}

pub async fn email_exists(db: Db, email: String) -> Result<bool, Error> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM login WHERE email = ?")
        .bind(&email)
        .fetch_one(&**db)
        .await?;
    Ok(count > 0)
}

pub async fn get_user_id(db: Db, email: String) -> Result<String, Error> {
    let id: i64 = sqlx::query_scalar("SELECT id FROM login WHERE email = ?")
        .bind(&email)
        .fetch_one(&**db)
        .await?;
    Ok(id.to_string())
}

pub async fn start_game<P: Into<Players>>(
    db: Db,
    name: String,
    players: P,
) -> Result<(String, Game), Error> {
    let game = Game::new(name, players);
    let json = serde_json::to_string(&game).unwrap();

    let result = sqlx::query("INSERT INTO game (game) VALUES (?)")
        .bind(&json)
        .execute(&**db)
        .await?;

    Ok((result.last_insert_rowid().to_string(), game))
}

pub async fn save_game(db: Db, owner: String, id: String, game: Game) -> Result<(), Error> {
    let game_id: i64 = id.parse().map_err(|_| Error::NoGameError)?;
    let json = serde_json::to_string(&game).unwrap();

    sqlx::query(
        "UPDATE game SET game = ?
         WHERE id = ?
           AND EXISTS (
               SELECT 1 FROM plays p
               JOIN login l ON l.id = p.login_id
               WHERE p.game_id = game.id AND l.email = ?
           )",
    )
    .bind(&json)
    .bind(game_id)
    .bind(&owner)
    .execute(&**db)
    .await?;

    Ok(())
}

pub async fn get_game(db: Db, owner: String, id: String) -> Result<Game, Error> {
    let game_id: i64 = id.parse().map_err(|_| Error::NoGameError)?;

    let row = sqlx::query(
        "SELECT g.game FROM game g
         JOIN plays p ON p.game_id = g.id
         JOIN login l ON l.id = p.login_id
         WHERE g.id = ? AND l.email = ?",
    )
    .bind(game_id)
    .bind(&owner)
    .fetch_optional(&**db)
    .await?
    .ok_or(Error::NoGameError)?;

    let json: String = row.try_get("game")?;
    serde_json::from_str(&json).map_err(|_| Error::NoGameError)
}

pub async fn get_game_by_id(db: Db, owner: String, id: String) -> Result<Game, Error> {
    get_game(db, owner, id).await
}

pub async fn get_games_with_ids(db: Db, owner: String) -> Result<Vec<IdGame>, Error> {
    let rows = sqlx::query(
        "SELECT g.id, g.game FROM game g
         JOIN plays p ON p.game_id = g.id
         JOIN login l ON l.id = p.login_id
         WHERE l.email = ?",
    )
    .bind(&owner)
    .fetch_all(&**db)
    .await?;

    rows.into_iter()
        .map(|r| {
            let id: i64 = r.try_get("id")?;
            let json: String = r.try_get("game")?;
            let game: Game = serde_json::from_str(&json).map_err(|_| {
                sqlx::Error::Decode("failed to deserialize game JSON".into())
            })?;
            Ok(IdGame {
                id: id.to_string(),
                game,
            })
        })
        .collect()
}

pub async fn num_players(db: Db, game_id: String) -> Result<usize, Error> {
    let gid: i64 = game_id.parse().map_err(|_| Error::NoGameError)?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM plays WHERE game_id = ?")
        .bind(gid)
        .fetch_one(&**db)
        .await?;
    Ok(count as usize)
}

pub async fn delete_game_by_id(db: Db, owner: String, id: String) -> Result<(), Error> {
    let game_id: i64 = id.parse().map_err(|_| Error::NoGameError)?;

    sqlx::query(
        "DELETE FROM game WHERE id = ?
         AND EXISTS (
             SELECT 1 FROM plays p
             JOIN login l ON l.id = p.login_id
             WHERE p.game_id = game.id AND l.email = ?
         )",
    )
    .bind(game_id)
    .bind(&owner)
    .execute(&**db)
    .await?;

    Ok(())
}

pub async fn add_player(
    db: Db,
    game_id: String,
    user_id: String,
    user_alias: String,
) -> Result<(), Error> {
    let gid: i64 = game_id.parse().map_err(|_| Error::NoGameError)?;
    let uid: i64 = user_id.parse().map_err(|_| Error::NoGameError)?;

    sqlx::query("INSERT OR IGNORE INTO plays (login_id, game_id, alias) VALUES (?, ?, ?)")
        .bind(uid)
        .bind(gid)
        .bind(&user_alias)
        .execute(&**db)
        .await?;

    Ok(())
}

pub async fn remove_player(db: Db, game_id: String, user_id: String) -> Result<(), Error> {
    let gid: i64 = game_id.parse().map_err(|_| Error::NoGameError)?;
    let uid: i64 = user_id.parse().map_err(|_| Error::NoGameError)?;

    sqlx::query("DELETE FROM plays WHERE game_id = ? AND login_id = ?")
        .bind(gid)
        .bind(uid)
        .execute(&**db)
        .await?;

    Ok(())
}

pub async fn get_game_players(db: Db, game_id: String) -> Result<Vec<LinkedPlayer>, Error> {
    let gid: i64 = game_id.parse().map_err(|_| Error::NoGameError)?;

    let rows = sqlx::query(
        "SELECT p.alias, l.email FROM plays p
         JOIN login l ON l.id = p.login_id
         WHERE p.game_id = ?",
    )
    .bind(gid)
    .fetch_all(&**db)
    .await?;

    rows.into_iter()
        .map(|r| {
            Ok(LinkedPlayer {
                alias: r.try_get("alias")?,
                email: r.try_get("email")?,
            })
        })
        .collect()
}
