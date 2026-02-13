//! Edge-case tests for list types and nullability in typescript-operations.
//!
//! Covers all 4 nullability combos ([T!]!, [T]!, [T!], [T]) across:
//! - Scalar fields
//! - Object fields (with sub-selections)
//! - Variant fields (unions/interfaces with inline fragments)

use gql_codegen_core::PluginOptions;
use gql_codegen_core::test_utils::TestGen;

/// Helper: generate typescript-operations from inline schema + query
fn gen_ops(schema: &str, query: &str) -> String {
    TestGen::new()
        .no_base_schema()
        .schema_str(schema)
        .operations_str(query)
        .plugin("typescript-operations")
        .options(PluginOptions::default())
        .generate()
}

struct FieldType {
    optional: bool,
    type_expr: String,
}

/// Extract the type annotation for a given field name from generated output.
///
/// Given output like:
///   readonly items?: ReadonlyArray<{ ... } | null> | null;
///
/// `extract_field(&output, "items")` returns:
///   FieldType { optional: true, type_expr: "ReadonlyArray<{ ... } | null> | null" }
///
/// Handles multi-line types by tracking `{}`/`<>` depth.
fn extract_field(output: &str, field_name: &str) -> FieldType {
    let mut lines = output.lines();

    let first_line = lines
        .by_ref()
        .find(|line| {
            let trimmed = line.trim();
            trimmed.contains(field_name)
                && (trimmed.contains(&format!("{field_name}:"))
                    || trimmed.contains(&format!("{field_name}?:")))
        })
        .unwrap_or_else(|| panic!("field '{field_name}' not found in output"));

    let optional = first_line.contains(&format!("{field_name}?:"));
    let after_colon = first_line.split_once(": ").unwrap().1.trim();

    if after_colon.ends_with(';') {
        return FieldType {
            optional,
            type_expr: after_colon.trim_end_matches(';').to_string(),
        };
    }

    let mut result = after_colon.to_string();
    let mut depth = bracket_depth(after_colon);

    for line in lines {
        result.push('\n');
        result.push_str(line);
        depth += bracket_depth(line);
        if depth == 0 && line.trim_end().ends_with(';') {
            break;
        }
    }

    FieldType {
        optional,
        type_expr: result.trim_end_matches(';').to_string(),
    }
}

fn bracket_depth(s: &str) -> i32 {
    s.chars()
        .map(|c| match c {
            '{' | '<' => 1,
            '}' | '>' => -1,
            _ => 0,
        })
        .sum()
}

// ── scalar lists ─────────────────────────────────────────────────

const SCALAR_LIST_SCHEMA: &str = "\
type Query {
  both_non_null: [String!]!
  nullable_inner: [String]!
  nullable_outer: [String!]
  both_nullable: [String]
}
";

const SCALAR_LIST_QUERY: &str = "query ScalarLists {
  both_non_null
  nullable_inner
  nullable_outer
  both_nullable
}";

#[test]
fn scalar_list_both_non_null() {
    let output = gen_ops(SCALAR_LIST_SCHEMA, SCALAR_LIST_QUERY);
    let field = extract_field(&output, "both_non_null");
    assert_eq!(field.type_expr, "ReadonlyArray<string>");
    assert!(!field.optional);
}

#[test]
fn scalar_list_nullable_inner() {
    let output = gen_ops(SCALAR_LIST_SCHEMA, SCALAR_LIST_QUERY);
    let field = extract_field(&output, "nullable_inner");
    assert_eq!(field.type_expr, "ReadonlyArray<string | null | undefined>");
    assert!(!field.optional);
}

#[test]
fn scalar_list_nullable_outer() {
    let output = gen_ops(SCALAR_LIST_SCHEMA, SCALAR_LIST_QUERY);
    let field = extract_field(&output, "nullable_outer");
    assert_eq!(field.type_expr, "ReadonlyArray<string> | null | undefined");
    assert!(field.optional);
}

#[test]
fn scalar_list_both_nullable() {
    let output = gen_ops(SCALAR_LIST_SCHEMA, SCALAR_LIST_QUERY);
    let field = extract_field(&output, "both_nullable");
    assert_eq!(
        field.type_expr,
        "ReadonlyArray<string | null | undefined> | null | undefined"
    );
    assert!(field.optional);
}

// ── object lists ─────────────────────────────────────────────────

const OBJECT_LIST_SCHEMA: &str = "\
type Query {
  both_non_null: [Item!]!
  nullable_inner: [Item]!
  nullable_outer: [Item!]
  both_nullable: [Item]
}

type Item { id: ID!, name: String! }
";

const OBJECT_LIST_QUERY: &str = "\
query ObjectLists {
  both_non_null { id name }
  nullable_inner { id name }
  nullable_outer { id name }
  both_nullable { id name }
}
";

#[test]
fn object_list_both_non_null() {
    let output = gen_ops(OBJECT_LIST_SCHEMA, OBJECT_LIST_QUERY);
    let field = extract_field(&output, "both_non_null");
    assert!(!field.type_expr.contains("null"));
    assert!(!field.optional);
}

#[test]
fn object_list_nullable_inner() {
    let output = gen_ops(OBJECT_LIST_SCHEMA, OBJECT_LIST_QUERY);
    let field = extract_field(&output, "nullable_inner");
    // Inner null: element is nullable inside the array
    assert!(field.type_expr.contains("} | null"));
    // Outer non-null: no null after >
    assert!(!field.type_expr.ends_with("> | null"));
    assert!(!field.optional);
}

#[test]
fn object_list_nullable_outer() {
    let output = gen_ops(OBJECT_LIST_SCHEMA, OBJECT_LIST_QUERY);
    let field = extract_field(&output, "nullable_outer");
    // Outer null: array itself is nullable
    assert!(field.type_expr.ends_with("> | null | undefined"));
    // Inner non-null: no null inside the array
    assert!(!field.type_expr.contains("} | null | undefined"));
    assert!(field.optional);
}

#[test]
fn object_list_both_nullable() {
    let output = gen_ops(OBJECT_LIST_SCHEMA, OBJECT_LIST_QUERY);
    let field = extract_field(&output, "both_nullable");
    // Inner null
    assert!(field.type_expr.contains("} | null | undefined"));
    // Outer null
    assert!(field.type_expr.ends_with("> | null | undefined"));
    assert!(field.optional);
}

// ── variant lists (unions) ───────────────────────────────────────

const UNION_LIST_SCHEMA: &str = "\
type Query {
  both_non_null: [SearchResult!]!
  nullable_inner: [SearchResult]!
  nullable_outer: [SearchResult!]
  both_nullable: [SearchResult]
}
union SearchResult = Book | Movie
type Book { isbn: String! }
type Movie { imdbId: String! }
";

fn union_query(field: &str) -> String {
    format!("query UnionList {{ {field} {{ ... on Book {{ isbn }} ... on Movie {{ imdbId }} }} }}")
}

#[test]
fn variant_list_both_non_null() {
    let output = gen_ops(UNION_LIST_SCHEMA, &union_query("both_non_null"));
    assert!(output.contains("ReadonlyArray<"));
    assert!(output.contains("'Book'"));
    assert!(output.contains("'Movie'"));
    let field = extract_field(&output, "both_non_null");
    assert!(!field.type_expr.contains("| null | undefined"));
    assert!(!field.optional);
}

#[test]
fn variant_list_nullable_inner() {
    let output = gen_ops(UNION_LIST_SCHEMA, &union_query("nullable_inner"));
    let field = extract_field(&output, "nullable_inner");
    // Inner null: | null appears inside the array (before >)
    assert!(field.type_expr.contains("| null | undefined"));
    // Outer non-null: doesn't end with > | null
    assert!(!field.type_expr.ends_with("> | null | undefined"));
    assert!(!field.optional);
}

#[test]
fn variant_list_nullable_outer() {
    let output = gen_ops(UNION_LIST_SCHEMA, &union_query("nullable_outer"));
    let field = extract_field(&output, "nullable_outer");
    // Outer null: ends with > | null
    assert!(field.type_expr.ends_with("> | null | undefined"));
    assert!(field.optional);
}

#[test]
fn variant_list_both_nullable() {
    let output = gen_ops(UNION_LIST_SCHEMA, &union_query("both_nullable"));
    let field = extract_field(&output, "both_nullable");
    assert!(
        field.type_expr.ends_with("> | null | undefined"),
        "outer null missing"
    );
    // Should also have inner null (before the closing >)
    let inner = &field.type_expr[..field.type_expr.rfind('>').unwrap()];
    assert!(inner.contains("| null | undefined"), "inner null missing");
    assert!(field.optional);
}

// ── non-list nullability (baseline) ──────────────────────────────

#[test]
fn non_list_nullable_object() {
    let schema = "type Query { item: Item }\ntype Item { id: ID! }";
    let query = "query Q { item { id } }";
    let output = gen_ops(schema, query);
    let field = extract_field(&output, "item");
    assert!(
        field.type_expr.contains("| null"),
        "nullable object field should have | null"
    );
    assert!(field.optional);
}

#[test]
fn non_list_non_null_object() {
    let schema = "type Query { item: Item! }\ntype Item { id: ID! }";
    let query = "query Q { item { id } }";
    let output = gen_ops(schema, query);
    let field = extract_field(&output, "item");
    assert!(
        !field.type_expr.contains("null"),
        "non-null object field should not have null"
    );
    assert!(!field.optional);
}

// ── optional property markers ────────────────────────────────────

#[test]
fn nullable_list_field_has_optional_marker() {
    let schema = "type Query { items: [String!] }";
    let query = "query Q { items }";
    let output = gen_ops(schema, query);
    let field = extract_field(&output, "items");
    assert!(field.optional, "nullable list should have ? marker");
}

#[test]
fn non_null_list_field_no_optional_marker() {
    let schema = "type Query { items: [String!]! }";
    let query = "query Q { items }";
    let output = gen_ops(schema, query);
    let field = extract_field(&output, "items");
    assert!(!field.optional, "non-null list should not have ? marker");
}

// ── nested lists ─────────────────────────────────────────────────

#[test]
fn nested_scalar_list_all_non_null() {
    // [[String!]!]!
    let schema = "type Query { matrix: [[String!]!]! }";
    let query = "query Q { matrix }";
    let output = gen_ops(schema, query);
    let field = extract_field(&output, "matrix");
    assert_eq!(field.type_expr, "ReadonlyArray<ReadonlyArray<string>>");
    assert!(!field.optional);
}

#[test]
fn nested_scalar_list_nullable_element() {
    // [[String]!]!
    let schema = "type Query { matrix: [[String]!]! }";
    let query = "query Q { matrix }";
    let output = gen_ops(schema, query);
    let field = extract_field(&output, "matrix");
    assert_eq!(
        field.type_expr,
        "ReadonlyArray<ReadonlyArray<string | null | undefined>>"
    );
    assert!(!field.optional);
}

#[test]
fn nested_scalar_list_nullable_inner_list() {
    // [[String!]]!
    let schema = "type Query { matrix: [[String!]]! }";
    let query = "query Q { matrix }";
    let output = gen_ops(schema, query);
    let field = extract_field(&output, "matrix");
    assert_eq!(
        field.type_expr,
        "ReadonlyArray<ReadonlyArray<string> | null | undefined>"
    );
    assert!(!field.optional);
}

#[test]
fn nested_scalar_list_nullable_outer_list() {
    // [[String!]!]
    let schema = "type Query { matrix: [[String!]!] }";
    let query = "query Q { matrix }";
    let output = gen_ops(schema, query);
    let field = extract_field(&output, "matrix");
    assert_eq!(
        field.type_expr,
        "ReadonlyArray<ReadonlyArray<string>> | null | undefined"
    );
    assert!(field.optional);
}

#[test]
fn nested_scalar_list_all_nullable() {
    // [[String]]
    let schema = "type Query { matrix: [[String]] }";
    let query = "query Q { matrix }";
    let output = gen_ops(schema, query);
    let field = extract_field(&output, "matrix");
    assert_eq!(
        field.type_expr,
        "ReadonlyArray<ReadonlyArray<string | null | undefined> | null | undefined> | null | undefined"
    );
    assert!(field.optional);
}

#[test]
fn nested_object_list_both_non_null() {
    let schema = "\
type Query { grid: [[Cell!]!]! }
type Cell { x: Int!, y: Int! }
";
    let query = "query Q { grid { x y } }";
    let output = gen_ops(schema, query);
    let field = extract_field(&output, "grid");
    assert!(field.type_expr.starts_with("ReadonlyArray<ReadonlyArray<"));
    assert!(!field.type_expr.contains("null"));
    assert!(!field.optional);
}

#[test]
fn nested_object_list_nullable_element() {
    let schema = "\
type Query { grid: [[Cell]!]! }
type Cell { x: Int!, y: Int! }
";
    let query = "query Q { grid { x y } }";
    let output = gen_ops(schema, query);
    let field = extract_field(&output, "grid");
    assert!(field.type_expr.starts_with("ReadonlyArray<ReadonlyArray<"));
    // Inner element is nullable
    assert!(field.type_expr.contains("} | null | undefined"));
    // Outer lists are non-null
    assert!(!field.type_expr.ends_with("> | null | undefined"));
    assert!(!field.optional);
}

// ── immutable vs mutable array types ─────────────────────────────

#[test]
fn immutable_types_uses_readonly_array() {
    let schema = "type Query { items: [String!]! }";
    let query = "query Q { items }";
    let output = TestGen::new()
        .no_base_schema()
        .schema_str(schema)
        .operations_str(query)
        .plugin("typescript-operations")
        .options(PluginOptions {
            immutable_types: true,
            ..PluginOptions::default()
        })
        .generate();

    assert!(output.contains("ReadonlyArray<"));
}

#[test]
fn mutable_types_uses_array() {
    let schema = "type Query { items: [String!]! }";
    let query = "query Q { items }";
    let output = TestGen::new()
        .no_base_schema()
        .schema_str(schema)
        .operations_str(query)
        .plugin("typescript-operations")
        .options(PluginOptions {
            immutable_types: false,
            ..PluginOptions::default()
        })
        .generate();

    assert!(output.contains("Array<"));
    assert!(!output.contains("ReadonlyArray<"));
}
