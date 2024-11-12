use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use serde::Deserialize;

use crate::whist::{Game, Players, Points};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {}

#[derive(Template)]
#[template(path = "main.html")]
pub struct MainTemplate {}

#[derive(Template)]
#[template(path = "deal_form.html", block = "dealForm")]
pub struct DealFormTemplate {
    pub id: String,
    pub game: Game,
    pub solobids: Vec<String>,
    pub duobids: Vec<String>,
}

#[derive(Template)]
#[template(path = "deal_form.html")]
pub struct GameTemplate {
    pub id: String,
    pub game: Game,
    pub solobids: Vec<String>,
    pub duobids: Vec<String>,
}

#[derive(Template)]
#[template(path = "alert.html")]
pub struct AlertTemplate {
    pub code: StatusCode,
    pub alert: String,
}

#[derive(Template)]
#[template(path = "success.html")]
pub struct SuccessTemplate {
    pub message: String,
}

#[derive(Template)]
#[template(path = "points.html")]
pub struct PointsTemplate {
    pub id: String,
    pub points: Points,
    pub players: Players,
    pub scores: Points,
}

#[derive(Template)]
#[template(path = "new_game.html")]
pub struct NewGameTemplate {}

#[derive(Deserialize, Debug)]
pub struct IdGame {
    pub id: String,
    pub game: Game,
}

#[derive(Template)]
#[template(path = "games.html")]
pub struct GamesTemplate {
    pub games: Vec<IdGame>,
}

// Turns askama templates into responses that can be handled by server
pub struct HtmlTemplate<T>(pub T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}

impl IntoResponse for AlertTemplate {
    fn into_response(self) -> Response {
        match self.render() {
            Ok(html) => (self.code, html).into_response(),
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to render template",
            )
                .into_response(),
        }
    }
}
