//! Configuration types matching the TypeScript interface.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main configuration - matches TypeScript `CodegenConfig`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodegenConfig {
    /// Path to GraphQL schema file(s)
    pub schema: StringOrArray,

    /// Glob patterns for documents
    pub documents: StringOrArray,

    /// Output configurations keyed by output path
    pub generates: HashMap<String, OutputConfig>,

    /// Base directory for resolving paths (set by CLI)
    #[serde(default)]
    pub base_dir: Option<String>,
}

/// Either a single string or array of strings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringOrArray {
    Single(String),
    Multiple(Vec<String>),
}

impl StringOrArray {
    pub fn as_vec(&self) -> Vec<&str> {
        match self {
            Self::Single(s) => vec![s.as_str()],
            Self::Multiple(v) => v.iter().map(|s| s.as_str()).collect(),
        }
    }

    pub fn into_vec(self) -> Vec<String> {
        match self {
            Self::Single(s) => vec![s],
            Self::Multiple(v) => v,
        }
    }
}

/// Configuration for a single output file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputConfig {
    /// Plugins to run
    pub plugins: Vec<PluginConfig>,

    /// Content to prepend
    #[serde(default)]
    pub prelude: Option<String>,

    /// Shared config for all plugins
    #[serde(default)]
    pub config: Option<PluginOptions>,

    /// Only generate for documents, skip schema types
    #[serde(default)]
    pub documents_only: bool,
}

/// Plugin configuration - either just name or name with config
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PluginConfig {
    /// Just the plugin name string
    Name(String),
    /// Object with plugin name as key and config as value
    WithConfig(HashMap<String, PluginOptions>),
}

impl PluginConfig {
    /// Get the plugin name
    pub fn name(&self) -> &str {
        match self {
            Self::Name(name) => name,
            Self::WithConfig(map) => map.keys().next().map(|s| s.as_str()).unwrap_or(""),
        }
    }

    /// Get the plugin-specific config if any
    pub fn options(&self) -> Option<&PluginOptions> {
        match self {
            Self::Name(_) => None,
            Self::WithConfig(map) => map.values().next(),
        }
    }
}

/// Plugin options - shared config structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginOptions {
    /// Custom scalar mappings
    #[serde(default)]
    pub scalars: HashMap<String, String>,

    /// Add readonly modifier
    #[serde(default)]
    pub immutable_types: bool,

    /// Generate enums as string unions
    #[serde(default = "default_true")]
    pub enums_as_types: bool,

    /// Add future-proof value to enums
    #[serde(default)]
    pub future_proof_enums: bool,

    /// Skip __typename field
    #[serde(default)]
    pub skip_typename: bool,

    /// Use null instead of undefined for optional
    #[serde(default)]
    pub use_null_for_optional: bool,

    /// GraphQL tag style
    #[serde(default)]
    pub graphql_tag: Option<GraphqlTag>,

    /// Formatting options
    #[serde(default)]
    pub formatting: Option<FormattingOptions>,

    /// Inline fragment spreads into document text (document generator)
    /// Replaces `...FragmentName` with the fragment's actual fields
    #[serde(default)]
    pub inline_fragments: bool,

    /// Remove duplicate field selections (document generator)
    /// e.g. `{ id id name }` becomes `{ id name }`
    #[serde(default)]
    pub dedupe_selections: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GraphqlTag {
    #[default]
    Gql,
    Graphql,
    None,
}

/// Formatting options for generated code
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormattingOptions {
    #[serde(default = "default_indent_width")]
    pub indent_width: usize,

    #[serde(default)]
    pub use_tabs: bool,

    #[serde(default = "default_true")]
    pub single_quote: bool,

    #[serde(default = "default_true")]
    pub semicolons: bool,
}

impl Default for FormattingOptions {
    fn default() -> Self {
        Self {
            indent_width: default_indent_width(),
            use_tabs: false,
            single_quote: true,
            semicolons: true,
        }
    }
}

fn default_indent_width() -> usize {
    2
}
