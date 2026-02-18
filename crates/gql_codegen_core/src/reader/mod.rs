use crate::CodegenConfig;

pub trait Reader: Send + Sync {
    fn get_schema(&self, config: &CodegenConfig) -> String;

    fn get_documents(&self, config: &CodegenConfig) -> Vec<String>;
}
