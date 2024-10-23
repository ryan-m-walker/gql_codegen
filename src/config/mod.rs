use std::{fs, process};

use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub schema: String,
}

pub fn read_config() -> Config {
    let file = fs::read_to_string("config.json").expect("Unable to read file");
    let parsed: Result<Config> = serde_json::from_str(&file);

    match parsed {
        Ok(config) => {
            return config;
        }
        Err(err) => {
            println!("{:?}", err);
            process::exit(1);
        }
    }
}
