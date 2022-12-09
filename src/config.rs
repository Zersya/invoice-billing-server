use config::ConfigError;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ServerConfig {
    pub host: String, 
    pub port: u16
}

#[derive(Deserialize)]
pub struct DatabaseConfig {
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: u32,
    pub dbname: String,
    pub poolmaxsize: u32,
}

#[derive(Deserialize)]
pub struct Config {
    pub server: Option<ServerConfig>,
    pub environment: Option<String>,
    pub appkey: Option<String>,
    pub pg: Option<DatabaseConfig>,
}

impl Config {
    pub fn from_env () -> Result<Self, ConfigError> {
       config::Config::builder()
            .add_source(config::Environment::default().separator("_"))
            .build()
            .unwrap()
            .try_deserialize()
    }

    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.pg.as_ref().unwrap().user,
            self.pg.as_ref().unwrap().password,
            self.pg.as_ref().unwrap().host,
            self.pg.as_ref().unwrap().port,
            self.pg.as_ref().unwrap().dbname
        )
    }
}
