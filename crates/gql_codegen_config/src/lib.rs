use std::{collections::HashMap, fs::File};

use gql_codegen_generators::{
    ts_operation_types::TsOperationTypesGeneratorConfig,
    ts_schema_types::TsSchemaTypesGeneratorConfig,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "name")]
pub enum Generator {
    #[serde(rename = "ts_schema_types")]
    TsSchemaTypes {
        config: TsSchemaTypesGeneratorConfig,
    },
    #[serde(rename = "ts_operation_types")]
    TsOperationTypes {
        config: TsOperationTypesGeneratorConfig,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    pub generators: Vec<Generator>,
    pub prelude: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub schemas: Vec<String>,
    pub documents: String,
    pub outputs: HashMap<String, Output>,
}

impl Config {
    pub fn from_file(path: std::path::PathBuf) -> Self {
        let file = File::open(&path).unwrap();
        serde_json::from_reader(file).unwrap()
    }

    pub fn from_path(path: &str) -> Self {
        let path = std::path::PathBuf::from(path);
        Self::from_file(path)
    }
}
