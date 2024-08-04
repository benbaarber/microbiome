use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: String,
    pub static_path: String,
    pub sub_at: String,
}

impl Config {
    pub fn read_env() -> Result<Self, env::VarError> {
        Ok(Self {
            host: env::var("MB_SERVER_HOST")?,
            port: env::var("MB_SERVER_PORT")?,
            static_path: env::var("MB_UI_PATH")?,
            sub_at: env::var("MB_PUBSUB")?,
        })
    }
}
