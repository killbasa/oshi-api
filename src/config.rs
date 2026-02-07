use std::{collections::HashMap, env};

use once_cell::sync::Lazy;

pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub log_level: tracing::Level,
}

pub struct YoutubeConfig {
    pub apikey: String,
}

pub struct Config {
    pub server: ServerConfig,
    pub youtube: YoutubeConfig,
    pub browser_redirect: String,
    pub oshi: HashMap<String, String>,
}

impl Config {
    fn get() -> Config {
        let level = match env::var("DEBUG_LOG").as_deref() {
            Ok("1") | Ok("true") => tracing::Level::DEBUG,
            _ => tracing::Level::INFO,
        };

        Config {
            browser_redirect: "https://github.com/killbasa/oshi-api".into(),
            server: ServerConfig {
                host: env::var("HOST").unwrap_or("127.0.0.1".to_string()),
                port: env::var("PORT").unwrap_or("3000".to_string()).parse().unwrap_or(3000),
                log_level: level,
            },
            youtube: YoutubeConfig {
                apikey: env::var("YOUTUBE_APIKEY").unwrap(), //
            },
            oshi: HashMap::from([
                ("furi".into(), "UCb8dLvDvmZ-d92KEy_9oWog".into()),
                ("phish".into(), "UC9iiZCKQ9jnIM7zZ_mRX_cg".into()),
                ("mono".into(), "UCdubotSy4pPOsiaW4MrYn3Q".into()),
                ("raki".into(), "UCtuoyOZhnxJ12pE294FdH8Q".into()),
            ]),
        }
    }
}

pub static CONFIG: Lazy<Config> = Lazy::new(Config::get);
