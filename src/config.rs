use std::collections::HashMap;
use std::sync::OnceLock;

static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct Config {
    pub port: String,
    pub domain: String,
    pub db_endpoint: String,
    pub session_token_key: String,
    pub telegram_user_id: String,
    pub telegram_bot_key: String,
}

pub fn config() -> &'static Config {
    CONFIG.get_or_init(Config::load)
}

fn parse_env_file(file: &str) -> HashMap<String, String> {
    let mut env_map = HashMap::new();

    for line in file.lines() {
        // Trim whitespace and skip empty lines or comments
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Split the line into key and value
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().to_string();
            env_map.insert(key, value);
        }
    }

    env_map
}

impl Config {
    pub fn load() -> Self {
        // read the .env file
        let env_file = std::fs::read_to_string(".env").expect("make a .env file at root");

        let envs = parse_env_file(&env_file);

        let default_config = Config::default();

        Self {
            port: envs.get("PORT").unwrap_or(&default_config.port).to_string(),
            domain: envs
                .get("DOMAIN")
                .unwrap_or(&default_config.domain)
                .to_string(),
            db_endpoint: envs
                .get("DB_ENDPOINT")
                .unwrap_or(&default_config.db_endpoint)
                .to_string(),
            session_token_key: envs
                .get("TOKEN_KEY")
                .unwrap_or(&default_config.session_token_key)
                .to_string(),
            telegram_user_id: envs
                .get("TEL_USR_ID")
                .unwrap_or(&default_config.telegram_user_id)
                .to_string(),
            telegram_bot_key: envs
                .get("TEL_BOT_KEY")
                .unwrap_or(&default_config.telegram_bot_key)
                .to_string(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: "8080".into(),
            domain: "http://localhost:8080".into(),
            db_endpoint: "ws://localhost:5556".into(),
            session_token_key: "TvuUto7mf8EHYHzzV/sL25hzjQODnDv/4BXpg0laDfE=".into(),
            telegram_user_id: "0123456789".into(),
            telegram_bot_key: "0123456789".into(),
        }
    }
}
