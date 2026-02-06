//! Zero-copy document discovery and parsing with parallel processing

use std::fmt;
use std::path::{Path, PathBuf};

use apollo_compiler::Name;
use apollo_compiler::ast::Definition;
use apollo_compiler::validation::DiagnosticList;
use globset::{Glob, GlobSetBuilder};
use indexmap::IndexMap;
use rayon::prelude::*;

use crate::config::StringOrArray;
use crate::error::{Error, Result};
use crate::extract::{self, ExtractConfig, Extracted};
use crate::source_cache::SourceCache;

/// Structured warning from document collection (non-fatal)
#[derive(Debug, Clone)]
pub enum DocumentWarning {
    /// Parse errors with full diagnostic info (rendered through our pipeline)
    ParseErrors(DiagnosticList),
    DuplicateName { kind: &'static str, name: String },
    /// Conflicting or redundant config options
    ConfigConflict { message: String },
}

impl fmt::Display for DocumentWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DocumentWarning::ParseErrors(diagnostics) => {
                write!(f, "{} parse error(s)", diagnostics.len())
            }
            DocumentWarning::DuplicateName { kind, name } => {
                write!(f, "Duplicate {kind} '{name}' (skipped)")
            }
            DocumentWarning::ConfigConflict { message } => {
                write!(f, "{message}")
            }
        }
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
        // source_idx is always assigned from cache.iter() during collection
        cache
            .get(self.source_idx)
            .map(|(p, _)| p)
            .expect("source_idx should be valid — assigned from SourceCache during collection")
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
        cache
            .get(self.source_idx)
            .map(|(p, _)| p)
            .expect("source_idx should be valid — assigned from SourceCache during collection")
    }
}

/// Result of collecting all documents
#[derive(Debug, Default)]
pub struct CollectedDocuments<'a> {
    pub operations: IndexMap<Name, ParsedOperation<'a>>,
    pub fragments: IndexMap<Name, ParsedFragment<'a>>,
    /// Warnings encountered during collection (non-fatal)
    pub warnings: Vec<DocumentWarning>,
}

/// Load all matching files into the source cache (parallel file reading)
///
/// This expands globs internally. If you've already expanded globs (e.g., for caching),
/// use [`load_sources_from_paths`] instead to avoid duplicate work.
pub fn load_sources(
    patterns: &StringOrArray,
    base_dir: Option<&Path>,
    cache: &mut SourceCache,
) -> Result<()> {
    let base = base_dir.unwrap_or(Path::new("."));
    let paths = expand_document_globs(patterns, base)?;
    load_sources_from_paths(&paths, cache)
}

/// Load files from pre-resolved paths
pub fn load_sources_from_paths(paths: &[PathBuf], cache: &mut SourceCache) -> Result<()> {
    let contents: Vec<_> = paths
        .par_iter()
        .map(|path| {
            let content = std::fs::read_to_string(path);
            (path.clone(), content)
        })
        .collect();

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

/// Expand glob patterns to matching file paths
///
/// Handles both relative patterns (resolved against base_dir) and absolute patterns.
/// Supports negation patterns with `!` prefix:
/// ```ignore
/// ["src/**/*.tsx", "!src/__generated__/**", "!**/*.test.tsx"]
/// ```
pub fn expand_document_globs(patterns: &StringOrArray, base_dir: &Path) -> Result<Vec<PathBuf>> {
    use std::collections::HashSet;

    let pattern_strs = patterns.as_vec();

    let mut include_patterns = Vec::new();
    let mut exclude_builder = GlobSetBuilder::new();
    let mut has_excludes = false;

    for pattern in &pattern_strs {
        if let Some(negated) = pattern.strip_prefix('!') {
            // Build exclude set for filtering later
            let glob = Glob::new(negated).map_err(|e| Error::InvalidGlob {
                pattern: pattern.to_string(),
                message: e.to_string(),
            })?;
            exclude_builder.add(glob);
            has_excludes = true;
        } else {
            include_patterns.push(*pattern);
        }
    }

    if include_patterns.is_empty() {
        return Ok(Vec::new());
    }

    let exclude_set = if has_excludes {
        Some(exclude_builder.build().map_err(|e| Error::InvalidGlob {
            pattern: pattern_strs.join(", "),
            message: e.to_string(),
        })?)
    } else {
        None
    };

    // Use glob crate which handles extracting walk root from patterns
    let mut all_paths = HashSet::new();

    for pattern in include_patterns {
        // If pattern is relative, join with base_dir
        let full_pattern = if Path::new(pattern).is_absolute() {
            pattern.to_string()
        } else {
            base_dir.join(pattern).to_string_lossy().to_string()
        };

        let glob_paths = glob::glob(&full_pattern).map_err(|e| Error::InvalidGlob {
            pattern: pattern.to_string(),
            message: e.to_string(),
        })?;

        for entry in glob_paths {
            match entry {
                Ok(path) => {
                    // Skip ignored directories
                    if path.components().any(|c| {
                        matches!(
                            c.as_os_str().to_str(),
                            Some("node_modules" | ".git" | "target" | "__generated__")
                        )
                    }) {
                        continue;
                    }

                    // Apply exclude patterns
                    if let Some(ref excludes) = exclude_set {
                        if excludes.is_match(&path) {
                            continue;
                        }
                    }

                    if path.is_file() {
                        all_paths.insert(path);
                    }
                }
                Err(_) => continue, // Skip unreadable paths
            }
        }
    }

    Ok(all_paths.into_iter().collect())
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
                        result.warnings.push(DocumentWarning::DuplicateName {
                            kind: "operation",
                            name: name.to_string(),
                        });
                    } else {
                        result.operations.insert(name, op);
                    }
                }
                for (name, frag) in fragments {
                    if result.fragments.contains_key(&name) {
                        result.warnings.push(DocumentWarning::DuplicateName {
                            kind: "fragment",
                            name: name.to_string(),
                        });
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
    Error(DocumentWarning),
}

fn parse_document<'a>(doc: &ExtractedDoc<'a>) -> ParseResult<'a> {
    let document = match apollo_compiler::ast::Document::parse(doc.text, doc.path) {
        Ok(d) => d,
        Err(e) => {
            return ParseResult::Error(DocumentWarning::ParseErrors(e.errors));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_cache::SourceCache;

    #[test]
    fn test_source_cache() {
        let mut cache = SourceCache::new();
        cache.push(PathBuf::from("test.graphql"), "query { user }".to_string());

        let (path, content) = cache.get(0).unwrap();
        assert_eq!(path, Path::new("test.graphql"));
        assert_eq!(content, "query { user }");
    }

    #[test]
    fn test_zero_copy_text() {
        let mut cache = SourceCache::new();
        let source = "query GetUser { user { id } }".to_string();
        let source_ptr = source.as_ptr();
        cache.push(PathBuf::from("test.graphql"), source);

        let (_, content) = cache.get(0).unwrap();
        assert_eq!(content.as_ptr(), source_ptr);
    }
}
