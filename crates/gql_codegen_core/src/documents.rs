//! Zero-copy document discovery and parsing with parallel processing
//!
//! Uses rayon for parallel file reading and document extraction.
//! All source files are loaded into a cache, then operations/fragments
//! borrow from that cache - no redundant string allocations.

use std::path::{Path, PathBuf};

use apollo_compiler::Name;
use apollo_compiler::ast::Definition;
use globset::{Glob, GlobSetBuilder};
use indexmap::IndexMap;
use rayon::prelude::*;
use walkdir::WalkDir;

use crate::config::StringOrArray;
use crate::error::{Error, Result};
use crate::extract::{self, ExtractConfig, Extracted};

/// Cache of source file contents - keeps sources alive for borrowing
#[derive(Debug, Default)]
pub struct SourceCache {
    /// (file path, file contents) pairs
    files: Vec<(PathBuf, String)>,
}

impl SourceCache {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    /// Add pre-loaded content to the cache
    fn push(&mut self, path: PathBuf, content: String) -> usize {
        let idx = self.files.len();
        self.files.push((path, content));
        idx
    }

    /// Get a reference to a loaded file
    #[inline]
    pub fn get(&self, idx: usize) -> Option<(&Path, &str)> {
        self.files.get(idx).map(|(p, c)| (p.as_path(), c.as_str()))
    }

    /// Iterate over all loaded files
    pub fn iter(&self) -> impl Iterator<Item = (usize, &Path, &str)> {
        self.files
            .iter()
            .enumerate()
            .map(|(i, (p, c))| (i, p.as_path(), c.as_str()))
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

/// A parsed GraphQL operation with metadata (zero-copy text)
#[derive(Debug, Clone)]
pub struct ParsedOperation<'a> {
    /// The operation AST (owned by apollo-compiler)
    pub definition: apollo_compiler::ast::OperationDefinition,
    /// Index into SourceCache
    pub source_idx: usize,
    /// Slice into the source file's GraphQL text
    pub text: &'a str,
    /// Location in source file
    pub line: usize,
    pub column: usize,
}

impl<'a> ParsedOperation<'a> {
    /// Get the file path from the cache
    pub fn file_path<'c>(&self, cache: &'c SourceCache) -> &'c Path {
        cache.get(self.source_idx).map(|(p, _)| p).unwrap()
    }
}

/// A parsed GraphQL fragment with metadata (zero-copy text)
#[derive(Debug, Clone)]
pub struct ParsedFragment<'a> {
    /// The fragment AST
    pub definition: apollo_compiler::ast::FragmentDefinition,
    /// Index into SourceCache
    pub source_idx: usize,
    /// Slice into the source file's GraphQL text
    pub text: &'a str,
    /// Location in source file
    pub line: usize,
    pub column: usize,
}

impl<'a> ParsedFragment<'a> {
    /// Get the file path from the cache
    pub fn file_path<'c>(&self, cache: &'c SourceCache) -> &'c Path {
        cache.get(self.source_idx).map(|(p, _)| p).unwrap()
    }
}

/// Result of collecting all documents
#[derive(Debug, Default)]
pub struct CollectedDocuments<'a> {
    pub operations: IndexMap<Name, ParsedOperation<'a>>,
    pub fragments: IndexMap<Name, ParsedFragment<'a>>,
    /// Warnings encountered during collection (non-fatal)
    pub warnings: Vec<String>,
}

/// Load all matching files into the source cache (parallel file reading)
pub fn load_sources(
    patterns: &StringOrArray,
    base_dir: Option<&Path>,
    cache: &mut SourceCache,
) -> Result<()> {
    let base = base_dir.unwrap_or(Path::new("."));
    let pattern_strs = patterns.as_vec();

    // Build glob set
    let mut builder = GlobSetBuilder::new();
    for pattern in &pattern_strs {
        let glob = Glob::new(pattern).map_err(|e| Error::InvalidGlob {
            pattern: pattern.to_string(),
            message: e.to_string(),
        })?;
        builder.add(glob);
    }
    let glob_set = builder.build().map_err(|e| Error::InvalidGlob {
        pattern: pattern_strs.join(", "),
        message: e.to_string(),
    })?;

    // Phase 1: Walk directory and collect matching paths (sequential - fast)
    let paths: Vec<PathBuf> = WalkDir::new(base)
        .into_iter()
        .filter_entry(|e| !is_ignored(e))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            let path = e.path();
            let relative = path.strip_prefix(base).unwrap_or(path);
            glob_set.is_match(relative)
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    // Phase 2: Read files in parallel
    let contents: Vec<_> = paths
        .par_iter()
        .map(|path| {
            let content = std::fs::read_to_string(path);
            (path.clone(), content)
        })
        .collect();

    // Phase 3: Add to cache (sequential - maintains order, handles errors)
    for (path, content_result) in contents {
        match content_result {
            Ok(content) => {
                cache.push(path, content);
            }
            Err(e) => {
                return Err(Error::FileRead(path, e.to_string()));
            }
        }
    }

    Ok(())
}

/// Intermediate result from parallel extraction
struct ExtractedDoc<'a> {
    source_idx: usize,
    path: &'a Path,
    text: &'a str,
    line: usize,
    column: usize,
}

/// Extract and parse all GraphQL documents from loaded sources (parallel)
pub fn collect_documents<'a>(
    cache: &'a SourceCache,
    extract_config: &ExtractConfig,
) -> CollectedDocuments<'a> {
    // Phase 1: Extract GraphQL from all files in parallel
    let extracted: Vec<ExtractedDoc<'a>> = cache
        .iter()
        .collect::<Vec<_>>()
        .par_iter()
        .flat_map_iter(|(idx, path, source)| {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

            let docs: Vec<Extracted<'a>> = match ext {
                "graphql" | "gql" => {
                    vec![Extracted {
                        text: source,
                        line: 1,
                        column: 1,
                    }]
                }
                "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs" | "mts" | "cts" => {
                    extract::extract(source, extract_config)
                }
                _ => vec![],
            };

            docs.into_iter().map(move |doc| ExtractedDoc {
                source_idx: *idx,
                path,
                text: doc.text,
                line: doc.line,
                column: doc.column,
            })
        })
        .collect();

    // Phase 2: Parse all documents in parallel
    let parsed: Vec<ParseResult<'a>> = extracted
        .par_iter()
        .map(|doc| parse_document(doc))
        .collect();

    // Phase 3: Merge results (sequential - handles duplicates)
    let mut result = CollectedDocuments::default();

    for parse_result in parsed {
        match parse_result {
            ParseResult::Success {
                operations,
                fragments,
            } => {
                for (name, op) in operations {
                    if result.operations.contains_key(&name) {
                        result
                            .warnings
                            .push(format!("Duplicate operation '{name}' (skipped)"));
                    } else {
                        result.operations.insert(name, op);
                    }
                }
                for (name, frag) in fragments {
                    if result.fragments.contains_key(&name) {
                        result
                            .warnings
                            .push(format!("Duplicate fragment '{name}' (skipped)"));
                    } else {
                        result.fragments.insert(name, frag);
                    }
                }
            }
            ParseResult::Error(warning) => {
                result.warnings.push(warning);
            }
        }
    }

    result
}

enum ParseResult<'a> {
    Success {
        operations: Vec<(Name, ParsedOperation<'a>)>,
        fragments: Vec<(Name, ParsedFragment<'a>)>,
    },
    Error(String),
}

fn parse_document<'a>(doc: &ExtractedDoc<'a>) -> ParseResult<'a> {
    let document = match apollo_compiler::ast::Document::parse(doc.text, doc.path) {
        Ok(d) => d,
        Err(e) => {
            return ParseResult::Error(format!("{}:{}: {}", doc.path.display(), doc.line, e));
        }
    };

    let mut operations = Vec::new();
    let mut fragments = Vec::new();
    let mut anon_count = 0;

    for definition in document.definitions {
        match definition {
            Definition::OperationDefinition(op) => {
                let name = match &op.name {
                    Some(n) => n.clone(),
                    None => {
                        anon_count += 1;
                        Name::new(&format!("Anonymous_{anon_count}")).expect("valid name")
                    }
                };

                // Extract just this operation's text using source location
                let text = extract_definition_text(doc.text, op.location());

                operations.push((
                    name,
                    ParsedOperation {
                        definition: (*op).clone(),
                        source_idx: doc.source_idx,
                        text,
                        line: doc.line,
                        column: doc.column,
                    },
                ));
            }

            Definition::FragmentDefinition(frag) => {
                let name = frag.name.clone();

                // Extract just this fragment's text using source location
                let text = extract_definition_text(doc.text, frag.location());

                fragments.push((
                    name,
                    ParsedFragment {
                        definition: (*frag).clone(),
                        source_idx: doc.source_idx,
                        text,
                        line: doc.line,
                        column: doc.column,
                    },
                ));
            }

            _ => {}
        }
    }

    ParseResult::Success {
        operations,
        fragments,
    }
}

/// Extract the text for a single definition using its source location.
/// Falls back to the full text if location info is unavailable.
fn extract_definition_text(
    full_text: &str,
    location: Option<apollo_compiler::parser::SourceSpan>,
) -> &str {
    match location {
        Some(span) => {
            let start = span.offset();
            let end = span.end_offset();
            // Safety: apollo-compiler guarantees valid UTF-8 boundaries
            &full_text[start..end]
        }
        None => full_text,
    }
}

/// Check if a directory entry should be skipped
fn is_ignored(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.') || s == "node_modules" || s == "target")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures")
    }

    #[test]
    fn test_source_cache() {
        let mut cache = SourceCache::new();
        cache
            .files
            .push((PathBuf::from("test.graphql"), "query { user }".to_string()));

        let (path, content) = cache.get(0).unwrap();
        assert_eq!(path, Path::new("test.graphql"));
        assert_eq!(content, "query { user }");
    }

    #[test]
    fn test_zero_copy_text() {
        let mut cache = SourceCache::new();
        let source = "query GetUser { user { id } }".to_string();
        let source_ptr = source.as_ptr();
        cache.files.push((PathBuf::from("test.graphql"), source));

        let (_, content) = cache.get(0).unwrap();
        assert_eq!(content.as_ptr(), source_ptr);
    }

    #[test]
    fn test_load_sources_graphql_files() {
        let mut cache = SourceCache::new();
        let patterns = StringOrArray::Single("documents/*.graphql".into());

        load_sources(&patterns, Some(&fixtures_dir()), &mut cache).unwrap();

        assert_eq!(cache.len(), 2); // queries.graphql and fragments.graphql
    }

    #[test]
    fn test_load_sources_tsx_files() {
        let mut cache = SourceCache::new();
        let patterns = StringOrArray::Single("documents/*.tsx".into());

        load_sources(&patterns, Some(&fixtures_dir()), &mut cache).unwrap();

        assert_eq!(cache.len(), 2); // component.tsx and broken.tsx
    }

    #[test]
    fn test_load_sources_multiple_patterns() {
        let mut cache = SourceCache::new();
        let patterns =
            StringOrArray::Multiple(vec!["documents/*.graphql".into(), "documents/*.tsx".into()]);

        load_sources(&patterns, Some(&fixtures_dir()), &mut cache).unwrap();

        assert_eq!(cache.len(), 4);
    }

    #[test]
    fn test_collect_documents_from_graphql() {
        let mut cache = SourceCache::new();
        let patterns = StringOrArray::Single("documents/queries.graphql".into());
        load_sources(&patterns, Some(&fixtures_dir()), &mut cache).unwrap();

        let docs = collect_documents(&cache, &ExtractConfig::default());

        assert_eq!(docs.operations.len(), 3); // GetUser, GetUsers, CreateUser
        assert!(docs.operations.contains_key(&Name::new("GetUser").unwrap()));
        assert!(
            docs.operations
                .contains_key(&Name::new("GetUsers").unwrap())
        );
        assert!(
            docs.operations
                .contains_key(&Name::new("CreateUser").unwrap())
        );
    }

    #[test]
    fn test_collect_documents_with_fragments() {
        let mut cache = SourceCache::new();
        let patterns = StringOrArray::Single("documents/fragments.graphql".into());
        load_sources(&patterns, Some(&fixtures_dir()), &mut cache).unwrap();

        let docs = collect_documents(&cache, &ExtractConfig::default());

        assert_eq!(docs.fragments.len(), 2); // UserFields, PostFields
        assert!(
            docs.fragments
                .contains_key(&Name::new("UserFields").unwrap())
        );
        assert!(
            docs.fragments
                .contains_key(&Name::new("PostFields").unwrap())
        );

        assert_eq!(docs.operations.len(), 1); // GetUserWithFragments
    }

    #[test]
    fn test_collect_documents_from_tsx() {
        let mut cache = SourceCache::new();
        let patterns = StringOrArray::Single("documents/component.tsx".into());
        load_sources(&patterns, Some(&fixtures_dir()), &mut cache).unwrap();

        let docs = collect_documents(&cache, &ExtractConfig::default());

        // Should extract gql`` and /* GraphQL */ tagged queries
        // Should NOT extract the untagged template literal
        assert_eq!(docs.operations.len(), 2);
        assert!(
            docs.operations
                .contains_key(&Name::new("GetUserFromTsx").unwrap())
        );
        assert!(
            docs.operations
                .contains_key(&Name::new("GetPostsFromTsx").unwrap())
        );
    }

    #[test]
    fn test_collect_documents_from_broken_tsx() {
        let mut cache = SourceCache::new();
        let patterns = StringOrArray::Single("documents/broken.tsx".into());
        load_sources(&patterns, Some(&fixtures_dir()), &mut cache).unwrap();

        let docs = collect_documents(&cache, &ExtractConfig::default());

        // Should still extract GraphQL despite broken JS syntax
        assert_eq!(docs.operations.len(), 2);
        assert!(
            docs.operations
                .contains_key(&Name::new("StillExtractable").unwrap())
        );
        assert!(
            docs.operations
                .contains_key(&Name::new("AlsoExtractable").unwrap())
        );
    }

    #[test]
    fn test_zero_copy_document_text() {
        let mut cache = SourceCache::new();
        let patterns = StringOrArray::Single("documents/queries.graphql".into());
        load_sources(&patterns, Some(&fixtures_dir()), &mut cache).unwrap();

        let (_, source) = cache.get(0).unwrap();
        let source_start = source.as_ptr();
        let source_end = unsafe { source_start.add(source.len()) };

        let docs = collect_documents(&cache, &ExtractConfig::default());

        // All operation texts should point into the cached source
        for (_, op) in &docs.operations {
            let text_ptr = op.text.as_ptr();
            assert!(
                text_ptr >= source_start && text_ptr < source_end,
                "operation text should be zero-copy slice into source"
            );
        }
    }

    #[test]
    fn test_parallel_load_many_files() {
        // Test that parallel loading works with multiple files
        let mut cache = SourceCache::new();
        let patterns = StringOrArray::Multiple(vec![
            "documents/*.graphql".into(),
            "documents/*.tsx".into(),
            "schemas/*.graphql".into(),
        ]);

        load_sources(&patterns, Some(&fixtures_dir()), &mut cache).unwrap();

        // Should have loaded all matching files
        assert!(cache.len() >= 6);
    }
}
