//! Test utilities for generating TypeScript from GraphQL schemas.
//!
//! Two levels of API:
//! - [`TestGen`]: Full pipeline builder (schema → codegen → output string)
//! - [`TestCtx`]: Lightweight context for unit-testing individual renderers

use std::collections::HashMap;
use std::path::PathBuf;

use apollo_compiler::schema::{
    EnumType, ExtendedType, InputObjectType, InterfaceType, ObjectType, ScalarType, UnionType,
};
use apollo_compiler::validation::Valid;
use apollo_compiler::{Node, Schema};
use indexmap::IndexMap;

use crate::codegen::{GenerateInput, GenerateResult, generate_from_input};
use crate::config::{OutputConfig, PluginConfig, PluginOptions, Preset};
use crate::documents::collect_documents;
use crate::extract::ExtractConfig;
use crate::generators::GeneratorContext;
use crate::schema::load_schema_from_contents;
use crate::source_cache::SourceCache;
use crate::{Error, Result};

/// Get the path to the test fixtures directory.
pub fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

// ─────────────────────────────────────────────────────────────────────────────
// TestGen: Full pipeline builder
// ─────────────────────────────────────────────────────────────────────────────

enum Source {
    File(String),
    Inline(String),
}

/// Full-pipeline test builder.
///
/// Replaces boilerplate of loading schemas, creating source caches,
/// collecting documents, and calling `generate_from_input`.
///
/// # Examples
///
/// ```ignore
/// // From fixture files
/// let output = TestGen::new()
///     .schema("schemas/enum.graphql")
///     .options(PluginOptions { enums_as_types: Some(true), ..Default::default() })
///     .generate();
///
/// // From inline strings
/// let output = TestGen::new()
///     .schema_str("type Query { hello: String }")
///     .operations_str("query Hello { hello }")
///     .plugin("typescript-operations")
///     .generate();
/// ```
pub struct TestGen {
    schemas: Vec<Source>,
    operations: Vec<Source>,
    plugin: String,
    options: PluginOptions,
    preset: Preset,
    include_base_schema: bool,
}

impl TestGen {
    /// Create a new builder with sensible defaults.
    ///
    /// Defaults: `"typescript"` plugin, default preset, auto-includes `schemas/base.graphql`.
    pub fn new() -> Self {
        Self {
            schemas: Vec::new(),
            operations: Vec::new(),
            plugin: "typescript".to_string(),
            options: PluginOptions::default(),
            preset: Preset::default(),
            include_base_schema: true,
        }
    }

    /// Add a schema file path (relative to fixtures directory).
    pub fn schema(mut self, path: &str) -> Self {
        self.schemas.push(Source::File(path.to_string()));
        self
    }

    /// Add inline schema SDL.
    pub fn schema_str(mut self, sdl: &str) -> Self {
        self.schemas.push(Source::Inline(sdl.to_string()));
        self
    }

    /// Add an operations file path (relative to fixtures directory).
    pub fn operations(mut self, path: &str) -> Self {
        self.operations.push(Source::File(path.to_string()));
        self
    }

    /// Add inline GraphQL operations.
    pub fn operations_str(mut self, ops: &str) -> Self {
        self.operations.push(Source::Inline(ops.to_string()));
        self
    }

    /// Set the plugin name (default: `"typescript"`).
    pub fn plugin(mut self, name: &str) -> Self {
        self.plugin = name.to_string();
        self
    }

    /// Set plugin options.
    pub fn options(mut self, options: PluginOptions) -> Self {
        self.options = options;
        self
    }

    /// Set preset (default: `Preset::default()`).
    pub fn preset(mut self, preset: Preset) -> Self {
        self.preset = preset;
        self
    }

    /// Skip auto-including `schemas/base.graphql`.
    pub fn no_base_schema(mut self) -> Self {
        self.include_base_schema = false;
        self
    }

    /// Run generation and return the output string. Panics on error.
    pub fn generate(&self) -> String {
        let result = self.generate_result();
        assert_eq!(result.files.len(), 1, "expected exactly one output file");
        result.files[0].content.clone()
    }

    /// Run generation and return the full [`GenerateResult`]. Panics on error.
    pub fn generate_result(&self) -> GenerateResult {
        self.try_generate().expect("generation should succeed")
    }

    /// Run generation and return a [`Result`] for error testing.
    pub fn try_generate(&self) -> Result<GenerateResult> {
        let fixtures = fixtures_dir();

        // Build schema from file paths + inline strings
        let mut schema_files: Vec<(PathBuf, String)> = Vec::new();

        if self.include_base_schema {
            let base = fixtures.join("schemas/base.graphql");
            let content = std::fs::read_to_string(&base)
                .map_err(|e| Error::SchemaRead(base.clone(), e.to_string()))?;
            schema_files.push((base, content));
        }

        let mut inline_count = 0;
        for source in &self.schemas {
            match source {
                Source::File(path) => {
                    let full = fixtures.join(path);
                    let content = std::fs::read_to_string(&full)
                        .map_err(|e| Error::SchemaRead(full.clone(), e.to_string()))?;
                    schema_files.push((full, content));
                }
                Source::Inline(sdl) => {
                    schema_files.push((
                        PathBuf::from(format!("<inline-schema:{inline_count}>")),
                        sdl.clone(),
                    ));
                    inline_count += 1;
                }
            }
        }

        let schema = load_schema_from_contents(&schema_files)?;

        // Build source cache from file paths + inline operations
        let mut source_cache = SourceCache::new();
        let mut inline_doc_count = 0;

        for source in &self.operations {
            match source {
                Source::File(path) => {
                    let full = fixtures.join(path);
                    let content = std::fs::read_to_string(&full)
                        .map_err(|e| Error::FileRead(full.clone(), e.to_string()))?;
                    source_cache.push(full, content);
                }
                Source::Inline(ops) => {
                    // .graphql extension ensures collect_documents treats content as raw GraphQL
                    source_cache.push(
                        PathBuf::from(format!("<inline-ops:{inline_doc_count}>.graphql")),
                        ops.clone(),
                    );
                    inline_doc_count += 1;
                }
            }
        }

        let extract_config = ExtractConfig::default();
        let documents = collect_documents(&source_cache, &extract_config);

        let mut generates = HashMap::new();
        generates.insert(
            "output.ts".to_string(),
            OutputConfig {
                plugins: vec![PluginConfig::Name(self.plugin.clone())],
                config: Some(self.options.clone()),
                prelude: None,
                documents_only: false,
            },
        );

        let input = GenerateInput {
            schema: &schema,
            documents: &documents,
            generates: &generates,
            preset: self.preset,
        };

        generate_from_input(&input)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TestCtx: Lightweight context for unit-testing renderers
// ─────────────────────────────────────────────────────────────────────────────

const DEFAULT_SCHEMA: &str = "type Query { _: Boolean }";

/// Builder for [`TestCtx`].
///
/// # Examples
///
/// ```ignore
/// // One-shot — build + run in one step
/// let output = TestCtxBuilder::new()
///     .schema_str("enum Role { ADMIN USER }")
///     .run(|ctx| { generate_typescript(ctx) });
///
/// // With type helpers — build first
/// let ctx = TestCtxBuilder::new()
///     .schema_str("enum Role { ADMIN USER }")
///     .build();
/// let output = ctx.run(|gen_ctx| {
///     render_enum(gen_ctx, ctx.get_enum("Role"))?;
///     Ok(())
/// });
///
/// // From fixture file (no default schema)
/// let ctx = TestCtxBuilder::with_schema("schemas/enum.graphql").build();
/// ```
pub struct TestCtxBuilder {
    schemas: Vec<Source>,
    options: PluginOptions,
    include_default_schema: bool,
}

impl Default for TestCtxBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TestCtxBuilder {
    /// Start with a default minimal schema (`type Query { _: Boolean }`).
    ///
    /// Use `.schema_str()` to add your types on top.
    pub fn new() -> Self {
        Self {
            schemas: Vec::new(),
            options: PluginOptions::default(),
            include_default_schema: true,
        }
    }

    /// Start with a schema file (relative to fixtures directory).
    /// Does NOT include the default schema.
    pub fn with_schema(path: &str) -> Self {
        Self {
            schemas: vec![Source::File(path.to_string())],
            options: PluginOptions::default(),
            include_default_schema: false,
        }
    }

    /// Start with inline schema SDL.
    /// Does NOT include the default schema — you provide the full schema.
    pub fn with_schema_str(sdl: &str) -> Self {
        Self {
            schemas: vec![Source::Inline(sdl.to_string())],
            options: PluginOptions::default(),
            include_default_schema: false,
        }
    }

    /// Set plugin options.
    pub fn options(mut self, options: PluginOptions) -> Self {
        self.options = options;
        self
    }

    /// Add an additional schema file (relative to fixtures directory).
    pub fn schema(mut self, path: &str) -> Self {
        self.schemas.push(Source::File(path.to_string()));
        self
    }

    /// Add additional inline schema SDL (merged with existing schemas).
    pub fn schema_str(mut self, sdl: &str) -> Self {
        self.schemas.push(Source::Inline(sdl.to_string()));
        self
    }

    /// Build and immediately run a closure. Shortcut for `build().run(f)`.
    pub fn run<F>(self, f: F) -> String
    where
        F: FnOnce(&mut GeneratorContext) -> Result<()>,
    {
        self.build().run(f)
    }

    /// Build the test context. Use this when you need [`TestCtx::get_enum`] etc.
    pub fn build(self) -> TestCtx {
        let fixtures = fixtures_dir();
        let mut schema_files: Vec<(PathBuf, String)> = Vec::new();
        let mut inline_count = 0;

        if self.include_default_schema {
            schema_files.push((
                PathBuf::from("<default-schema>"),
                DEFAULT_SCHEMA.to_string(),
            ));
        }

        for source in &self.schemas {
            match source {
                Source::File(path) => {
                    let full = fixtures.join(path);
                    let content = std::fs::read_to_string(&full)
                        .unwrap_or_else(|e| panic!("failed to read {}: {e}", full.display()));
                    schema_files.push((full, content));
                }
                Source::Inline(sdl) => {
                    schema_files.push((
                        PathBuf::from(format!("<inline-schema:{inline_count}>")),
                        sdl.clone(),
                    ));
                    inline_count += 1;
                }
            }
        }

        let schema =
            load_schema_from_contents(&schema_files).expect("schema should parse and validate");

        TestCtx {
            schema,
            options: self.options,
        }
    }
}

/// Built test context with a validated schema.
///
/// Use [`TestCtx::run`] to execute a renderer, and the `get_*` helpers
/// to extract specific types from the schema.
pub struct TestCtx {
    pub schema: Valid<Schema>,
    pub options: PluginOptions,
}

impl TestCtx {
    /// Run a closure with a [`GeneratorContext`] and return the generated output.
    ///
    /// Creates empty operations/fragments — use [`TestGen`] if you need operations.
    pub fn run<F>(&self, f: F) -> String
    where
        F: FnOnce(&mut GeneratorContext) -> Result<()>,
    {
        let operations = IndexMap::new();
        let fragments = IndexMap::new();
        let mut buffer = Vec::new();

        let mut ctx = GeneratorContext {
            schema: &self.schema,
            operations: &operations,
            fragments: &fragments,
            options: &self.options,
            writer: &mut buffer,
        };

        f(&mut ctx).expect("renderer should not error");

        String::from_utf8(buffer).expect("output should be valid UTF-8")
    }

    /// Get an enum type from the schema by name. Panics if not found.
    pub fn get_enum(&self, name: &str) -> &Node<EnumType> {
        match self.schema.types.get(name) {
            Some(ExtendedType::Enum(e)) => e,
            _ => panic!("enum type '{name}' not found in schema"),
        }
    }

    /// Get an object type from the schema by name. Panics if not found.
    pub fn get_object(&self, name: &str) -> &Node<ObjectType> {
        match self.schema.types.get(name) {
            Some(ExtendedType::Object(o)) => o,
            _ => panic!("object type '{name}' not found in schema"),
        }
    }

    /// Get an interface type from the schema by name. Panics if not found.
    pub fn get_interface(&self, name: &str) -> &Node<InterfaceType> {
        match self.schema.types.get(name) {
            Some(ExtendedType::Interface(i)) => i,
            _ => panic!("interface type '{name}' not found in schema"),
        }
    }

    /// Get a union type from the schema by name. Panics if not found.
    pub fn get_union(&self, name: &str) -> &Node<UnionType> {
        match self.schema.types.get(name) {
            Some(ExtendedType::Union(u)) => u,
            _ => panic!("union type '{name}' not found in schema"),
        }
    }

    /// Get an input object type from the schema by name. Panics if not found.
    pub fn get_input(&self, name: &str) -> &Node<InputObjectType> {
        match self.schema.types.get(name) {
            Some(ExtendedType::InputObject(i)) => i,
            _ => panic!("input type '{name}' not found in schema"),
        }
    }

    /// Get a scalar type from the schema by name. Panics if not found.
    pub fn get_scalar(&self, name: &str) -> &Node<ScalarType> {
        match self.schema.types.get(name) {
            Some(ExtendedType::Scalar(s)) => s,
            _ => panic!("scalar type '{name}' not found in schema"),
        }
    }
}
