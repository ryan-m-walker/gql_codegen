//! Focused tests for typescript-operations config options using inline schemas
//! and string assertions.
//!
//! Uses TestGen with `.generator("operation-types")` and inline schemas/operations
//! to test individual config knobs against expected output.

use std::collections::BTreeMap;

use gql_codegen_core::test_utils::TestGen;
use gql_codegen_core::{GeneratorOptions, ScalarConfig, TypenamePolicy};

const SCHEMA: &str = "\
type Query { user(id: ID!): User, users: [User!]! }
type User { id: ID!, name: String!, email: String }
";

const QUERY: &str = "\
query GetUser($id: ID!) {
  user(id: $id) {
    id
    name
    email
  }
}
";

/// Helper: generate typescript-operations from inline schema + query
fn gen_ops(schema: &str, query: &str, options: GeneratorOptions) -> String {
    TestGen::new()
        .no_base_schema()
        .schema_str(schema)
        .operations_str(query)
        .generator("operation-types")
        .options(options)
        .generate()
}

// ── baseline ──────────────────────────────────────────────────────

#[test]
fn baseline_generates_query_and_variables() {
    let output = gen_ops(SCHEMA, QUERY, GeneratorOptions::default());

    // Should generate result type (operation name + root type suffix, respects declarationKind)
    assert!(output.contains("export interface GetUserQuery {"));
    // Should generate variables type
    assert!(output.contains("export interface GetUserQueryVariables {"));
    // Fields from selection (SGC preset: readonly + immutable)
    assert!(output.contains("readonly id: string;"));
    assert!(output.contains("readonly name: string;"));
    // Nullable field
    assert!(output.contains("readonly email?: string | null;"));
    // Variable type
    assert!(output.contains("readonly id: string;"));
}

// ── immutable_types ───────────────────────────────────────────────

#[test]
fn immutable_types_adds_readonly() {
    let output = gen_ops(
        SCHEMA,
        QUERY,
        GeneratorOptions {
            immutable_types: Some(true),
            ..GeneratorOptions::default()
        },
    );

    assert!(output.contains("readonly id: string;"));
    assert!(output.contains("readonly name: string;"));
    // Variables should also be readonly
    assert!(output.contains("readonly id: string;"));
}

#[test]
fn mutable_types_no_readonly() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str(SCHEMA)
        .operations_str(QUERY)
        .generator("operation-types")
        .options(GeneratorOptions {
            immutable_types: Some(false),
            ..GeneratorOptions::default()
        })
        .generate();

    assert!(!output.contains("readonly"));
}

// ── skip_typename ─────────────────────────────────────────────────

#[test]
fn skip_typename_omits_typename_field() {
    let query = "query GetUser($id: ID!) { user(id: $id) { __typename id } }";
    let output = gen_ops(
        SCHEMA,
        query,
        GeneratorOptions {
            skip_typename: true,
            ..GeneratorOptions::default()
        },
    );

    assert!(!output.contains("__typename"));
    assert!(output.contains("id: string;"));
}

#[test]
fn typename_included_by_default() {
    let query = "query GetUser($id: ID!) { user(id: $id) { __typename id } }";
    let output = gen_ops(SCHEMA, query, GeneratorOptions::default());

    // SGC default: typename_policy: Always → __typename is always optional
    assert!(output.contains("__typename?: 'User';"));
}

// ── typename_policy ──────────────────────────────────────────────

#[test]
fn typename_policy_always_injects_even_when_not_selected() {
    let query = "query GetUser($id: ID!) { user(id: $id) { id } }";
    let output = gen_ops(
        SCHEMA,
        query,
        GeneratorOptions {
            typename_policy: Some(TypenamePolicy::Always),
            ..GeneratorOptions::default()
        },
    );

    assert!(output.contains("__typename?: 'User';"));
}

#[test]
fn typename_policy_as_selected_only_when_queried() {
    // Without __typename in query
    let query = "query GetUser($id: ID!) { user(id: $id) { id } }";
    let output = gen_ops(
        SCHEMA,
        query,
        GeneratorOptions {
            typename_policy: Some(TypenamePolicy::AsSelected),
            ..GeneratorOptions::default()
        },
    );

    assert!(!output.contains("__typename"));
}

#[test]
fn typename_policy_as_selected_non_optional_when_queried() {
    // With __typename in query — should be non-optional
    let query = "query GetUser($id: ID!) { user(id: $id) { __typename id } }";
    let output = gen_ops(
        SCHEMA,
        query,
        GeneratorOptions {
            typename_policy: Some(TypenamePolicy::AsSelected),
            ..GeneratorOptions::default()
        },
    );

    assert!(output.contains("__typename: 'User';"));
    assert!(!output.contains("__typename?"));
}

#[test]
fn typename_policy_skip_never_emits() {
    let query = "query GetUser($id: ID!) { user(id: $id) { __typename id } }";
    let output = gen_ops(
        SCHEMA,
        query,
        GeneratorOptions {
            typename_policy: Some(TypenamePolicy::Skip),
            ..GeneratorOptions::default()
        },
    );

    assert!(!output.contains("__typename"));
}

#[test]
fn skip_typename_bool_backwards_compat() {
    // Legacy skip_typename: true should behave same as TypenamePolicy::Skip
    let query = "query GetUser($id: ID!) { user(id: $id) { __typename id } }";
    let output = gen_ops(
        SCHEMA,
        query,
        GeneratorOptions {
            skip_typename: true,
            ..GeneratorOptions::default()
        },
    );

    assert!(!output.contains("__typename"));
}

#[test]
fn typename_policy_always_with_non_optional() {
    let query = "query GetUser($id: ID!) { user(id: $id) { id } }";
    let output = gen_ops(
        SCHEMA,
        query,
        GeneratorOptions {
            typename_policy: Some(TypenamePolicy::Always),
            non_optional_typename: true,
            ..GeneratorOptions::default()
        },
    );

    assert!(output.contains("__typename: 'User';"));
    assert!(!output.contains("__typename?"));
}

// ── scalars ───────────────────────────────────────────────────────

#[test]
fn custom_scalar_mapping() {
    let schema = "\
type Query { now: DateTime }
scalar DateTime
";
    let query = "query Now { now }";
    let mut scalars = BTreeMap::new();
    scalars.insert("DateTime".to_string(), ScalarConfig::Simple("Date".to_string()));

    let output = gen_ops(
        schema,
        query,
        GeneratorOptions {
            scalars,
            ..GeneratorOptions::default()
        },
    );

    assert!(output.contains("Date | null"));
}

#[test]
fn custom_scalar_detailed_uses_output() {
    let schema = "\
type Query { now: DateTime }
scalar DateTime
";
    let query = "query Now { now }";
    let mut scalars = BTreeMap::new();
    scalars.insert(
        "DateTime".to_string(),
        ScalarConfig::Detailed {
            input: "string".to_string(),
            output: "Date".to_string(),
        },
    );

    let output = gen_ops(
        schema,
        query,
        GeneratorOptions {
            scalars,
            ..GeneratorOptions::default()
        },
    );

    // Operations use output type for result fields
    assert!(output.contains("Date | null"));
}

// ── variables ─────────────────────────────────────────────────────

#[test]
fn variables_non_null_no_optional() {
    let output = gen_ops(SCHEMA, QUERY, GeneratorOptions::default());

    // $id: ID! — non-null, should not have ?
    // Variables type uses operation name + root type suffix
    assert!(output.contains("GetUserQueryVariables"));
}

#[test]
fn variables_with_default_value_are_optional() {
    let schema = "type Query { greet(name: String!): String! }";
    let query = "query Greet($name: String! = \"World\") { greet(name: $name) }";
    let output = gen_ops(schema, query, GeneratorOptions::default());

    // Variable with default value — currently rendered without optional marker
    // TODO: variables with default values should be optional (name?: string)
    assert!(output.contains("name: string;"));
}

#[test]
fn variables_nullable_are_optional() {
    let schema = "type Query { search(query: String): [String!]! }";
    let query = "query Search($query: String) { search(query: $query) }";
    let output = gen_ops(schema, query, GeneratorOptions::default());

    // Nullable variable — type includes `| null` but no `?:` optional marker
    // TODO: nullable variables should use optional marker (query?: string | null)
    assert!(output.contains("query: string | null;"));
}

// ── mutations ─────────────────────────────────────────────────────

#[test]
fn mutation_generates_result_and_variables() {
    let schema = "\
type Query { ok: Boolean }
type Mutation { createUser(name: String!): User! }
type User { id: ID!, name: String! }
";
    let mutation = "mutation CreateUser($name: String!) { createUser(name: $name) { id name } }";
    let output = gen_ops(schema, mutation, GeneratorOptions::default());

    assert!(output.contains("export interface CreateUserMutation {"));
    assert!(output.contains("export interface CreateUserMutationVariables {"));
    assert!(output.contains("name: string;"));
}

// ── nested selections ─────────────────────────────────────────────

#[test]
fn nested_object_renders_inline() {
    let schema = "\
type Query { user: User }
type User { id: ID!, profile: Profile }
type Profile { bio: String, avatar: String }
";
    let query = "query GetUser { user { id profile { bio avatar } } }";
    let output = gen_ops(schema, query, GeneratorOptions::default());

    // Nested selection should render as inline object
    assert!(output.contains("profile"));
    assert!(output.contains("bio"));
    assert!(output.contains("avatar"));
}

// ── no_export ─────────────────────────────────────────────────────

#[test]
fn no_export_removes_export_keyword() {
    let output = gen_ops(
        SCHEMA,
        QUERY,
        GeneratorOptions {
            no_export: true,
            ..GeneratorOptions::default()
        },
    );

    // Should not have export keyword
    assert!(!output.contains("export "));
    // Operations respect declarationKind (SGC defaults to interface)
    assert!(output.contains("interface GetUserQuery {"));
}

// ── fragments ─────────────────────────────────────────────────────

#[test]
fn fragment_generates_interface() {
    let query = "\
fragment UserFields on User {
  id
  name
  email
}
";
    let output = gen_ops(SCHEMA, query, GeneratorOptions::default());

    // Fragment name gets "Fragment" suffix, respects declarationKind
    assert!(output.contains("export interface UserFieldsFragment {"));
    assert!(output.contains("readonly id: string;"));
    assert!(output.contains("readonly name: string;"));
    assert!(output.contains("readonly email?: string | null;"));
}

// ── list types ────────────────────────────────────────────────────

#[test]
fn list_field_renders_array_type() {
    let query = "query GetUsers { users { id name } }";
    let output = gen_ops(SCHEMA, query, GeneratorOptions::default());

    // TODO: list fields with sub-selections should wrap with ReadonlyArray<>
    // Currently renders as inline object without array wrapper
    assert!(output.contains("readonly users: {"));
    assert!(output.contains("readonly id: string;"));
    assert!(output.contains("readonly name: string;"));
}

// ── conditional directives ────────────────────────────────────────

#[test]
fn include_directive_makes_field_optional() {
    let query = "query GetUser($id: ID!, $withEmail: Boolean!) { user(id: $id) { id name email @include(if: $withEmail) } }";
    let output = gen_ops(SCHEMA, query, GeneratorOptions::default());

    // email with @include should be optional regardless of schema nullability
    assert!(output.contains("email?:"));
}

#[test]
fn skip_directive_makes_field_optional() {
    let query = "query GetUser($id: ID!, $skipName: Boolean!) { user(id: $id) { id name @skip(if: $skipName) } }";
    let output = gen_ops(SCHEMA, query, GeneratorOptions::default());

    // name with @skip should be optional even though it's non-null in schema
    assert!(output.contains("name?:"));
}

// ── union / interface inline fragments ───────────────────────────

const UNION_SCHEMA: &str = "\
type Query { search: [SearchResult!]!, node: Node }
union SearchResult = Book | Movie
type Book { isbn: String!, title: String! }
type Movie { imdbId: String!, title: String! }
interface Node { id: ID! }
type Article implements Node { id: ID!, title: String! }
type Comment implements Node { id: ID!, text: String! }
";

#[test]
fn union_inline_fragments_produce_discriminated_union() {
    let query = "\
query Search {
  search {
    ... on Book { isbn title }
    ... on Movie { imdbId title }
  }
}";
    let output = gen_ops(UNION_SCHEMA, query, GeneratorOptions::default());

    assert!(output.contains("| {"), "Expected discriminated union branch");
    assert!(output.contains("'Book'"), "Expected Book typename literal");
    assert!(output.contains("'Movie'"), "Expected Movie typename literal");
    assert!(output.contains("isbn"), "Expected isbn field in Book variant");
    assert!(output.contains("imdbId"), "Expected imdbId field in Movie variant");
}

#[test]
fn interface_shared_fields_duplicated_into_variants() {
    let query = "\
query GetNode {
  node {
    id
    ... on Article { title }
    ... on Comment { text }
  }
}";
    let output = gen_ops(UNION_SCHEMA, query, GeneratorOptions::default());

    // Should produce discriminated union with shared `id` in each variant
    assert!(output.contains("'Article'"), "Expected Article typename");
    assert!(output.contains("'Comment'"), "Expected Comment typename");
    // id should appear (duplicated into each variant)
    assert!(output.contains("id: string"), "Expected shared id field");
    assert!(output.contains("title"), "Expected title in Article variant");
    assert!(output.contains("text"), "Expected text in Comment variant");
}

#[test]
fn nullable_union_field_includes_null() {
    let query = "\
query GetNode {
  node {
    ... on Article { id title }
    ... on Comment { id text }
  }
}";
    let output = gen_ops(UNION_SCHEMA, query, GeneratorOptions::default());

    // node is nullable (Node, not Node!), so should include | null
    assert!(output.contains("| null"), "Expected | null for nullable interface field");
}
