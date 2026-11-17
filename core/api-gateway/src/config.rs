use std::env;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: Option<String>,
    pub database_max_connections: u32,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, String> {
        let host = env::var("CLOUDFORGE_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_owned());
        if host.trim().is_empty() {
            return Err("CLOUDFORGE_HOST cannot be empty".to_owned());
        }

        let port = parse_env("CLOUDFORGE_PORT", 8080_u16)?;
        let database_max_connections =
            parse_env("CLOUDFORGE_DATABASE_MAX_CONNECTIONS", 5_u32)?;
        if database_max_connections == 0 {
            return Err("CLOUDFORGE_DATABASE_MAX_CONNECTIONS must be greater than zero".to_owned());
        }

        let database_url = env::var("DATABASE_URL")
            .ok()
            .filter(|value| !value.trim().is_empty());

        Ok(Self {
            host,
            port,
            database_url,
            database_max_connections,
        })
    }

    pub fn socket_addr(&self) -> Result<String, String> {
        Ok(format!("{}:{}", self.host, self.port))
    }
}

fn parse_env<T>(key: &str, default: T) -> Result<T, String>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    match env::var(key) {
        Ok(value) => value
            .parse()
            .map_err(|error| format!("{key} has an invalid value: {error}")),
        Err(env::VarError::NotPresent) => Ok(default),
        Err(error) => Err(format!("Unable to read {key}: {error}")),
    }
}
