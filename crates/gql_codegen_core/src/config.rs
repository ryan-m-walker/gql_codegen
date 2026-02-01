//! Configuration types matching the TypeScript interface.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

// Re-export casing types for convenience
pub use crate::casing::{NamingCase, NamingConvention, NamingConventionConfig};

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
/// Uses BTreeMap for scalars to enable Hash derivation (cache key generation)
#[derive(Debug, Clone, Default, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginOptions {
    /// Custom scalar mappings (GraphQL scalar name -> TypeScript type)
    #[serde(default)]
    pub scalars: BTreeMap<String, String>,

    /// Error if a custom scalar is found without a mapping in `scalars`
    /// Helps catch missing scalar configurations early
    #[serde(default)]
    pub strict_scalars: bool,

    /// Default type to use for unknown scalars (default: "unknown")
    /// Common values: "unknown", "any", "string"
    #[serde(default)]
    pub default_scalar_type: Option<String>,

    /// Add readonly modifier to generated types
    #[serde(default)]
    pub immutable_types: bool,

    /// Generate enums as TypeScript string union types (default: true)
    #[serde(default = "default_true")]
    pub enums_as_types: bool,

    /// Generate enums as `as const` objects (better tree-shaking)
    #[serde(default)]
    pub enums_as_const: bool,

    /// Add future-proof "%future added value" to enums
    #[serde(default)]
    pub future_proof_enums: bool,

    /// Prefix to add to enum type names
    #[serde(default)]
    pub enum_prefix: Option<String>,

    /// Suffix to add to enum type names
    #[serde(default)]
    pub enum_suffix: Option<String>,

    /// Use `const enum` instead of `enum` for better tree-shaking
    #[serde(default)]
    pub const_enums: bool,

    /// Skip `export` keyword on generated types
    #[serde(default)]
    pub no_export: bool,

    /// Only generate types that are used in operations
    /// Reduces output size by omitting unused schema types
    #[serde(default)]
    pub only_operation_types: bool,

    /// Add future-proof entry to union types
    #[serde(default)]
    pub future_proof_unions: bool,

    /// Skip __typename field in generated types
    #[serde(default)]
    pub skip_typename: bool,

    /// Make __typename non-optional (always required)
    #[serde(default)]
    pub non_optional_typename: bool,

    /// Avoid using TypeScript optionals (?), use explicit null instead
    /// Alias: useNullForOptional
    #[serde(default, alias = "useNullForOptional")]
    pub avoid_optionals: bool,

    /// Customize the Maybe type (default: "T | null")
    /// Examples: "T | null | undefined", "Maybe<T>"
    #[serde(default)]
    pub maybe_value: Option<String>,

    /// Separate Maybe type for input fields/arguments (default: uses maybe_value)
    /// Useful for differentiating input vs output nullability handling
    #[serde(default)]
    pub input_maybe_value: Option<String>,

    /// Use `type` instead of `interface` for object types
    #[serde(default)]
    pub declaration_kind: Option<DeclarationKind>,

    /// Prefix to add to all generated type names
    #[serde(default)]
    pub types_prefix: Option<String>,

    /// Suffix to add to all generated type names
    #[serde(default)]
    pub types_suffix: Option<String>,

    /// Use `import type` syntax for type imports
    #[serde(default)]
    pub use_type_imports: bool,

    /// GraphQL tag style for document generator
    #[serde(default)]
    pub graphql_tag: Option<GraphqlTag>,

    /// Formatting options
    #[serde(default)]
    pub formatting: Option<FormattingOptions>,

    /// Inline fragment spreads into document text (document generator)
    #[serde(default)]
    pub inline_fragments: bool,

    /// Remove duplicate field selections (document generator)
    #[serde(default)]
    pub dedupe_selections: bool,

    /// Naming convention for generated types
    /// Can be a string ("keep", "pascalCase", etc.) or object with typeNames/enumValues
    #[serde(default)]
    pub naming_convention: Option<NamingConvention>,
}

/// Declaration kind for generated types
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeclarationKind {
    #[default]
    Interface,
    Type,
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
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
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
