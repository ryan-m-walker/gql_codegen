use crate::cache::Cache;
use crate::{CodegenConfig, Writer};

// Cache  - FS, Memory, Noop
// Writer - FS, Memory, Stdout, Noop
// Reader - FS, Memory,
// Logger - Stdout, Memory, configurable for output format/ascii or not, etc...
struct RunInput {
    config: CodegenConfig,
    cache: Option<Box<dyn Cache>>,
    writer: Option<Box<dyn Writer>>,
    // reader/sources
}

pub fn generate(config: RunInput) {}

// Reader:
// or maybe Sources? .get_document, .get_schema, maybe based off of our current source cache
// get_schema(config: CodegenConfig) -> String
// get_documents(config: CodegenConfig) -> Vec<String>
//
// MemoryReader::new(schema, docs);
// FsReader::new().read()?; -> load sources from the filesystem, or maybe just a function that returns a source cache?
// Maybe Sources::read_from_fs(config) ? or something
