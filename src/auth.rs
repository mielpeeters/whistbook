use std::time::UNIX_EPOCH;

use aes_gcm::aead::Aead;
use aes_gcm::{AeadCore, Aes256Gcm, Key, KeyInit};
use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

use crate::error::{Error, LoginErr, TokenError};

const ACCESS_MSG: &str = "This is a signed token for the whistbook website";
const REFRESH_MSG: &str = "This is a refresh token for the whistbook website";
const TOKEN_HOURS: u64 = 24;
const REFRESH_TOKEN_DAYS: u64 = 60;

#[derive(Serialize, Deserialize)]
pub struct Token {
    message: String,
    pub user: String,
    expires_at: u128,
}

impl Token {
    fn new(message: String, user: String, duration: std::time::Duration) -> Self {
        let expiry = std::time::SystemTime::now()
            .checked_add(duration)
            .unwrap();
        let expires_at = expiry.duration_since(UNIX_EPOCH).unwrap().as_nanos();
        Token { message, user, expires_at }
    }

    fn is_expired(&self) -> bool {
        let current = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        self.expires_at <= current
    }
}

fn encrypt_token(token: &Token) -> Result<String, Error> {
    let key: Vec<u8> = crate::config_bytes("TOKEN_KEY")?;
    let key = STANDARD.decode(key).map_err(Error::EnvVarDecodeError)?;
    let key = Key::<Aes256Gcm>::from_slice(&key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let token_json = serde_json::to_vec(token).unwrap();
    let mut ciphertext = cipher
        .encrypt(&nonce, token_json.as_ref())
        .map_err(|_| Error::EncryptError)?;
    ciphertext.extend(nonce.iter());

    Ok(STANDARD.encode(ciphertext))
}

fn decrypt_token(token: &str) -> Result<Token, Error> {
    let token = STANDARD.decode(token).map_err(|_| Error::DecryptError)?;
    let (ciphertext, nonce) = token.split_at(token.len() - 12);

    let key: Vec<u8> = crate::config_bytes("TOKEN_KEY")?;
    let key = STANDARD.decode(key).map_err(Error::EnvVarDecodeError)?;
    let key = Key::<Aes256Gcm>::from_slice(&key);
    let cipher = Aes256Gcm::new(key);

    let plaintext = cipher
        .decrypt(nonce.into(), ciphertext)
        .map_err(|_| Error::DecryptError)?;

    serde_json::from_slice(&plaintext).map_err(|_| Error::TokenError(TokenError::NotSigned))
}

pub fn create_token(user: String) -> Result<String, Error> {
    let token = Token::new(
        ACCESS_MSG.into(),
        user,
        std::time::Duration::from_secs(TOKEN_HOURS * 3600),
    );
    encrypt_token(&token)
}

pub fn verify_token(token: &str) -> Result<Token, Error> {
    let token = decrypt_token(token)?;
    if token.message != ACCESS_MSG {
        return Err(Error::TokenError(TokenError::NotSigned));
    }
    if token.is_expired() {
        return Err(Error::TokenError(TokenError::Expired));
    }
    Ok(token)
}

pub fn create_refresh_token(user: String) -> Result<String, Error> {
    let token = Token::new(
        REFRESH_MSG.into(),
        user,
        std::time::Duration::from_secs(REFRESH_TOKEN_DAYS * 24 * 3600),
    );
    encrypt_token(&token)
}

pub fn verify_refresh_token(token: &str) -> Result<Token, Error> {
    let token = decrypt_token(token)?;
    if token.message != REFRESH_MSG {
        return Err(Error::TokenError(TokenError::NotSigned));
    }
    if token.is_expired() {
        return Err(Error::TokenError(TokenError::Expired));
    }
    Ok(token)
}

pub fn check_email(email: &str) -> bool {
    // Split the string by the '@' symbol
    let parts: Vec<&str> = email.split('@').collect();

    // Ensure there are exactly two parts (local and domain)
    if parts.len() != 2 {
        return false;
    }

    let local_part = parts[0];
    let domain_part = parts[1];

    // Ensure both local and domain parts are non-empty
    if local_part.is_empty() || domain_part.is_empty() {
        return false;
    }

    // Ensure the domain part contains at least one period ('.')
    if !domain_part.contains('.') {
        return false;
    }

    // Basic check: ensure local part and domain part don't start with special characters
    if local_part.starts_with('.') || domain_part.starts_with('.') {
        return false;
    }

    // Additional basic checks could be added here, like checking for other invalid characters
    true
}

pub fn check_pw(password: &str) -> Result<(), LoginErr> {
    // Check password length
    if password.len() < 8 {
        return Err(LoginErr::TooShort);
    }

    // If all checks pass
    Ok(())
}
