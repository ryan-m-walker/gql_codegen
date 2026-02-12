//! Error types for gql_codegen_core

pub type Result<T> = std::result::Result<T, crate::diagnostic::Diagnostics>;
