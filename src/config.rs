use std::fs::read_to_string;

use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
pub struct CfConfig {
    db: ServiceConfig,
    http: ServiceConfig
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceConfig{
    host: String,
    port: usize,
}

impl CfConfig {
    /// Load configuration from file
    pub fn load(path: &str) -> anyhow::Result<Self> {
        info!("Load configuation file {}", path);
        let content = read_to_string(path)?;
        let decoded_config = toml::from_str(&*content)?;
        Ok(decoded_config)
    }

    /// Make connect URL string to MongoDB
    pub fn db_url(&self)  -> String {
        format!("mongodb://{}:{}", self.db.host, self.db.port)
    }

     /// Make rest service URL string 
     pub fn service_url(&self)  -> String {
        format!("http://{}:{}", self.http.host, self.http.port)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let cf=  CfConfig::load("src/config.toml").expect("load configration file");
        assert_eq!(cf.db.host, "127.0.0.1");
        assert_eq!(cf.db.port, 27017);
        assert_eq!(cf.db_url(), "mongodb://127.0.0.1:27017");
        assert_eq!(cf.service_url(), "http://127.0.0.1:18080");
    }
}
