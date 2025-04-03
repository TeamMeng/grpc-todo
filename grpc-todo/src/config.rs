use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::File;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database_url: String,
}

impl Config {
    pub fn new() -> Result<Self> {
        let rdr = File::open("backend.yaml")?;
        let ret = serde_yaml::from_reader(rdr)?;
        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn config_new_should_work() -> Result<()> {
        let config = Config::new()?;
        assert_eq!(
            config.database_url,
            "postgres://postgres:postgres@localhost:5432/todo"
        );
        Ok(())
    }
}
