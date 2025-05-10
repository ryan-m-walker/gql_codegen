use apollo_compiler::{
    Node,
    ast::{FragmentDefinition, OperationDefinition},
    parser::SourceMap,
};

#[derive(Debug, Default)]
pub struct ReadResult {
    pub path: String,
    pub documents: Vec<String>,
}

#[derive(Debug)]
pub struct OperationResult {
    pub operation: Node<OperationDefinition>,
    pub sources: SourceMap,
}

impl OperationResult {
    pub fn new(operation: Node<OperationDefinition>, sources: SourceMap) -> Self {
        Self { operation, sources }
    }
}

#[derive(Debug)]
pub struct FragmentResult {
    pub fragment: Node<FragmentDefinition>,
    pub sources: SourceMap,
}

impl FragmentResult {
    pub fn new(fragment: Node<FragmentDefinition>, sources: SourceMap) -> Self {
        Self { fragment, sources }
    }
}
