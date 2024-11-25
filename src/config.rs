use std::collections::HashMap;
use std::sync::OnceLock;

use crate::error::Error;

pub static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct Config(HashMap<String, String>);

pub fn config(key: &str) -> Result<&String, Error> {
    CONFIG
        .get_or_init(Config::load)
        .0
        .get(key)
        .ok_or(Error::EnvVar(key.into()))
}

pub fn config_bytes(key: &str) -> Result<Vec<u8>, Error> {
    CONFIG
        .get_or_init(Config::load)
        .0
        .get(key)
        .ok_or(Error::EnvVar(key.into()))
        .map(|string| string.clone().into())
}

impl Config {
    pub fn load() -> Self {
        let file = std::fs::read_to_string(".env").unwrap_or_else(|_| {
            std::fs::read_to_string(".env.dev").expect("you either .env or .env.dev")
        });

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

        // resolve nested env vars
        let mut resolved_map = HashMap::new();
        for (key, value) in env_map.iter() {
            let mut resolved_value = value.clone();

            while let Some(start) = resolved_value.find("${") {
                if let Some(end) = resolved_value[start..].find("}") {
                    let var_name = &resolved_value[start + 2..start + end];
                    if let Some(var_value) = env_map.get(var_name) {
                        resolved_value.replace_range(start..start + end + 1, var_value);
                    } else {
                        // If the variable is not found, leave it as is
                        break;
                    }
                }
            }

            // Insert the resolved value into the final map
            resolved_map.insert(key.clone(), resolved_value);
        }

        Self(resolved_map)
    }

    pub fn show(&self) {
        println!("config: {self:?}");
    }
}
