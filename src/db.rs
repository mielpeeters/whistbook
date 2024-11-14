use crate::error::{Error, LoginErr};
use crate::template::IdGame;
use crate::whist::{Game, Players};
use crate::{auth, Db};

use surrealdb::engine::any::{self, Any};
use surrealdb::opt::auth::Root;
use surrealdb::opt::Config;
use surrealdb::Surreal;

pub const DB: &str = "whistbook";
pub const NS: &str = "whistbook";

pub async fn get_db() -> Result<Surreal<Any>, surrealdb::Error> {
    let endpoint = std::env::var("DB_ENDPOINT").unwrap_or("ws://localhost:34343".to_owned());

    let root = Root {
        username: "root",
        password: "root",
    };

    let config = Config::new().user(root);

    let db = any::connect((endpoint, config)).await?;

    db.signin(root).await?;

    db.use_ns(NS).use_db(DB).await?;

    Ok(db)
}

pub async fn init_db(_db: Db) -> Result<(), Error> {
    Ok(())
}

pub async fn check_login(db: Db, email: &str, pw: &str) -> Result<bool, Error> {
    let query = r#"
        LET $hash = SELECT VALUE pw FROM ONLY login WHERE email = $email LIMIT 1;
        RETURN crypto::argon2::compare($hash, $pw)
    "#;

    let mut res = db
        .query(query)
        .bind(("email", email.to_string()))
        .bind(("pw", pw.to_string()))
        .await
        .map_err(Error::SurrealError)?;

    let res: Option<bool> = res
        .take(1)
        .map_err(|_| Error::LoginErr(LoginErr::WrongCreds))?;

    Ok(res.unwrap())
}

pub async fn set_login(db: Db, email: &str, pw: &str) -> Result<(), Error> {
    if !auth::check_email(email) {
        return Err(Error::LoginErr(LoginErr::WrongEmail));
    }

    auth::check_pw(pw).map_err(Error::LoginErr)?;

    let query = r#"
        CREATE login
        SET
            email = $email,
            pw = crypto::argon2::generate(<string>$pw)
    "#;

    let res = db
        .query(query)
        .bind(("email", email.to_string()))
        .bind(("pw", pw.to_string()))
        .await
        .map_err(Error::SurrealError)?;

    if let Err(e) = res.check() {
        if let surrealdb::Error::Db(error) = &e
            && let surrealdb::error::Db::IndexExists { .. } = error
        {
            return Err(Error::LoginAlreadyExists(email.to_string()));
        } else {
            return Err(Error::SurrealError(e));
        };
    }

    Ok(())
}

pub async fn start_game<P: Into<Players>>(
    db: Db,
    owner: String,
    name: String,
    players: P,
) -> Result<(String, Game), Error> {
    let game = Game::new(name, players);

    let query = r#"
        SELECT VALUE id.id()
        FROM (CREATE game
         SET 
            owner = $owner,
            game = $game
        );
    "#;

    let mut res = db
        .query(query)
        .bind(("owner", owner))
        .bind(("game", game.clone()))
        .await
        .map_err(Error::SurrealError)?;

    let id: Option<String> = res.take(0).map_err(Error::SurrealError)?;

    Ok((id.unwrap(), game))
}

pub async fn save_game(db: Db, owner: String, id: String, game: Game) -> Result<(), Error> {
    let query = r#"
       UPDATE game
       SET game = $game
       WHERE owner = $owner AND id.id() = $id;
    "#;

    let _ = db
        .query(query)
        .bind(("owner", owner))
        .bind(("game", game))
        .bind(("id", id))
        .await
        .map_err(Error::SurrealError)?;

    Ok(())
}

pub async fn get_game(db: Db, owner: String, id: String) -> Result<Game, Error> {
    let query = r#"
        SELECT VALUE game
        FROM ONLY type::thing(game, $id)
        WHERE owner = $owner
        LIMIT 1;
    "#;

    let mut res = db
        .query(query)
        .bind(("owner", owner))
        .bind(("id", id))
        .await
        .map_err(Error::SurrealError)?;

    let game: Option<Game> = res.take(0).map_err(Error::SurrealError)?;

    game.ok_or(Error::NoGameError)
}

pub async fn get_games_with_ids(db: Db, owner: String) -> Result<Vec<IdGame>, Error> {
    let query = r#"
        SELECT id.id() as id, game
        FROM game
        WHERE owner = $owner;
    "#;

    let mut res = db
        .query(query)
        .bind(("owner", owner))
        .await
        .map_err(Error::SurrealError)?;

    let games: Vec<IdGame> = res.take(0).map_err(Error::SurrealError)?;

    Ok(games)
}

pub async fn get_game_by_id(db: Db, owner: String, id: String) -> Result<Game, Error> {
    let query = r#"
        SELECT VALUE game
        FROM ONLY type::thing(game, $id)
        WHERE owner = $owner
        LIMIT 1;
    "#;

    let mut res = db
        .query(query)
        .bind(("owner", owner))
        .bind(("id", id))
        .await
        .map_err(Error::SurrealError)?;

    let game: Option<Game> = res.take(0).map_err(Error::SurrealError)?;

    Ok(game.unwrap())
}

pub async fn delete_game_by_id(db: Db, owner: String, id: String) -> Result<(), Error> {
    let query = r#"
        DELETE game
        WHERE owner = $owner AND id.id() = $id;
    "#;

    db.query(query)
        .bind(("owner", owner))
        .bind(("id", id))
        .await
        .map_err(Error::SurrealError)?;

    Ok(())
}
