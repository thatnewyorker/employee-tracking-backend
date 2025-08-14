// src/config.rs

use std::env;
use std::net::SocketAddr;
use std::str::FromStr;

use tracing::error;

#[derive(Debug)]
pub struct Config {
    pub bind_address: SocketAddr,
    pub database_url: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let bind_address_str =
            env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1:3000".to_string());

        let bind_address = SocketAddr::from_str(&bind_address_str)
            .map_err(|e| format!("Invalid BIND_ADDRESS: {}", e))?;

        #[cfg(feature = "no-auth")]
        if bind_address.ip() != std::net::IpAddr::from([127, 0, 0, 1]) {
            error!("In no-auth mode, BIND_ADDRESS must be 127.0.0.1");
            return Err("In no-auth mode, BIND_ADDRESS must be 127.0.0.1".to_string());
        }

        let database_url = env::var("DATABASE_URL").ok();

        Ok(Self {
            bind_address,
            database_url,
        })
    }
}
