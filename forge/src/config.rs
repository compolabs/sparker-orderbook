use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader, path::Path};

use crate::error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketInfo {
    /// Market id
    pub id: String,

    /// Market name
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Pangea host url
    pub pangea_host: String,

    /// Pangea start block
    pub pangea_start_block: i64,

    /// Markets
    pub markets: Vec<MarketInfo>,
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let config: Config = serde_json::from_reader(reader)?;

        Ok(config)
    }
}
