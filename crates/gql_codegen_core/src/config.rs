//! Configuration types matching the TypeScript interface.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

// Re-export casing types for convenience
pub use crate::casing::{NamingCase, NamingConvention, NamingConventionConfig};

/// Main configuration - matches TypeScript `CodegenConfig`
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

    /// Pre-resolved schema SDL strings (set by Node CLI for .ts/.js schemas).
    /// These are parsed alongside `schema` file paths — both contribute to
    /// the final merged schema.
    #[serde(default)]
    pub schema_content: Option<Vec<String>>,

    /// Lifecycle hooks — shell commands run after generation
    #[serde(default)]
    pub hooks: Option<HooksConfig>,
}

/// Either a single string or array of strings
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

    /// Lifecycle hooks — shell commands run after this output is written
    #[serde(default)]
    pub hooks: Option<HooksConfig>,
}

/// Plugin configuration - either just name or name with config
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AvoidOptionalsConfig {
    field: Option<bool>,
    object: Option<bool>,
    input_value: Option<bool>,
    default_value: Option<bool>,
    resolvers: Option<bool>,
    query: Option<bool>,
    mutation: Option<bool>,
    subscription: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum AvoidOptionals {
    Boolean(bool),
    Complex(AvoidOptionalsConfig),
}

impl Default for AvoidOptionals {
    fn default() -> Self {
        AvoidOptionals::Boolean(false)
    }
}

/// Resolved avoid-optionals flags — all fields are concrete bools.
/// Mirrors JS `normalizeAvoidOptionals()` from visitor-plugin-common.
#[derive(Debug, Clone, Copy)]
pub struct NormalizedAvoidOptionals {
    pub field: bool,
    pub object: bool,
    pub input_value: bool,
    pub default_value: bool,
}

impl AvoidOptionals {
    pub fn normalize(&self) -> NormalizedAvoidOptionals {
        match self {
            AvoidOptionals::Boolean(b) => NormalizedAvoidOptionals {
                field: *b,
                object: *b,
                input_value: *b,
                default_value: *b,
            },
            AvoidOptionals::Complex(c) => NormalizedAvoidOptionals {
                field: c.field.unwrap_or(false),
                object: c.object.unwrap_or(false),
                input_value: c.input_value.unwrap_or(false),
                default_value: c.default_value.unwrap_or(false),
            },
        }
    }
}

pub struct ScalarConfigNormalized {
    input: String,
    output: String,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ScalarConfig {
    Simple(String),
    Detailed { input: String, output: String },
}

impl ScalarConfig {
    pub fn normalize(&self) -> ScalarConfigNormalized {
        match self {
            ScalarConfig::Simple(s) => ScalarConfigNormalized {
                input: s.clone(),
                output: s.clone(),
            },
            ScalarConfig::Detailed { input, output } => ScalarConfigNormalized {
                input: input.clone(),
                output: output.clone(),
            },
        }
    }
}

/// Plugin options - shared config structure.
///
/// Only `scalars` is user-configurable via JSON config. All other fields are
/// locked to SGC defaults via `#[serde(skip)]` and the manual `Default` impl.
/// Generators read these fields as before — they just always get SGC values now.
///
/// Uses BTreeMap for scalars to enable Hash derivation (cache key generation).
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PluginOptions {
    // ─────────────────────────────────────────────────────────────────────────
    // User-configurable fields
    // ─────────────────────────────────────────────────────────────────────────
    /// Custom scalar mappings (GraphQL scalar name -> TypeScript type)
    #[serde(default)]
    pub scalars: BTreeMap<String, ScalarConfig>,

    // ─────────────────────────────────────────────────────────────────────────
    // Fixed SGC defaults (not exposed in config schema)
    // ─────────────────────────────────────────────────────────────────────────
    /// When false (default): inline scalars and nullables directly
    #[serde(skip)]
    #[schemars(skip)]
    pub use_utility_types: bool,

    /// Inline fragment spreads into document text (document generator)
    #[serde(skip)]
    #[schemars(skip)]
    pub inline_fragments: bool,

    /// Remove duplicate field selections (document generator)
    #[serde(skip)]
    #[schemars(skip)]
    pub dedupe_selections: bool,

    /// Skip rendering of built-in scalars
    #[serde(skip)]
    #[schemars(skip)]
    pub disable_descriptions: bool,

    /// Error if a custom scalar is found without a mapping in `scalars`
    #[serde(skip)]
    #[schemars(skip)]
    pub strict_scalars: bool,

    /// Default type to use for unknown scalars (default: "unknown")
    #[serde(skip)]
    #[schemars(skip)]
    pub default_scalar_type: Option<String>,

    /// Add readonly modifier to generated types
    #[serde(skip)]
    #[schemars(skip)]
    pub immutable_types: bool,

    /// Generate enums as TypeScript string union types
    #[serde(skip)]
    #[schemars(skip)]
    pub enums_as_types: Option<bool>,

    /// Generate enums as `as const` objects (better tree-shaking)
    #[serde(skip)]
    #[schemars(skip)]
    pub enums_as_const: bool,

    /// Add future-proof "%future added value" to enums
    #[serde(skip)]
    #[schemars(skip)]
    pub future_proof_enums: bool,

    /// Prefix to add to enum type names
    #[serde(skip)]
    #[schemars(skip)]
    pub enum_prefix: Option<String>,

    /// Suffix to add to enum type names
    #[serde(skip)]
    #[schemars(skip)]
    pub enum_suffix: Option<String>,

    /// Use `const enum` instead of `enum` for better tree-shaking
    #[serde(skip)]
    #[schemars(skip)]
    pub const_enums: bool,

    /// Use numeric enum values instead of string literals
    #[serde(skip)]
    #[schemars(skip)]
    pub numeric_enums: bool,

    /// Only generate enums, no other types
    #[serde(skip)]
    #[schemars(skip)]
    pub only_enums: bool,

    /// Skip `export` keyword on generated types
    #[serde(skip)]
    #[schemars(skip)]
    pub no_export: bool,

    /// Only generate types that are used in operations
    #[serde(skip)]
    #[schemars(skip)]
    pub only_operation_types: bool,

    /// Add future-proof entry to union types
    #[serde(skip)]
    #[schemars(skip)]
    pub future_proof_unions: bool,

    /// Controls how `__typename` is emitted
    #[serde(skip)]
    #[schemars(skip)]
    pub typename_policy: Option<TypenamePolicy>,

    /// Skip __typename field in generated types
    #[serde(skip)]
    #[schemars(skip)]
    pub skip_typename: bool,

    /// Make __typename non-optional
    #[serde(skip)]
    #[schemars(skip)]
    pub non_optional_typename: bool,

    /// Avoid using TypeScript optionals (?), use explicit null instead
    #[serde(skip)]
    #[schemars(skip)]
    pub avoid_optionals: AvoidOptionals,

    /// Customize the Maybe type
    #[serde(skip)]
    #[schemars(skip)]
    pub maybe_value: Option<String>,

    /// Separate Maybe type for input fields/arguments
    #[serde(skip)]
    #[schemars(skip)]
    pub input_maybe_value: Option<String>,

    /// Use `type` instead of `interface` for object types
    #[serde(skip)]
    #[schemars(skip)]
    pub declaration_kind: Option<DeclarationKind>,

    /// Prefix to add to all generated type names
    #[serde(skip)]
    #[schemars(skip)]
    pub types_prefix: Option<String>,

    /// Suffix to add to all generated type names
    #[serde(skip)]
    #[schemars(skip)]
    pub types_suffix: Option<String>,

    /// Use `import type` syntax for type imports
    #[serde(skip)]
    #[schemars(skip)]
    pub use_type_imports: bool,

    /// GraphQL tag style for document generator
    #[serde(skip)]
    #[schemars(skip)]
    pub graphql_tag: Option<GraphqlTag>,

    /// Naming convention for generated types
    #[serde(skip)]
    #[schemars(skip)]
    pub naming_convention: Option<NamingConvention>,

    /// Pretty-print document strings with indentation
    #[serde(skip)]
    #[schemars(skip)]
    pub pretty_documents: bool,
}

/// Manual Default impl — returns fixed SGC defaults.
///
/// These are the opinionated defaults optimized for TypeScript compiler
/// performance and type safety. All generators receive these values.
impl Default for PluginOptions {
    fn default() -> Self {
        Self {
            scalars: BTreeMap::new(),
            // SGC defaults
            // https://github.com/microsoft/TypeScript/wiki/Performance#preferring-interfaces-over-intersections
            declaration_kind: Some(DeclarationKind::Interface),
            use_utility_types: false,
            enums_as_types: Some(true),
            future_proof_enums: true,
            future_proof_unions: true,
            immutable_types: true,
            default_scalar_type: Some("unknown".to_string()),
            typename_policy: Some(TypenamePolicy::Always),
            pretty_documents: true,
            // Everything else: false/None/default
            inline_fragments: false,
            dedupe_selections: false,
            disable_descriptions: false,
            strict_scalars: false,
            enums_as_const: false,
            enum_prefix: None,
            enum_suffix: None,
            const_enums: false,
            numeric_enums: false,
            only_enums: false,
            no_export: false,
            only_operation_types: false,
            skip_typename: false,
            non_optional_typename: false,
            avoid_optionals: AvoidOptionals::Boolean(false),
            maybe_value: None,
            input_maybe_value: None,
            types_prefix: None,
            types_suffix: None,
            use_type_imports: false,
            graphql_tag: None,
            naming_convention: None,
        }
    }
}

impl PluginOptions {
    /// Resolve the effective typename policy.
    /// Legacy `skip_typename` flag takes precedence for backwards compat.
    pub fn resolved_typename_policy(&self) -> TypenamePolicy {
        if self.skip_typename {
            return TypenamePolicy::Skip;
        }
        self.typename_policy.unwrap_or(TypenamePolicy::Always)
    }
}

/// Controls how `__typename` is emitted in generated types
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum TypenamePolicy {
    /// Always inject `__typename?: 'Type'` on every selection set, even if not
    /// explicitly queried. Matches graphql-codegen behavior.
    #[default]
    Always,
    /// Only emit `__typename` when explicitly selected in the query.
    /// When selected, it's non-optional.
    AsSelected,
    /// Never emit `__typename`.
    Skip,
}

/// Declaration kind for generated types
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum DeclarationKind {
    #[default]
    Type,
    Interface,
    Class,
    #[serde(rename = "abstract class")]
    AbstractClass,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum GraphqlTag {
    #[default]
    Gql,
    Graphql,
    None,
}

/// Lifecycle hooks — shell commands run at various pipeline stages.
/// Commands receive file path(s) as trailing arguments.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HooksConfig {
    /// Commands to run after each file is written (receives single file path)
    #[serde(default)]
    pub after_one_file_write: Vec<String>,

    /// Commands to run after all files are written (receives all file paths)
    #[serde(default)]
    pub after_all_file_write: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hooks_config_empty_json() {
        let hooks: HooksConfig = serde_json::from_str("{}").unwrap();
        assert!(hooks.after_one_file_write.is_empty());
        assert!(hooks.after_all_file_write.is_empty());
    }

    #[test]
    fn hooks_config_partial() {
        let hooks: HooksConfig =
            serde_json::from_str(r#"{"afterOneFileWrite": ["biome format --write"]}"#).unwrap();
        assert_eq!(hooks.after_one_file_write, vec!["biome format --write"]);
        assert!(hooks.after_all_file_write.is_empty());
    }

    #[test]
    fn hooks_config_full() {
        let hooks: HooksConfig = serde_json::from_str(
            r#"{"afterOneFileWrite": ["prettier --write"], "afterAllFileWrite": ["eslint --fix"]}"#,
        )
        .unwrap();
        assert_eq!(hooks.after_one_file_write, vec!["prettier --write"]);
        assert_eq!(hooks.after_all_file_write, vec!["eslint --fix"]);
    }

    #[test]
    fn codegen_config_with_hooks() {
        let json = r#"{
            "schema": "schema.graphql",
            "documents": "src/**/*.graphql",
            "generates": {},
            "hooks": {
                "afterOneFileWrite": ["biome format --write"]
            }
        }"#;
        let config: CodegenConfig = serde_json::from_str(json).unwrap();
        let hooks = config.hooks.unwrap();
        assert_eq!(hooks.after_one_file_write, vec!["biome format --write"]);
    }

    #[test]
    fn codegen_config_without_hooks() {
        let json = r#"{
            "schema": "schema.graphql",
            "documents": "src/**/*.graphql",
            "generates": {}
        }"#;
        let config: CodegenConfig = serde_json::from_str(json).unwrap();
        assert!(config.hooks.is_none());
    }

    #[test]
    fn output_config_with_hooks() {
        let json = r#"{
            "plugins": ["typescript"],
            "hooks": {
                "afterOneFileWrite": ["biome format --write"],
                "afterAllFileWrite": ["echo done"]
            }
        }"#;
        let config: OutputConfig = serde_json::from_str(json).unwrap();
        let hooks = config.hooks.unwrap();
        assert_eq!(hooks.after_one_file_write, vec!["biome format --write"]);
        assert_eq!(hooks.after_all_file_write, vec!["echo done"]);
    }

    #[test]
    fn default_plugin_options_are_sgc_defaults() {
        let opts = PluginOptions::default();
        assert_eq!(opts.declaration_kind, Some(DeclarationKind::Interface));
        assert!(opts.immutable_types);
        assert_eq!(opts.enums_as_types, Some(true));
        assert!(opts.future_proof_enums);
        assert!(opts.future_proof_unions);
        assert_eq!(opts.default_scalar_type, Some("unknown".to_string()));
        assert_eq!(opts.typename_policy, Some(TypenamePolicy::Always));
        assert!(opts.pretty_documents);
        assert!(!opts.use_utility_types);
    }

    #[test]
    fn plugin_options_serde_only_reads_scalars() {
        // Verify that JSON config only affects scalars — other fields are skipped
        let json = r#"{"scalars": {"DateTime": "string"}, "immutableTypes": false}"#;
        let opts: PluginOptions = serde_json::from_str(json).unwrap();
        // scalars should be set
        assert!(opts.scalars.contains_key("DateTime"));
        // immutableTypes is #[serde(skip)] → gets bool::default() (false), NOT
        // the struct's manual Default impl (true). The production path then
        // merge_options() with SGC defaults as base, which restores it to true.
        assert!(!opts.immutable_types);
    }
}
