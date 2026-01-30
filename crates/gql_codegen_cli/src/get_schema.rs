use std::{fs, path::PathBuf};

use anyhow::{Context, anyhow};
use apollo_compiler::{Schema, validation::Valid};
use gql_codegen_config::Config;
use gql_codegen_logger::Logger;

pub fn get_schema(config: &Config, logger: &Logger) -> Result<Valid<Schema>, anyhow::Error> {
    let mut schema = Schema::builder();

    for schema_path in &config.schemas {
        let path = PathBuf::from(&config.src).join(schema_path);

        logger.info("Parsing schema file...");
        logger.debug(&format!(
            "Using schema filepath path {}",
            path.to_string_lossy()
        ));

        let schema_source = fs::read_to_string(&path)
            .context("Failed to read schema file. Please ensure that your configuration schema value is pointing to a valid file.")?;
        schema = schema.parse(schema_source, path);
    }

    let schema = match schema.build().unwrap().validate() {
        Ok(valid) => valid,
        Err(with_errors) => {
            let mut message = String::from("Error parsing schema:\n");

            for error in with_errors.errors.iter() {
                // TODO: show sources
                message.push_str(&format!("{}", error.error));
            }

            return Err(anyhow!(message));
        }
    };

    Ok(schema)
}
