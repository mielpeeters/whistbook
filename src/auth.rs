use std::time::UNIX_EPOCH;

use aes_gcm::aead::Aead;
use aes_gcm::{AeadCore, Aes256Gcm, Key, KeyInit};
use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

use crate::error::{Error, LoginErr, TokenError};

const EXPECTED: &str = "This is a signed token for the bonkrank website";
const TOKEN_HOURS: u64 = 2;

#[derive(Serialize, Deserialize)]
pub struct Token {
    message: String,
    pub user: String,
    expires_at: u128,
}

impl Token {
    fn new(message: String, user: String) -> Self {
        let now = std::time::SystemTime::now();
        let expiry = now
            .checked_add(std::time::Duration::from_hours(TOKEN_HOURS))
            .unwrap();
        let expires_at = expiry.duration_since(UNIX_EPOCH).unwrap().as_nanos();

        Token {
            message,
            user,
            expires_at,
        }
    }

    fn is_valid(&self) -> bool {
        if self.message != EXPECTED {
            return false;
        }

        let now = std::time::SystemTime::now();
        let current_unix = now.duration_since(UNIX_EPOCH).unwrap().as_nanos();

        self.expires_at > current_unix
    }
}

pub fn get_env_key(env_var: &str) -> Result<Vec<u8>, Error> {
    let key_b64 =
        std::env::var(env_var).unwrap_or("TvuUto7mf8EHYHzzV/sL25hzjQODnDv/4BXpg0laDfE=".into());
    STANDARD.decode(key_b64).map_err(Error::EnvVarDecodeError)
}

pub fn create_token(user: String) -> Result<String, Error> {
    let key = get_env_key("TOKEN_KEY")?;

    // key needs to be 32 bytes long

    let key = Key::<Aes256Gcm>::from_slice(&key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let token = Token::new(EXPECTED.into(), user);
    let token_json = serde_json::to_vec(&token).unwrap();

    let mut signed_token = cipher
        .encrypt(&nonce, token_json.as_ref())
        .map_err(|_| Error::EncryptError)?;

    signed_token.extend(nonce.iter());

    let stringified = STANDARD.encode(signed_token);
    Ok(stringified)
}

pub fn verify_token(token: &str) -> Result<Token, Error> {
    let token = STANDARD.decode(token).map_err(|_| Error::DecryptError)?;

    let (signed_token, nonce) = token.split_at(token.len() - 12);

    let key = get_env_key("TOKEN_KEY")?;

    // key needs to be 32 bytes long

    let key = Key::<Aes256Gcm>::from_slice(&key);
    let cipher = Aes256Gcm::new(key);

    let token = cipher
        .decrypt(nonce.into(), signed_token)
        .map_err(|_| Error::DecryptError)?;

    let token: Token =
        serde_json::from_slice(&token).map_err(|_| Error::TokenError(TokenError::NotSigned))?;

    if !token.is_valid() {
        return Err(Error::TokenError(TokenError::Expired));
    }

    Ok(token)
}

pub fn check_webhook_secret(secret: &[u8]) -> Result<bool, Error> {
    let key = get_env_key("WEBHOOK_SECRET")?;

    Ok(key == secret)
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

    // Check for at least one uppercase letter
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(LoginErr::MissingUppercase);
    }

    // Check for at least one lowercase letter
    if !password.chars().any(|c| c.is_lowercase()) {
        return Err(LoginErr::MissingLowercase);
    }

    // Check for at least one digit
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(LoginErr::MissingDigit);
    }

    // Check for at least one special character
    let special_chars = "!@#$%^&*()_+-=[]{}|;:,.<>?";
    if !password.chars().any(|c| special_chars.contains(c)) {
        return Err(LoginErr::MissingSpecialChar);
    }

    // If all checks pass
    Ok(())
}
