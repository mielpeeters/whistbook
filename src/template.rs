use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use serde::Deserialize;

use crate::error::Error;
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
#[template(path = "deal_form.html")]
pub struct DealFormTemplate {
    pub id: String,
    pub game: Game,
    pub solobids: Vec<String>,
    pub duobids: Vec<String>,
}

#[derive(Template)]
#[template(path = "game.html")]
pub struct GameTemplate {
    pub id: String,
    pub game: Game,
    pub solobids: Vec<String>,
    pub duobids: Vec<String>,
}

#[derive(Template, Clone)]
#[template(path = "alert.html")]
pub struct AlertTemplate {
    pub code: StatusCode,
    pub alert: String,
}

impl AlertTemplate {
    pub fn internal_server_error() -> Self {
        AlertTemplate {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            alert: StatusCode::INTERNAL_SERVER_ERROR.to_string(),
        }
    }

    pub fn bad_request(message: &str) -> Self {
        AlertTemplate {
            code: StatusCode::BAD_REQUEST,
            alert: message.to_string(),
        }
    }

    pub fn unauthorized() -> Self {
        AlertTemplate {
            code: StatusCode::UNAUTHORIZED,
            alert: StatusCode::UNAUTHORIZED.to_string(),
        }
    }
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

#[derive(Template)]
#[template(path = "login_actions.html")]
pub struct LoginActions {
    pub exists: bool,
}

#[derive(Template)]
#[template(path = "svg.html")]
pub struct Svg {
    pub svg: String,
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
            Ok(html) => (self.code, Html(html)).into_response(),
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to render template",
            )
                .into_response(),
        }
    }
}

impl From<StatusCode> for AlertTemplate {
    fn from(value: StatusCode) -> Self {
        Self {
            code: value,
            alert: value.to_string(),
        }
    }
}

impl From<Error> for AlertTemplate {
    fn from(value: Error) -> Self {
        AlertTemplate {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            alert: value.to_string(),
        }
    }
}
