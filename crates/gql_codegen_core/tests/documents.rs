//! Integration tests for document loading and parsing

use std::path::PathBuf;

use apollo_compiler::Name;
use gql_codegen_core::{
    ExtractConfig, SourceCache, StringOrArray, collect_documents, load_sources,
};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
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
