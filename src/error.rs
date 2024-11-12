use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    LoginAlreadyExists(String),
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
}

#[derive(Debug)]
pub enum TokenError {
    NotSigned,
    Expired,
}

#[derive(Debug)]
pub enum LoginErr {
    TooShort,
    MissingUppercase,
    MissingLowercase,
    MissingDigit,
    MissingSpecialChar,
    WrongEmail,
    WrongCreds,
}

impl LoginErr {
    pub fn to_help_string(&self) -> String {
        match self {
            LoginErr::TooShort => "Minimaal 8 tekens.".to_string(),
            LoginErr::MissingUppercase => "Bevat een hoofdletter.".to_string(),
            LoginErr::MissingLowercase => "Bevat een kleine letter.".to_string(),
            LoginErr::MissingDigit => "Bevat een cijfer.".to_string(),
            LoginErr::MissingSpecialChar => "Bevat een speciaal teken.".to_string(),
            LoginErr::WrongEmail => "Geen email".to_string(),
            LoginErr::WrongCreds => "Fout email of wachtwrood".to_string(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::LoginAlreadyExists(s) => {
                write!(f, "AlreadyExists: account with email {s} already exists")
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
