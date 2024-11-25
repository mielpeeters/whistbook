use std::fmt::Display;

use http::StatusCode;

use crate::template::AlertTemplate;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("alreadyExists: account with email {0} already exists")]
    LoginAlreadyExists(String),
    #[error("spel met naam \"{0}\" bestaat al")]
    GameNameExists(String),
    #[error("geef 4 verschillende namen")]
    PlayerNameProblem,
    #[error("SurrealError: {0}")]
    SurrealError(#[from] surrealdb::Error),
    #[error("Env Var decoding failed: {0}")]
    EnvVarDecodeError(base64::DecodeError),
    #[error("Please set the {0} env variable in .env or .env.dev")]
    EnvVar(String),
    #[error("Token could not be decoded as base64")]
    TokenDecodeError,
    #[error("Token had an error: {0}")]
    TokenError(TokenError),
    #[error("Encryption error")]
    EncryptError,
    #[error("Decryption error")]
    DecryptError,
    #[error("Secrets do not match")]
    WrongSecret,
    #[error("Reqwest error: {0}")]
    ReqwestError(reqwest::Error),
    #[error("No gam efor this owner has been found")]
    NoGameError,
    #[error("This was a bad login")]
    BadLogin,
    #[error("The password does not fulfil: {0}")]
    LoginErr(LoginErr),
    #[error("The user input was not valid.")]
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

impl Display for LoginErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoginErr::TooShort => write!(f, "minimaal 8 tekens"),
            LoginErr::WrongEmail => write!(f, "geen juiste email"),
            LoginErr::WrongCreds => write!(f, "fout email of wachtwoord"),
        }
    }
}

impl Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenError::NotSigned => write!(f, "This token was not signed"),
            TokenError::Expired => write!(f, "This token has expired"),
        }
    }
}

impl Error {
    pub fn into_alert(self) -> AlertTemplate {
        AlertTemplate {
            code: StatusCode::BAD_REQUEST,
            alert: self.to_string(),
        }
    }
}
