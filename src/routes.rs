use std::sync::Arc;
use std::time::Duration;

use askama::Template;
use axum::extract::{Path, State};
use axum::http::{StatusCode, Uri};
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, Router};
use axum::Form;
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use axum_garde::WithValidation;
use garde::Validate;
use serde::{Deserialize, Serialize};
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::KeyExtractor;
use tower_governor::{GovernorError, GovernorLayer};
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;
use tower_livereload::LiveReloadLayer;

use crate::auth::{create_token, verify_token};
use crate::db::{
    self, delete_game_by_id, email_exists, get_game, get_game_by_id, get_games_with_ids, save_game,
    set_login, start_game,
};
use crate::embed::StaticFile;
use crate::error::Error;
use crate::template::{
    AlertTemplate, DealFormTemplate, GameTemplate, GamesTemplate, HtmlTemplate, IndexTemplate,
    LoginActions, LoginTemplate, MainTemplate, NewGameTemplate, PointsTemplate, Svg,
};
use crate::whist::{duo_bids, solo_bids, Bid, Deal, Players, Team};
use crate::Db;

macro_rules! auth {
    ($jar:ident, $token:ident, $block:block) => {
        #[allow(unused)]
        if let Some(Ok($token)) = $jar
            .get("token")
            .and_then(|t| Some(verify_token(t.value())))
        {
            $block
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    };

    ($jar:ident, $token:ident, $block:block, $else:block) => {
        #[allow(unused)]
        if let Some(Ok($token)) = $jar
            .get("token")
            .and_then(|t| Some(verify_token(t.value())))
        {
            $block
        } else {
            $else
        }
    };
}

macro_rules! int_err {
    ($res:expr) => {
        $res.map_err(|e| e.into_alert())
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
            .per_second(1)
            .burst_size(20)
            .use_headers()
            .key_extractor(RateLimitToken)
            .error_handler(|_| {
                AlertTemplate {
                    code: StatusCode::TOO_MANY_REQUESTS,
                    alert: "Slow down there".into(),
                }
                .into_response()
            })
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
        .route("/api/logout", get(logout))
        .route("/api/qr", get(user_qr))
        .route("/form/:game_id", get(deal_form))
        .route("/games", get(games))
        .route("/game/:game_id", get(game))
        .route("/game/:game_id", delete(delete_game))
        .route("/api/deal/:game_id", post(deal))
        .route("/new-game", get(new_game_form))
        .route("/api/new-game", post(new_game))
        .route("/api/check-email", post(check_email))
        .route("/public/*file", get(static_handler))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .with_state(app_state);

    if cfg!(debug_assertions) {
        // debug only
        router.layer(LiveReloadLayer::new().reload_interval(Duration::from_millis(2000)))
    } else {
        // release only
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

#[derive(Deserialize, Validate)]
struct Login {
    #[garde(email)]
    email: String,
    #[garde(skip)]
    password: String,
}

async fn login(jar: CookieJar) -> impl IntoResponse {
    auth!(jar, token, { main_page().await }, {
        let login = LoginTemplate {};
        login.render().unwrap().into_response()
    })
}

async fn logout(jar: CookieJar) -> impl IntoResponse {
    (
        [("HX-Redirect", "/")],
        jar.remove(
            Cookie::build("token")
                .path("/")
                .same_site(SameSite::Strict)
                .http_only(true),
        ),
    )
}

async fn register(
    state: State<Db>,
    jar: CookieJar,
    login: Form<Login>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    if let Err(_e) = login.0.validate() {
        return Err(AlertTemplate {
            code: 422.try_into().unwrap(),
            alert: "Not a valid email...".into(),
        });
    }

    // use the index on the login table as a check to see if this login already exists!
    let res = set_login(state.0.clone(), &login.0.email, &login.0.password).await;

    if let Err(Error::LoginErr(e)) = res {
        let message = e.to_string();

        return Err(AlertTemplate {
            code: StatusCode::BAD_REQUEST,
            alert: message,
        });
    }

    check_credentials(state, jar, login).await
}

async fn main_page() -> axum::http::Response<axum::body::Body> {
    HtmlTemplate(MainTemplate {}).into_response()
}

async fn check_credentials(
    State(db): State<Db>,
    jar: CookieJar,
    Form(login): Form<Login>,
) -> Result<impl IntoResponse, AlertTemplate> {
    if login.validate().is_err() {
        return Err(AlertTemplate {
            code: 422.try_into().unwrap(),
            alert: "not a valid email".into(),
        });
    }

    let check = crate::db::check_login(db.clone(), &login.email, &login.password).await?;

    if check {
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
) -> Result<impl IntoResponse, AlertTemplate> {
    auth!(
        jar,
        token,
        {
            let body = urlencoding::decode(&body).unwrap().into_owned();
            let parts = body.split('&').map(|part| {
                let mut key_and_value = part.split('=');
                (key_and_value.next().unwrap(), key_and_value.next().unwrap())
            });

            let mut team = vec![];
            let mut opps = vec![];
            let mut bid = Bid::GrandSlam;
            let mut slagen = 13;

            for (key, value) in parts {
                match key {
                    "team" => team.push(value.to_string()),
                    "bid" => bid = value.into(),
                    "slagen" => slagen = value.parse().unwrap(),
                    "opp" => opps.push(value.to_string()),
                    _ => unreachable!(),
                }
            }

            let mut current_game = get_game(db.clone(), token.user.clone(), game_id.clone())
                .await
                .unwrap();

            // convert team into a usize or (usize, usize)
            let mut indexes: Vec<_> = team
                .iter()
                .map(|player| {
                    current_game
                        .players
                        .clone()
                        .into_iter()
                        .position(|p| p.to_lowercase() == player.to_lowercase())
                        .unwrap()
                })
                .collect();

            let other_indexes: Vec<_> = if opps.is_empty() {
                (0..4)
                    .filter(|i| !indexes.iter().any(|j| *i == *j))
                    .collect()
            } else {
                opps.iter()
                    .map(|player| {
                        current_game
                            .players
                            .clone()
                            .into_iter()
                            .position(|p| p.to_lowercase() == player.to_lowercase())
                            .unwrap()
                    })
                    .collect()
            };

            let team = match indexes.len() {
                1 => {
                    if other_indexes.len() != 3 {
                        return Err(AlertTemplate::bad_request("need three opponents"));
                    }
                    Team::Solo(
                        indexes[0],
                        (other_indexes[0], other_indexes[1], other_indexes[2]),
                    )
                }
                2 => {
                    if other_indexes.len() != 2 {
                        return Err(AlertTemplate::bad_request("need two opponents"));
                    }
                    Team::Duo(
                        (indexes[0], indexes[1]),
                        (other_indexes[0], other_indexes[1]),
                    )
                }
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
            .map_err(|_| AlertTemplate::internal_server_error())?;

            Ok(HtmlTemplate(PointsTemplate {
                id: game_id,
                points,
                players,
                scores,
            }))
        },
        {
            Err(AlertTemplate {
                code: 401.try_into().unwrap(),
                alert: "Unauthorized".into(),
            })
        }
    )
}

pub async fn new_game_form(jar: CookieJar) -> Result<impl IntoResponse, impl IntoResponse> {
    auth!(jar, token, { Ok(HtmlTemplate(NewGameTemplate {})) })
}

#[derive(Deserialize, Validate)]
pub struct NewGameForm {
    #[garde(length(min = 1))]
    name: String,
    #[garde(length(min = 1))]
    player1: String,
    #[garde(length(min = 1))]
    player2: String,
    #[garde(length(min = 1))]
    player3: String,
    #[garde(length(min = 1))]
    player4: String,
    #[garde(alphanumeric)]
    id2: String,
    #[garde(alphanumeric)]
    id3: String,
    #[garde(alphanumeric)]
    id4: String,
    #[garde(skip)]
    player5: String,
    #[garde(skip)]
    player6: String,
    #[garde(skip)]
    player7: String,
    #[garde(alphanumeric)]
    id5: String,
    #[garde(alphanumeric)]
    id6: String,
    #[garde(alphanumeric)]
    id7: String,
}

pub async fn new_game(
    State(db): State<Db>,
    jar: CookieJar,
    WithValidation(form): WithValidation<Form<NewGameForm>>,
) -> Result<impl IntoResponse, AlertTemplate> {
    auth!(
        jar,
        token,
        {
            let form = form.into_inner();

            let owner = token.user;
            let mut players: Players = [&form.player1, &form.player2, &form.player3, &form.player4]
                .as_slice()
                .into();

            // add optional players
            players.opt_add_player(&form.player5);
            players.opt_add_player(&form.player6);
            players.opt_add_player(&form.player7);

            let (id, game) = start_game(db.clone(), form.name, players)
                .await
                .map_err(|e| e.into_alert())?;

            // add all given logins as players of this game
            // first: me myself and I
            let my_id = db::get_user_id(db.clone(), owner)
                .await
                .map_err(|e| e.into_alert())?;

            int_err!(db::add_player(db.clone(), id.clone(), my_id, form.player1).await)?;

            if !form.id2.is_empty() {
                int_err!(db::add_player(db.clone(), id.clone(), form.id2, form.player2).await)?;
            }
            if !form.id3.is_empty() {
                int_err!(db::add_player(db.clone(), id.clone(), form.id3, form.player3).await)?;
            }
            if !form.id4.is_empty() {
                int_err!(db::add_player(db.clone(), id.clone(), form.id4, form.player4).await)?;
            }
            if !form.id5.is_empty() {
                int_err!(db::add_player(db.clone(), id.clone(), form.id5, form.player5).await)?;
            }
            if !form.id6.is_empty() {
                int_err!(db::add_player(db.clone(), id.clone(), form.id6, form.player6).await)?;
            }
            if !form.id7.is_empty() {
                int_err!(db::add_player(db.clone(), id.clone(), form.id7, form.player7).await)?;
            }

            Ok(HtmlTemplate(GameTemplate {
                id,
                game,
                solobids: solo_bids(),
                duobids: duo_bids(),
            }))
        },
        {
            Err(AlertTemplate {
                code: StatusCode::UNAUTHORIZED,
                alert: "unauthorized".into(),
            })
        }
    )
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
    })
}

#[derive(Deserialize)]
pub struct Email {
    email: String,
}

pub async fn check_email(
    State(db): State<Db>,
    Form(email): Form<Email>,
) -> Result<HtmlTemplate<LoginActions>, AlertTemplate> {
    Ok(HtmlTemplate(LoginActions {
        exists: email_exists(db, email.email).await?,
    }))
}

pub async fn user_qr(
    State(db): State<Db>,
    jar: CookieJar,
) -> Result<HtmlTemplate<Svg>, AlertTemplate> {
    auth!(
        jar,
        token,
        {
            let id = db::get_user_id(db, token.user)
                .await
                .map_err(|_| AlertTemplate::internal_server_error())?;

            let svg = qrcode_generator::to_svg_to_string(
                id,
                qrcode_generator::QrCodeEcc::Low,
                256,
                None::<&str>,
            )
            .unwrap();

            Ok(HtmlTemplate(Svg { svg }))
        },
        { Err(AlertTemplate::unauthorized()) }
    )
}
