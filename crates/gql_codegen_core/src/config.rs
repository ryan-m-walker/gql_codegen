//! Configuration types for SGC.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

// Re-export casing types for convenience
pub use crate::casing::{NamingCase, NamingConvention, NamingConventionConfig};

/// Main configuration — matches TypeScript `CodegenConfig`
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CodegenConfig {
    /// Path to GraphQL schema file(s)
    pub schema: StringOrArray,

    /// Glob patterns for documents
    pub documents: StringOrArray,

    /// Output configurations keyed by output path
    pub outputs: HashMap<String, OutputConfig>,

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
    /// Generators to run. Defaults to all built-in generators when omitted.
    #[serde(default, alias = "plugins")]
    pub generators: Option<Vec<GeneratorConfig>>,

    /// Content to prepend
    #[serde(default)]
    pub prelude: Option<String>,

    /// Shared config for all generators in this output
    #[serde(default)]
    pub config: Option<GeneratorOptions>,
}

/// Generator configuration — either just a name or name with config
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum GeneratorConfig {
    /// Just the generator name string
    Name(String),
    /// Object with generator name as key and config as value
    WithConfig(HashMap<String, GeneratorOptions>),
}

impl GeneratorConfig {
    /// Get the generator name
    pub fn name(&self) -> &str {
        match self {
            Self::Name(name) => name,
            Self::WithConfig(map) => map.keys().next().map(|s| s.as_str()).unwrap_or(""),
        }
    }

    /// Get the generator-specific config if any
    pub fn options(&self) -> Option<&GeneratorOptions> {
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

/// Generator options — shared config structure for all generators.
///
/// User-configurable fields are exposed via normal serde deserialization.
/// Internal-only fields remain `#[serde(skip)]` and always use SGC defaults.
///
/// Fields that were previously `bool` but are now user-configurable use
/// `Option<bool>` so `merge_options` can distinguish "not set" (`None`)
/// from "explicitly set to false" (`Some(false)`).
///
/// Uses BTreeMap for scalars to enable Hash derivation (cache key generation).
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GeneratorOptions {
    // ─────────────────────────────────────────────────────────────────────────
    // User-configurable fields (exposed in config schema)
    // ─────────────────────────────────────────────────────────────────────────
    /// Custom scalar mappings (GraphQL scalar name -> TypeScript type)
    #[serde(default)]
    pub scalars: BTreeMap<String, ScalarConfig>,

    /// Add readonly modifier to generated types
    #[serde(default)]
    pub immutable_types: Option<bool>,

    /// Generate enums as TypeScript string union types
    #[serde(default)]
    pub enums_as_types: Option<bool>,

    /// Add future-proof "%future added value" to enums
    #[serde(default)]
    pub future_proof_enums: Option<bool>,

    /// Add future-proof entry to union types
    #[serde(default)]
    pub future_proof_unions: Option<bool>,

    /// Use `interface` or `type` for generated types
    #[serde(default)]
    pub declaration_kind: Option<DeclarationKind>,

    /// Prefix to add to all generated type names
    #[serde(default)]
    pub type_name_prefix: Option<String>,

    /// Suffix to add to all generated type names
    #[serde(default)]
    pub type_name_suffix: Option<String>,

    /// Error if a custom scalar is found without a mapping in `scalars`
    #[serde(default)]
    pub strict_scalars: bool,

    /// Only generate types that are referenced in operations
    #[serde(default)]
    pub only_referenced_types: bool,

    /// Default type to use for unknown scalars (default: "unknown")
    #[schemars(skip)]
    pub default_scalar_type: Option<String>,

    /// Controls how `__typename` is emitted
    #[schemars(skip)]
    pub typename_policy: Option<TypenamePolicy>,

    /// Naming convention for generated types
    #[schemars(skip)]
    pub naming_convention: Option<NamingConvention>,
}

/// Manual Default impl — returns fixed SGC defaults.
///
/// These are the opinionated defaults optimized for TypeScript compiler
/// performance and type safety. All generators receive these values.
impl Default for GeneratorOptions {
    fn default() -> Self {
        Self {
            scalars: BTreeMap::new(),
            // SGC defaults
            // https://github.com/microsoft/TypeScript/wiki/Performance#preferring-interfaces-over-intersections
            declaration_kind: Some(DeclarationKind::Interface),
            enums_as_types: Some(true),
            future_proof_enums: Some(true),
            future_proof_unions: Some(true),
            immutable_types: Some(true),
            only_referenced_types: false,
            default_scalar_type: Some("unknown".to_string()),
            typename_policy: Some(TypenamePolicy::Always),
            strict_scalars: false,
            type_name_prefix: None,
            type_name_suffix: None,
            naming_convention: None,
        }
    }
}

impl GeneratorOptions {
    /// Resolve the effective typename policy.
    /// Legacy `skip_typename` flag takes precedence for backwards compat.
    pub fn resolved_typename_policy(&self) -> TypenamePolicy {
        self.typename_policy.unwrap_or(TypenamePolicy::Always)
    }

    /// Resolve immutable_types with SGC default fallback
    pub fn immutable_types(&self) -> bool {
        self.immutable_types.unwrap_or(true)
    }

    /// Resolve future_proof_enums with SGC default fallback
    pub fn future_proof_enums(&self) -> bool {
        self.future_proof_enums.unwrap_or(true)
    }

    /// Resolve future_proof_unions with SGC default fallback
    pub fn future_proof_unions(&self) -> bool {
        self.future_proof_unions.unwrap_or(true)
    }
}

/// Controls how `__typename` is emitted in generated types
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum TypenamePolicy {
    /// Always inject `__typename?: 'Type'` on every selection set, even if not
    /// explicitly queried.
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

/// Lifecycle hooks — shell commands run after generation.
/// Commands receive file paths as trailing arguments.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HooksConfig {
    /// Commands to run after all files are generated (receives all file paths)
    #[serde(default)]
    pub after_generate: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hooks_config_empty_json() {
        let hooks: HooksConfig = serde_json::from_str("{}").unwrap();
        assert!(hooks.after_generate.is_empty());
    }

    #[test]
    fn hooks_config_with_after_generate() {
        let hooks: HooksConfig =
            serde_json::from_str(r#"{"afterGenerate": ["biome format --write"]}"#).unwrap();
        assert_eq!(hooks.after_generate, vec!["biome format --write"]);
    }

    #[test]
    fn codegen_config_with_hooks() {
        let json = r#"{
            "schema": "schema.graphql",
            "documents": "src/**/*.graphql",
            "outputs": {},
            "hooks": {
                "afterGenerate": ["biome format --write"]
            }
        }"#;
        let config: CodegenConfig = serde_json::from_str(json).unwrap();
        let hooks = config.hooks.unwrap();
        assert_eq!(hooks.after_generate, vec!["biome format --write"]);
    }

    #[test]
    fn codegen_config_without_hooks() {
        let json = r#"{
            "schema": "schema.graphql",
            "documents": "src/**/*.graphql",
            "outputs": {}
        }"#;
        let config: CodegenConfig = serde_json::from_str(json).unwrap();
        assert!(config.hooks.is_none());
    }

    #[test]
    fn codegen_config_backwards_compat_generates() {
        let json = r#"{
            "schema": "schema.graphql",
            "documents": "src/**/*.graphql",
            "generates": {}
        }"#;
        let config: CodegenConfig = serde_json::from_str(json).unwrap();
        assert!(config.outputs.is_empty());
    }

    #[test]
    fn output_config_backwards_compat_plugins() {
        let json = r#"{
            "plugins": ["typescript"]
        }"#;
        let config: OutputConfig = serde_json::from_str(json).unwrap();
        let generators = config.generators.unwrap();
        assert_eq!(generators.len(), 1);
        assert_eq!(generators[0].name(), "typescript");
    }

    #[test]
    fn output_config_generators_optional() {
        let json = r#"{}"#;
        let config: OutputConfig = serde_json::from_str(json).unwrap();
        assert!(config.generators.is_none());
    }

    #[test]
    fn default_generator_options_are_sgc_defaults() {
        let opts = GeneratorOptions::default();
        assert_eq!(opts.declaration_kind, Some(DeclarationKind::Interface));
        assert_eq!(opts.immutable_types, Some(true));
        assert_eq!(opts.enums_as_types, Some(true));
        assert_eq!(opts.future_proof_enums, Some(true));
        assert_eq!(opts.future_proof_unions, Some(true));
        assert_eq!(opts.default_scalar_type, Some("unknown".to_string()));
        assert_eq!(opts.typename_policy, Some(TypenamePolicy::Always));
    }

    #[test]
    fn generator_options_exposed_fields_are_deserialized() {
        let json = r#"{
            "scalars": {"DateTime": "string"},
            "immutableTypes": false,
            "enumsAsTypes": false,
            "futureProofEnums": false,
            "futureProofUnions": false,
            "declarationKind": "type",
            "typeNamePrefix": "I",
            "typeNameSuffix": "Type"
        }"#;
        let opts: GeneratorOptions = serde_json::from_str(json).unwrap();
        assert!(opts.scalars.contains_key("DateTime"));
        assert_eq!(opts.immutable_types, Some(false));
        assert_eq!(opts.enums_as_types, Some(false));
        assert_eq!(opts.future_proof_enums, Some(false));
        assert_eq!(opts.future_proof_unions, Some(false));
        assert_eq!(opts.declaration_kind, Some(DeclarationKind::Type));
        assert_eq!(opts.type_name_prefix, Some("I".to_string()));
        assert_eq!(opts.type_name_suffix, Some("Type".to_string()));
    }

    #[test]
    fn generator_options_unset_fields_are_none() {
        let json = r#"{"scalars": {"DateTime": "string"}}"#;
        let opts: GeneratorOptions = serde_json::from_str(json).unwrap();
        assert!(opts.scalars.contains_key("DateTime"));
        // Exposed Option fields should be None when not set in JSON
        assert_eq!(opts.immutable_types, None);
        assert_eq!(opts.enums_as_types, None);
        assert_eq!(opts.future_proof_enums, None);
        assert_eq!(opts.future_proof_unions, None);
        assert_eq!(opts.declaration_kind, None);
        assert_eq!(opts.type_name_prefix, None);
        assert_eq!(opts.type_name_suffix, None);
    }

    #[test]
    fn generator_config_with_options() {
        let json = r#"{"schema-types": {"scalars": {"DateTime": "string"}}}"#;
        let config: GeneratorConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.name(), "schema-types");
        assert!(config.options().is_some());
        assert!(config.options().unwrap().scalars.contains_key("DateTime"));
    }
}
