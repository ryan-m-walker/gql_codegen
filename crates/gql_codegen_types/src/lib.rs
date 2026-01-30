use std::path::PathBuf;

use apollo_compiler::{
    Name, Node, Schema,
    ast::{FragmentDefinition, OperationDefinition},
    parser::SourceMap,
    validation::Valid,
};
use gql_codegen_formatter::Formatter;
use gql_codegen_logger::Logger;
use indexmap::IndexMap;

#[derive(Debug, Default)]
pub struct ReadResult {
    pub path: PathBuf,
    pub documents: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct OperationResult {
    pub operation: Node<OperationDefinition>,
    pub sources: SourceMap,
    pub path: PathBuf,
}

impl OperationResult {
    pub fn new(operation: Node<OperationDefinition>, sources: SourceMap, path: PathBuf) -> Self {
        Self {
            operation,
            sources,
            path,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FragmentResult {
    pub fragment: Node<FragmentDefinition>,
    pub sources: SourceMap,
    pub path: PathBuf,
}

impl FragmentResult {
    pub fn new(fragment: Node<FragmentDefinition>, sources: SourceMap, path: PathBuf) -> Self {
        Self {
            fragment,
            sources,
            path,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Context<'a> {
    pub schema: &'a Valid<Schema>,
    pub operations: &'a IndexMap<Name, OperationResult>,
    pub fragments: &'a IndexMap<Name, FragmentResult>,
    pub formatter: Formatter,
    pub logger: &'a Logger,
}

impl<'a> Context<'a> {
    pub fn new(
        schema: &'a Valid<Schema>,
        operations: &'a IndexMap<Name, OperationResult>,
        fragments: &'a IndexMap<Name, FragmentResult>,
        formatter: Formatter,
        logger: &'a Logger,
    ) -> Self {
        Self {
            schema,
            operations,
            fragments,
            formatter,
            logger,
        }
    }
}
