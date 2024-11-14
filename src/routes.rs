use std::sync::Arc;
use std::time::Duration;

use askama::Template;
use axum::extract::{Path, State};
use axum::http::{StatusCode, Uri};
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, Router};
use axum::Form;
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde::{Deserialize, Serialize};
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::KeyExtractor;
use tower_governor::{GovernorError, GovernorLayer};
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;
use tower_livereload::LiveReloadLayer;

use crate::auth::{create_token, verify_token};
use crate::db::{
    delete_game_by_id, get_game, get_game_by_id, get_games_with_ids, save_game, set_login,
    start_game,
};
use crate::embed::StaticFile;
use crate::error::Error;
use crate::template::{
    AlertTemplate, DealFormTemplate, GameTemplate, GamesTemplate, HtmlTemplate, IndexTemplate,
    LoginTemplate, MainTemplate, NewGameTemplate, PointsTemplate,
};
use crate::whist::{duo_bids, solo_bids, Bid, Deal, Players, Team};
use crate::Db;

macro_rules! auth {
    ($jar:ident, $token:ident, $block:block) => {
        #[allow(unused)]
        if let Some($token) = $jar.get("token")
            && let Ok($token) = verify_token($token.value())
        {
            $block
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    };

    ($jar:ident, $token:ident, $block:block, $else:block) => {
        #[allow(unused)]
        if let Some($token) = $jar.get("token")
            && let Ok($token) = verify_token($token.value())
        {
            $block
        } else {
            $else
        }
    };
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
struct RateLimitToken;

impl KeyExtractor for RateLimitToken {
    type Key = String;

    fn extract<B>(&self, req: &http::request::Request<B>) -> Result<Self::Key, GovernorError> {
        req.headers()
            .get("x-forwarded-for")
            .and_then(|token| token.to_str().ok())
            .map(|token| token.trim().to_owned())
            .ok_or(GovernorError::Other {
                code: StatusCode::UNAUTHORIZED,
                msg: Some("You don't have permission to access".to_string()),
                headers: None,
            })
    }
}

pub async fn router(app_state: Db) -> Router {
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(3)
            .burst_size(10)
            .key_extractor(RateLimitToken)
            .finish()
            .unwrap(),
    );

    let governor_limiter = governor_conf.limiter().clone();
    let interval = Duration::from_secs(60);
    // a separate background task to clean up
    std::thread::spawn(move || loop {
        std::thread::sleep(interval);
        governor_limiter.retain_recent();
    });

    let router = axum::Router::new()
        .route("/", get(index))
        .route("/login", get(login))
        .route("/register", post(register))
        .route("/api/credentials", post(check_credentials))
        .route("/form/:game_id", get(deal_form))
        .route("/games", get(games))
        .route("/game/:game_id", get(game))
        .route("/game/:game_id", delete(delete_game))
        .route("/api/deal/:game_id", post(deal))
        .route("/new-game", get(new_game_form))
        .route("/api/new-game", post(new_game))
        .route("/public/*file", get(static_handler))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .with_state(app_state);

    if cfg!(debug_assertions) {
        router.layer(LiveReloadLayer::new().reload_interval(Duration::from_millis(2000)))
    } else {
        router.layer(GovernorLayer {
            config: governor_conf,
        })
    }
}

async fn index() -> impl IntoResponse {
    HtmlTemplate(IndexTemplate {})
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    StaticFile(
        uri.path()
            .trim_start_matches("/public")
            .trim_start_matches('/')
            .to_owned(),
    )
}

#[derive(Deserialize)]
struct Login {
    email: String,
    password: String,
}

async fn login(jar: CookieJar) -> impl IntoResponse {
    auth!(jar, token, { main_page().await }, {
        let login = LoginTemplate {};
        login.render().unwrap().into_response()
    })
}

async fn register(
    state: State<Db>,
    jar: CookieJar,
    login: Form<Login>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let res = set_login(state.0.clone(), &login.0.email, &login.0.password).await;

    if let Err(Error::LoginErr(e)) = res {
        let message = e.to_help_string();

        return Err(AlertTemplate {
            code: StatusCode::BAD_REQUEST,
            alert: message,
        });
    }

    check_credentials(state, jar, login).await
}

async fn main_page() -> axum::http::Response<axum::body::Body> {
    MainTemplate {}.render().unwrap().into_response()
}

async fn check_credentials(
    State(db): State<Db>,
    jar: CookieJar,
    Form(login): Form<Login>,
) -> Result<impl IntoResponse, AlertTemplate> {
    let check = crate::db::check_login(db.clone(), &login.email, &login.password).await;

    if let Ok(check) = check
        && check
    {
        let token = create_token(login.email).map_err(|_| AlertTemplate {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            alert: "int serv err".into(),
        })?;

        let cookie = Cookie::build(("token", token))
            .path("/")
            .same_site(SameSite::Strict)
            .http_only(true);

        let main = main_page().await;
        return Ok((jar.add(cookie), main));
    }

    Err(AlertTemplate {
        code: StatusCode::UNAUTHORIZED,
        alert: "Foute gegevens".into(),
    })
}

async fn deal(
    State(db): State<Db>,
    Path(game_id): Path<String>,
    jar: CookieJar,
    body: String,
) -> Result<impl IntoResponse, impl IntoResponse> {
    auth!(jar, token, {
        let body = urlencoding::decode(&body).unwrap().into_owned();
        let parts = body.split('&').map(|part| {
            let mut key_and_value = part.split('=');
            (key_and_value.next().unwrap(), key_and_value.next().unwrap())
        });

        let mut team = vec![];
        let mut bid = Bid::GrandSlam;
        let mut slagen = 13;

        for (key, value) in parts {
            match key {
                "team" => team.push(value.to_string()),
                "bid" => bid = value.into(),
                "slagen" => slagen = value.parse().unwrap(),
                _ => unreachable!(),
            }
        }

        let mut current_game = get_game(db.clone(), token.user.clone(), game_id.clone())
            .await
            .unwrap();

        // convert team into a usize or (usize, usize)
        let mut indexes = team.iter().map(|player| {
            current_game
                .players
                .clone()
                .into_iter()
                .position(|p| p.to_lowercase() == player.to_lowercase())
                .unwrap()
        });

        let team = match indexes.len() {
            1 => Team::Solo(indexes.next().unwrap()),
            2 => Team::Duo(indexes.next().unwrap(), indexes.next().unwrap()),
            _ => unreachable!(),
        };

        current_game.add_deal(Deal {
            team,
            bid,
            achieved: slagen,
        });

        let points = current_game.last_diff().unwrap();
        let players = current_game.players.clone();
        let scores = current_game.scores.last().unwrap().clone();

        save_game(
            db.clone(),
            token.user.clone(),
            game_id.clone(),
            current_game,
        )
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

        Ok(HtmlTemplate(PointsTemplate {
            id: game_id,
            points,
            players,
            scores,
        }))
    })
}

pub async fn new_game_form(jar: CookieJar) -> Result<impl IntoResponse, impl IntoResponse> {
    auth!(jar, token, { Ok(HtmlTemplate(NewGameTemplate {})) })
}

#[derive(Deserialize)]
pub struct NewGameForm {
    name: String,
    player1: String,
    player2: String,
    player3: String,
    player4: String,
}

pub async fn new_game(
    State(db): State<Db>,
    jar: CookieJar,
    Form(form): Form<NewGameForm>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    auth!(jar, token, {
        // TODO: check if all names are different!
        let owner = token.user;
        let players: Players = [form.player1, form.player2, form.player3, form.player4].into();

        let (id, game) = start_game(db, owner, form.name, players)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(HtmlTemplate(GameTemplate {
            id,
            game,
            solobids: solo_bids(),
            duobids: duo_bids(),
        }))
    })
}

pub async fn deal_form(
    State(db): State<Db>,
    Path(game_id): Path<String>,
    jar: CookieJar,
) -> Result<impl IntoResponse, impl IntoResponse> {
    auth!(jar, token, {
        let game = get_game(db, token.user, game_id.clone()).await.unwrap();

        Ok(HtmlTemplate(DealFormTemplate {
            id: game_id,
            game,
            solobids: solo_bids(),
            duobids: duo_bids(),
        }))
    })
}

pub async fn games(
    State(db): State<Db>,
    jar: CookieJar,
) -> Result<impl IntoResponse, impl IntoResponse> {
    auth!(jar, token, {
        let games = get_games_with_ids(db, token.user)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(HtmlTemplate(GamesTemplate { games }))
    })
}

pub async fn game(
    State(db): State<Db>,
    Path(game_id): Path<String>,
    jar: CookieJar,
) -> Result<impl IntoResponse, impl IntoResponse> {
    auth!(jar, token, {
        let game = get_game_by_id(db, token.user, game_id.clone())
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(HtmlTemplate(GameTemplate {
            id: game_id,
            game,
            solobids: solo_bids(),
            duobids: duo_bids(),
        }))
    })
}

pub async fn delete_game(
    State(db): State<Db>,
    Path(game_id): Path<String>,
    jar: CookieJar,
) -> Result<impl IntoResponse, impl IntoResponse> {
    auth!(jar, token, {
        delete_game_by_id(db, token.user, game_id.clone())
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(StatusCode::OK)
}
