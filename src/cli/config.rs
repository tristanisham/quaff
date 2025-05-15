use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub lang: String,
}

pub enum LangOption {
    PHP,
}

impl FromStr for LangOption {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "php" => Ok(Self::PHP),
            _ => Err(anyhow!("cannot convert string to language value")),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            lang: String::from("PHP"),
        }
    }
}
