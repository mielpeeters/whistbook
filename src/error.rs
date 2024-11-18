use std::fmt::Display;

use http::StatusCode;

use crate::template::AlertTemplate;

#[derive(Debug)]
pub enum Error {
    LoginAlreadyExists(String),
    GameNameExists(String),
    PlayerNameProblem,
    SurrealError(surrealdb::Error),
    EnvVarDecodeError(base64::DecodeError),
    TokenDecodeError,
    TokenError(TokenError),
    EncryptError,
    DecryptError,
    WrongSecret,
    ReqwestError(reqwest::Error),
    NoGameError,
    BadLogin,
    LoginErr(LoginErr),
    Sanitize,
}

#[derive(Debug)]
pub enum TokenError {
    NotSigned,
    Expired,
}

#[derive(Debug)]
pub enum LoginErr {
    TooShort,
    WrongEmail,
    WrongCreds,
}

impl LoginErr {
    pub fn to_help_string(&self) -> String {
        match self {
            LoginErr::TooShort => "minimaal 8 tekens".to_string(),
            LoginErr::WrongEmail => "geen juiste email".to_string(),
            LoginErr::WrongCreds => "fout email of wachtwoord".to_string(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::LoginAlreadyExists(s) => {
                write!(f, "alreadyExists: account with email {s} already exists")
            }
            Error::GameNameExists(s) => {
                write!(f, "spel met naam \"{s}\" bestaat al")
            }
            Error::PlayerNameProblem => {
                write!(f, "geef 4 verschillende namen")
            }
            Error::SurrealError(e) => write!(f, "SurrealError: {e}"),
            Error::TokenDecodeError => write!(f, "Token could not be decoded as base64"),
            Error::TokenError(token_error) => write!(f, "Token had an error: {token_error}"),
            Error::EncryptError => write!(f, "Encryption error"),
            Error::DecryptError => write!(f, "Decryption error"),
            Error::EnvVarDecodeError(decode_error) => {
                write!(f, "Env Var decoding failed: {}", decode_error)
            }
            Error::WrongSecret => write!(f, "Secrets do not match"),
            Error::ReqwestError(e) => write!(f, "ReqwestError: {e}"),
            Error::NoGameError => write!(f, "No game for this owner found"),
            Error::BadLogin => write!(f, "This was a bad login"),
            Error::LoginErr(e) => write!(f, "The password does not fulfil: {}", e.to_help_string()),
            Error::Sanitize => write!(f, "The user input was not valid."),
        }
    }
}

impl std::error::Error for Error {}

impl Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenError::NotSigned => write!(f, "This token was not signed"),
            TokenError::Expired => write!(f, "This token has expired"),
        }
    }
}

impl std::error::Error for TokenError {}

impl Error {
    pub fn into_alert(self) -> AlertTemplate {
        AlertTemplate {
            code: StatusCode::BAD_REQUEST,
            alert: self.to_string(),
        }
    }
}
