//! Focused tests for typescript-operations config options using inline schemas
//! and string assertions.
//!
//! Uses TestGen with `.plugin("typescript-operations")` and inline schemas/operations
//! to test individual config knobs against expected output.

use std::collections::BTreeMap;

use gql_codegen_core::test_utils::TestGen;
use gql_codegen_core::{PluginOptions, Preset, ScalarConfig, TypenamePolicy};

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
fn gen_ops(schema: &str, query: &str, options: PluginOptions) -> String {
    TestGen::new()
        .no_base_schema()
        .schema_str(schema)
        .operations_str(query)
        .plugin("typescript-operations")
        .options(options)
        .generate()
}

// ── baseline ──────────────────────────────────────────────────────

#[test]
fn baseline_generates_query_and_variables() {
    let output = gen_ops(SCHEMA, QUERY, PluginOptions::default());

    // Should generate result type
    assert!(output.contains("export interface GetUser {"));
    // Should generate variables type
    assert!(output.contains("export interface GetUserVariables {"));
    // Fields from selection
    assert!(output.contains("id: string;"));
    assert!(output.contains("name: string;"));
    // Nullable field
    assert!(output.contains("email?: string | null;"));
    // Variable type
    assert!(output.contains("id: string;"));
}

// ── immutable_types ───────────────────────────────────────────────

#[test]
fn immutable_types_adds_readonly() {
    let output = gen_ops(
        SCHEMA,
        QUERY,
        PluginOptions {
            immutable_types: true,
            ..PluginOptions::serde_default()
        },
    );

    assert!(output.contains("readonly id: string;"));
    assert!(output.contains("readonly name: string;"));
    // Variables should also be readonly
    assert!(output.contains("readonly id: string;"));
}

#[test]
fn mutable_types_no_readonly() {
    // graphql-codegen preset defaults immutable_types to false
    // (SGC preset defaults it to true, and boolean false can't override via merge)
    let output = TestGen::new()
        .no_base_schema()
        .schema_str(SCHEMA)
        .operations_str(QUERY)
        .plugin("typescript-operations")
        .preset(Preset::GraphqlCodegen)
        .options(PluginOptions::serde_default())
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
        PluginOptions {
            skip_typename: true,
            ..PluginOptions::serde_default()
        },
    );

    assert!(!output.contains("__typename"));
    assert!(output.contains("id: string;"));
}

#[test]
fn typename_included_by_default() {
    let query = "query GetUser($id: ID!) { user(id: $id) { __typename id } }";
    let output = gen_ops(SCHEMA, query, PluginOptions::default());

    // SGC preset defaults to AsSelected — explicitly selected __typename is non-optional
    assert!(output.contains("__typename: 'User';"));
    assert!(!output.contains("__typename?"));
}

// ── typename_policy ──────────────────────────────────────────────

#[test]
fn typename_policy_always_injects_even_when_not_selected() {
    let query = "query GetUser($id: ID!) { user(id: $id) { id } }";
    let output = gen_ops(
        SCHEMA,
        query,
        PluginOptions {
            typename_policy: Some(TypenamePolicy::Always),
            ..PluginOptions::serde_default()
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
        PluginOptions {
            typename_policy: Some(TypenamePolicy::AsSelected),
            ..PluginOptions::serde_default()
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
        PluginOptions {
            typename_policy: Some(TypenamePolicy::AsSelected),
            ..PluginOptions::serde_default()
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
        PluginOptions {
            typename_policy: Some(TypenamePolicy::Skip),
            ..PluginOptions::serde_default()
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
        PluginOptions {
            skip_typename: true,
            ..PluginOptions::serde_default()
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
        PluginOptions {
            typename_policy: Some(TypenamePolicy::Always),
            non_optional_typename: true,
            ..PluginOptions::serde_default()
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
        PluginOptions {
            scalars,
            ..PluginOptions::serde_default()
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
        PluginOptions {
            scalars,
            ..PluginOptions::serde_default()
        },
    );

    // Operations use output type for result fields
    assert!(output.contains("Date | null"));
}

// ── variables ─────────────────────────────────────────────────────

#[test]
fn variables_non_null_no_optional() {
    let output = gen_ops(SCHEMA, QUERY, PluginOptions::default());

    // $id: ID! — non-null, should not have ?
    // Find the variables type and check
    assert!(output.contains("GetUserVariables"));
}

#[test]
fn variables_with_default_value_are_optional() {
    let schema = "type Query { greet(name: String!): String! }";
    let query = "query Greet($name: String! = \"World\") { greet(name: $name) }";
    let output = gen_ops(schema, query, PluginOptions::default());

    // Variable with default value should be optional
    assert!(output.contains("name?: string;"));
}

#[test]
fn variables_nullable_are_optional() {
    let schema = "type Query { search(query: String): [String!]! }";
    let query = "query Search($query: String) { search(query: $query) }";
    let output = gen_ops(schema, query, PluginOptions::default());

    assert!(output.contains("query?: string | null;"));
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
    let output = gen_ops(schema, mutation, PluginOptions::default());

    assert!(output.contains("export interface CreateUser {"));
    assert!(output.contains("export interface CreateUserVariables {"));
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
    let output = gen_ops(schema, query, PluginOptions::default());

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
        PluginOptions {
            no_export: true,
            ..PluginOptions::serde_default()
        },
    );

    // Should not have export keyword
    assert!(!output.contains("export "));
    assert!(output.contains("interface GetUser {"));
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
    let output = gen_ops(SCHEMA, query, PluginOptions::default());

    assert!(output.contains("export interface UserFields {"));
    assert!(output.contains("id: string;"));
    assert!(output.contains("name: string;"));
    assert!(output.contains("email?: string | null;"));
}

// ── list types ────────────────────────────────────────────────────

#[test]
fn list_field_renders_array_type() {
    let query = "query GetUsers { users { id name } }";
    let output = gen_ops(SCHEMA, query, PluginOptions::default());

    assert!(output.contains("Array<"));
}

// ── conditional directives ────────────────────────────────────────

#[test]
fn include_directive_makes_field_optional() {
    let query = "query GetUser($id: ID!, $withEmail: Boolean!) { user(id: $id) { id name email @include(if: $withEmail) } }";
    let output = gen_ops(SCHEMA, query, PluginOptions::default());

    // email with @include should be optional regardless of schema nullability
    assert!(output.contains("email?:"));
}

#[test]
fn skip_directive_makes_field_optional() {
    let query = "query GetUser($id: ID!, $skipName: Boolean!) { user(id: $id) { id name @skip(if: $skipName) } }";
    let output = gen_ops(SCHEMA, query, PluginOptions::default());

    // name with @skip should be optional even though it's non-null in schema
    assert!(output.contains("name?:"));
}
