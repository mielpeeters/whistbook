use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    BotAlreadyExists(String),
    SurrealError(surrealdb::Error),
    EnvVarDecodeError(base64::DecodeError),
    TokenDecodeError,
    TokenError(TokenError),
    EncryptError,
    DecryptError,
    WrongSecret,
    ReqwestError(reqwest::Error),
    NoGameError,
}

#[derive(Debug)]
pub enum TokenError {
    NotSigned,
    Expired,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BotAlreadyExists(s) => write!(f, "AlreadyExists: bot {s} already exists"),
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
