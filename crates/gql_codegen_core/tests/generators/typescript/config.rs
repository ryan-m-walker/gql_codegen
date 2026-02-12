//! Focused tests for individual config options using inline schemas and string assertions.
//!
//! Unlike the snapshot-based tests in sibling modules, these use small inline schemas
//! and assert_eq! to catch regressions loudly. Each test exercises one config knob
//! against a minimal schema so the expected output is visible at the call site.

use std::collections::BTreeMap;

use gql_codegen_core::test_utils::TestGen;
use gql_codegen_core::{
    DeclarationKind, NamingCase, NamingConvention, NamingConventionConfig, PluginOptions, Preset,
    ScalarConfig, TypenamePolicy,
};

// ── numeric_enums ──────────────────────────────────────────────────

#[test]
fn numeric_enums() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean }\nenum Status { ACTIVE INACTIVE PENDING }")
        .options(PluginOptions {
            enums_as_types: Some(false),
            numeric_enums: true,
            ..PluginOptions::serde_default()
        })
        .generate();

    // SGC preset: interface decl + readonly + immutable + AsSelected typename
    // User overrides: TS enums with numeric values
    assert_eq!(
        output,
        "\
export interface Query {
  readonly ok?: boolean | null;
}

export enum Status {
  ACTIVE = 0,
  INACTIVE = 1,
  PENDING = 2,
}

"
    );
}

#[test]
fn numeric_enums_suppresses_const() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean }\nenum Color { RED GREEN }")
        .options(PluginOptions {
            enums_as_types: Some(false),
            numeric_enums: true,
            const_enums: true,
            ..PluginOptions::serde_default()
        })
        .generate();

    // const keyword should be suppressed when numeric_enums is true
    assert!(output.contains("export enum Color {"));
    assert!(!output.contains("const enum"));
    assert!(output.contains("RED = 0,"));
    assert!(output.contains("GREEN = 1,"));
}

// ── declaration_kind ───────────────────────────────────────────────

#[test]
fn declaration_kind_type_alias() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean! }")
        .options(PluginOptions {
            declaration_kind: Some(DeclarationKind::Type),
            ..PluginOptions::serde_default()
        })
        .generate();

    // Type alias uses `= {` opening and `};` closing
    assert_eq!(
        output,
        "\
export type Query = {
  readonly ok: boolean;
};

"
    );
}

#[test]
fn declaration_kind_class() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean! }")
        .options(PluginOptions {
            declaration_kind: Some(DeclarationKind::Class),
            ..PluginOptions::serde_default()
        })
        .generate();

    // Class uses space (not ` = `) and no trailing semicolon
    assert_eq!(
        output,
        "\
export class Query {
  readonly ok: boolean;
}

"
    );
}

#[test]
fn declaration_kind_abstract_class() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean! }")
        .options(PluginOptions {
            declaration_kind: Some(DeclarationKind::AbstractClass),
            ..PluginOptions::serde_default()
        })
        .generate();

    assert!(output.contains("export abstract class Query {"));
}

// ── only_enums ─────────────────────────────────────────────────────

#[test]
fn only_enums_filters_non_enum_types() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str(
            "\
type Query { user: User }
type User { name: String! }
enum Role { ADMIN USER }
input CreateUserInput { name: String! }
interface Node { id: ID! }
union SearchResult = User
scalar DateTime
",
        )
        .options(PluginOptions {
            only_enums: true,
            ..PluginOptions::serde_default()
        })
        .generate();

    // Only the enum should appear in output
    assert!(output.contains("Role"));
    assert!(!output.contains("export interface User"));
    assert!(!output.contains("export interface Query"));
    assert!(!output.contains("CreateUserInput"));
    assert!(!output.contains("Node"));
    assert!(!output.contains("SearchResult"));
    assert!(!output.contains("DateTime"));
}

// ── use_utility_types ──────────────────────────────────────────────

#[test]
fn use_utility_types_renders_scalars_map() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean }\nscalar DateTime")
        .options(PluginOptions {
            use_utility_types: true,
            ..PluginOptions::serde_default()
        })
        .generate();

    // Should have the Scalars map at the top
    assert!(output.contains("All built-in and custom scalars"));
    assert!(output.contains("Scalars"));
    // Built-in scalars should appear in the map
    assert!(output.contains("String: { input: string; output: string; }"));
    assert!(output.contains("Int: { input: number; output: number; }"));
    assert!(output.contains("Boolean: { input: boolean; output: boolean; }"));
    // Custom scalar with no mapping → unknown
    assert!(output.contains("DateTime: { input: unknown; output: unknown; }"));
    // Should have Maybe/InputMaybe utility types
    assert!(output.contains("type Maybe<T>"));
    assert!(output.contains("type InputMaybe<T>"));
}

#[test]
fn use_utility_types_wraps_nullable_fields() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { name: String }")
        .options(PluginOptions {
            use_utility_types: true,
            ..PluginOptions::serde_default()
        })
        .generate();

    // Nullable field should use Maybe<> wrapper
    assert!(output.contains("Maybe<Scalars['String']['output']>"));
}

// ── scalars detailed config ────────────────────────────────────────

#[test]
fn scalars_detailed_input_output() {
    let mut scalars = BTreeMap::new();
    scalars.insert(
        "DateTime".to_string(),
        ScalarConfig::Detailed {
            input: "string".to_string(),
            output: "Date".to_string(),
        },
    );

    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean }\nscalar DateTime")
        .options(PluginOptions {
            scalars,
            ..PluginOptions::serde_default()
        })
        .generate();

    // Detailed scalar should render with input/output split
    assert!(output.contains("input: string;"));
    assert!(output.contains("output: Date;"));
}

// ── disable_descriptions ───────────────────────────────────────────

#[test]
fn disable_descriptions_suppresses_doc_comments() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str(
            "\
\"\"\"A user in the system\"\"\"
type User { name: String! }
type Query { user: User }
",
        )
        .options(PluginOptions {
            disable_descriptions: true,
            ..PluginOptions::serde_default()
        })
        .generate();

    assert!(!output.contains("/**"));
    assert!(!output.contains("A user in the system"));
}

#[test]
fn descriptions_rendered_by_default() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str(
            "\
\"\"\"A user in the system\"\"\"
type User { name: String! }
type Query { user: User }
",
        )
        .options(PluginOptions::default())
        .generate();

    assert!(output.contains("/** A user in the system */"));
}

// ── non_optional_typename ──────────────────────────────────────────

#[test]
fn non_optional_typename() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean! }")
        .options(PluginOptions {
            non_optional_typename: true,
            typename_policy: Some(TypenamePolicy::Always),
            ..PluginOptions::serde_default()
        })
        .generate();

    // __typename should NOT have ? when non_optional_typename is true
    assert!(output.contains("__typename: 'Query';"));
    assert!(!output.contains("__typename?:"));
}

#[test]
fn non_optional_typename_default_is_optional() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean! }")
        .options(PluginOptions {
            typename_policy: Some(TypenamePolicy::Always),
            ..PluginOptions::serde_default()
        })
        .generate();

    // With Always policy, __typename is emitted and optional by default
    assert!(output.contains("__typename?: 'Query';"));
}

// ── types_prefix / types_suffix ────────────────────────────────────

#[test]
fn types_prefix_applied_to_all_types() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str(
            "\
type Query { role: Role }
enum Role { ADMIN USER }
",
        )
        .options(PluginOptions {
            types_prefix: Some("I".to_string()),
            ..PluginOptions::serde_default()
        })
        .generate();

    // Object type name gets prefix
    assert!(output.contains("export interface IQuery {"));
    // Enum type name gets prefix (on top of enum_prefix if any)
    assert!(output.contains("IRole"));
    // Field references should also use prefixed name
    assert!(output.contains("IRole"));
}

#[test]
fn types_suffix_applied_to_all_types() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean! }")
        .options(PluginOptions {
            types_suffix: Some("GQL".to_string()),
            ..PluginOptions::serde_default()
        })
        .generate();

    assert!(output.contains("export interface QueryGQL {"));
}

#[test]
fn types_prefix_does_not_affect_typename_value() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean! }")
        .options(PluginOptions {
            types_prefix: Some("I".to_string()),
            typename_policy: Some(TypenamePolicy::Always),
            ..PluginOptions::serde_default()
        })
        .generate();

    // The type declaration gets prefix, but __typename value stays original
    assert!(output.contains("export interface IQuery {"));
    assert!(output.contains("__typename?: 'Query';"));
    assert!(!output.contains("__typename?: 'IQuery';"));
}

// ── enums_as_const ─────────────────────────────────────────────────

#[test]
fn enums_as_const() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean }\nenum Status { ACTIVE INACTIVE }")
        .options(PluginOptions {
            enums_as_const: true,
            ..PluginOptions::serde_default()
        })
        .generate();

    // Should generate `as const` object + derived type
    assert!(output.contains("as const;"));
    assert!(output.contains("typeof Status"));
    // Should NOT be a type union or TS enum
    assert!(!output.contains("type Status =\n  | 'ACTIVE'"));
    assert!(!output.contains("enum Status {"));
}

#[test]
fn enums_as_const_with_values() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean }\nenum Color { RED GREEN BLUE }")
        .options(PluginOptions {
            enums_as_const: true,
            ..PluginOptions::serde_default()
        })
        .generate();

    assert!(output.contains("RED"));
    assert!(output.contains("GREEN"));
    assert!(output.contains("BLUE"));
    assert!(output.contains("as const;"));
}

#[test]
fn numeric_enums_overrides_enums_as_const() {
    // numeric_enums has highest priority — produces TS enum with numeric values
    // even when enums_as_const is also set (matches JS lib behavior)
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean }\nenum Status { ACTIVE INACTIVE }")
        .options(PluginOptions {
            enums_as_const: true,
            numeric_enums: true,
            ..PluginOptions::serde_default()
        })
        .generate();

    // Should be a TS enum with numeric values, not as const
    assert!(output.contains("enum Status {"));
    assert!(output.contains("ACTIVE = 0,"));
    assert!(output.contains("INACTIVE = 1,"));
    assert!(!output.contains("as const"));
}

#[test]
fn enums_as_const_full_output() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean }\nenum Role { ADMIN USER GUEST }")
        .options(PluginOptions {
            enums_as_const: true,
            ..PluginOptions::serde_default()
        })
        .generate();

    // Pin the exact enum output (Query part varies with preset)
    assert!(output.contains(
        "\
export const Role = {
  ADMIN: 'ADMIN',
  USER: 'USER',
  GUEST: 'GUEST',
} as const;

export type Role = typeof Role[keyof typeof Role];"
    ));
}

// ── future_proof_enums edge cases ──────────────────────────────────

#[test]
fn future_proof_enums_not_added_to_ts_enums() {
    // The JS lib incorrectly adds | '%future added value' inside TS enums.
    // We should NOT do that — it's not valid TypeScript syntax.
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean }\nenum Status { ACTIVE INACTIVE }")
        .options(PluginOptions {
            enums_as_types: Some(false),
            future_proof_enums: true,
            ..PluginOptions::serde_default()
        })
        .generate();

    assert!(output.contains("enum Status {"));
    // Should NOT have the future proof value inside a TS enum
    assert!(!output.contains("%future added value"));
}

#[test]
fn future_proof_enums_added_to_type_unions() {
    // future_proof should work for type union style
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean }\nenum Status { ACTIVE INACTIVE }")
        .options(PluginOptions {
            enums_as_types: Some(true),
            future_proof_enums: true,
            ..PluginOptions::serde_default()
        })
        .generate();

    assert!(output.contains("| '%future added value';"));
}

#[test]
fn future_proof_enums_not_added_to_const_objects() {
    // as const objects don't need future proofing — the typeof pattern handles it
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean }\nenum Status { ACTIVE INACTIVE }")
        .options(PluginOptions {
            enums_as_const: true,
            future_proof_enums: true,
            ..PluginOptions::serde_default()
        })
        .generate();

    assert!(output.contains("as const;"));
    assert!(!output.contains("%future added value"));
}

// ── preset differences ─────────────────────────────────────────────

#[test]
fn sgc_preset_uses_interface_declarations() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean! }")
        .preset(Preset::Sgc)
        .generate();

    assert!(output.contains("export interface Query {"));
    assert!(output.contains("readonly"));
}

#[test]
fn graphql_codegen_preset_uses_type_aliases() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean! }")
        .preset(Preset::GraphqlCodegen)
        .generate();

    // graphql-codegen compat: type aliases, Maybe wrappers, Scalars map
    assert!(output.contains("export type Query = {"));
    assert!(output.contains("type Maybe<T>"));
    assert!(output.contains("Scalars"));
}

// ── naming_convention: simple ─────────────────────────────────────

/// Schema with multiple type kinds for casing tests
const CASING_SCHEMA: &str = "\
type Query { user: User, role: UserRole }
type User { firstName: String!, lastName: String! }
enum UserRole { ADMIN_USER, REGULAR_USER }
input CreateUserInput { userName: String! }
interface Node { id: ID! }
union SearchResult = User
scalar DateTime
";

#[test]
fn naming_simple_camel_case_transforms_type_names() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str(CASING_SCHEMA)
        .options(PluginOptions {
            naming_convention: Some(NamingConvention::Simple(NamingCase::CamelCase)),
            ..PluginOptions::serde_default()
        })
        .generate();

    // Type names get camelCase
    assert!(output.contains("export interface query {"));
    assert!(output.contains("export interface user {"));
    assert!(output.contains("export interface node {"));
    assert!(output.contains("export interface createUserInput {"));
    // Enum type name gets camelCase
    assert!(output.contains("userRole"));
    // Scalar type name gets camelCase
    assert!(output.contains("type dateTime"));
    // Union type name gets camelCase
    assert!(output.contains("searchResult"));
}

#[test]
fn naming_simple_camel_case_also_transforms_enum_values() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean }\nenum Status { ACTIVE_USER INACTIVE_USER }")
        .options(PluginOptions {
            naming_convention: Some(NamingConvention::Simple(NamingCase::CamelCase)),
            ..PluginOptions::serde_default()
        })
        .generate();

    // Simple convention applies to both type names AND enum values
    assert!(output.contains("'active_User'"));
    assert!(output.contains("'inactive_User'"));
}

#[test]
fn naming_simple_constant_case() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean }\nenum UserRole { AdminUser RegularUser }")
        .options(PluginOptions {
            naming_convention: Some(NamingConvention::Simple(NamingCase::ConstantCase)),
            ..PluginOptions::serde_default()
        })
        .generate();

    // Type name: QUERY
    assert!(output.contains("export interface QUERY {"));
    // Enum values: ADMIN_USER, REGULAR_USER
    assert!(output.contains("'ADMIN_USER'"));
    assert!(output.contains("'REGULAR_USER'"));
}

#[test]
fn naming_simple_snake_case() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean }\ntype UserProfile { firstName: String! }")
        .options(PluginOptions {
            naming_convention: Some(NamingConvention::Simple(NamingCase::SnakeCase)),
            ..PluginOptions::serde_default()
        })
        .generate();

    assert!(output.contains("export interface query {"));
    assert!(output.contains("export interface user_profile {"));
}

// ── naming_convention: detailed ───────────────────────────────────

#[test]
fn naming_detailed_separate_type_and_enum() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { role: Role }\nenum Role { ADMIN_USER GUEST_USER }")
        .options(PluginOptions {
            naming_convention: Some(NamingConvention::Detailed(NamingConventionConfig {
                type_names: Some(NamingCase::PascalCase),
                enum_values: Some(NamingCase::CamelCase),
                transform_underscore: false,
            })),
            ..PluginOptions::serde_default()
        })
        .generate();

    // Type names: PascalCase (Query, Role already PascalCase — no change)
    assert!(output.contains("export interface Query {"));
    // Enum values: camelCase with underscores preserved (transform_underscore: false)
    assert!(output.contains("'admin_User'"));
    assert!(output.contains("'guest_User'"));
}

#[test]
fn naming_detailed_transform_underscore_removes_underscores() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean }\nenum Status { ACTIVE_USER INACTIVE_USER }")
        .options(PluginOptions {
            naming_convention: Some(NamingConvention::Detailed(NamingConventionConfig {
                type_names: Some(NamingCase::PascalCase),
                enum_values: Some(NamingCase::CamelCase),
                transform_underscore: true,
            })),
            ..PluginOptions::serde_default()
        })
        .generate();

    // With transform_underscore: true, underscores become word boundaries and are removed
    assert!(output.contains("'activeUser'"));
    assert!(output.contains("'inactiveUser'"));
}

#[test]
fn naming_detailed_constant_case_enum_values() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean }\nenum Role { adminUser guestUser }")
        .options(PluginOptions {
            naming_convention: Some(NamingConvention::Detailed(NamingConventionConfig {
                type_names: Some(NamingCase::Keep),
                enum_values: Some(NamingCase::ConstantCase),
                transform_underscore: false,
            })),
            ..PluginOptions::serde_default()
        })
        .generate();

    // Type names kept as-is
    assert!(output.contains("export interface Query {"));
    // Enum values → CONSTANT_CASE
    assert!(output.contains("'ADMIN_USER'"));
    assert!(output.contains("'GUEST_USER'"));
}

#[test]
fn naming_detailed_type_names_only() {
    // enum_values defaults to Keep when not specified
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean }\nenum UserRole { ADMIN GUEST }")
        .options(PluginOptions {
            naming_convention: Some(NamingConvention::Detailed(NamingConventionConfig {
                type_names: Some(NamingCase::SnakeCase),
                enum_values: None, // defaults to Keep
                transform_underscore: false,
            })),
            ..PluginOptions::serde_default()
        })
        .generate();

    // Type name: snake_case
    assert!(output.contains("export interface query {"));
    assert!(output.contains("user_role"));
    // Enum values: kept as-is
    assert!(output.contains("'ADMIN'"));
    assert!(output.contains("'GUEST'"));
}

// ── casing + field type references ────────────────────────────────

#[test]
fn naming_transforms_field_type_references() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { user: UserProfile }\ntype UserProfile { name: String! }")
        .options(PluginOptions {
            naming_convention: Some(NamingConvention::Simple(NamingCase::SnakeCase)),
            ..PluginOptions::serde_default()
        })
        .generate();

    // Field type references should also be transformed
    assert!(output.contains("readonly user?: user_profile | null"));
}

#[test]
fn naming_transforms_union_member_references() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean }\ntype Dog { name: String! }\ntype Cat { name: String! }\nunion Pet = Dog | Cat")
        .options(PluginOptions {
            naming_convention: Some(NamingConvention::Simple(NamingCase::ConstantCase)),
            ..PluginOptions::serde_default()
        })
        .generate();

    // Union name and members should be transformed
    assert!(output.contains("PET"));
    assert!(output.contains("DOG"));
    assert!(output.contains("CAT"));
}

// ── casing does NOT transform field names or __typename values ───

#[test]
fn naming_does_not_transform_field_names() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean! }\ntype UserProfile { firstName: String! }")
        .options(PluginOptions {
            naming_convention: Some(NamingConvention::Simple(NamingCase::ConstantCase)),
            ..PluginOptions::serde_default()
        })
        .generate();

    // Field names stay as-is from schema (not FIRST_NAME)
    assert!(output.contains("firstName"));
    // But type names are transformed
    assert!(output.contains("USER_PROFILE"));
}

#[test]
fn naming_does_not_transform_typename_value() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean! }")
        .options(PluginOptions {
            naming_convention: Some(NamingConvention::Simple(NamingCase::SnakeCase)),
            typename_policy: Some(TypenamePolicy::Always),
            ..PluginOptions::serde_default()
        })
        .generate();

    // Type declaration uses transformed name
    assert!(output.contains("export interface query {"));
    // But __typename value preserves the original GraphQL name
    assert!(output.contains("__typename?: 'Query'"));
}

// ── casing + enum rendering modes ─────────────────────────────────

#[test]
fn naming_with_ts_enums() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean }\nenum UserRole { ADMIN_USER GUEST_USER }")
        .options(PluginOptions {
            enums_as_types: Some(false),
            naming_convention: Some(NamingConvention::Detailed(NamingConventionConfig {
                type_names: Some(NamingCase::PascalCase),
                enum_values: Some(NamingCase::CamelCase),
                transform_underscore: true,
            })),
            ..PluginOptions::serde_default()
        })
        .generate();

    // TS enum: member names are transformed, string values are original
    assert!(output.contains("enum UserRole {"));
    assert!(output.contains("adminUser = 'ADMIN_USER',"));
    assert!(output.contains("guestUser = 'GUEST_USER',"));
}

#[test]
fn naming_with_const_objects() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean }\nenum UserRole { ADMIN_USER GUEST_USER }")
        .options(PluginOptions {
            enums_as_const: true,
            naming_convention: Some(NamingConvention::Detailed(NamingConventionConfig {
                type_names: Some(NamingCase::PascalCase),
                enum_values: Some(NamingCase::CamelCase),
                transform_underscore: true,
            })),
            ..PluginOptions::serde_default()
        })
        .generate();

    // const object: keys are transformed, values are original
    assert!(output.contains("adminUser: 'ADMIN_USER',"));
    assert!(output.contains("guestUser: 'GUEST_USER',"));
    assert!(output.contains("as const;"));
}

#[test]
fn naming_with_numeric_enums() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean }\nenum UserRole { ADMIN_USER GUEST_USER }")
        .options(PluginOptions {
            enums_as_types: Some(false),
            numeric_enums: true,
            naming_convention: Some(NamingConvention::Detailed(NamingConventionConfig {
                type_names: Some(NamingCase::PascalCase),
                enum_values: Some(NamingCase::CamelCase),
                transform_underscore: true,
            })),
            ..PluginOptions::serde_default()
        })
        .generate();

    // numeric enum: member names are transformed, values are numeric
    assert!(output.contains("adminUser = 0,"));
    assert!(output.contains("guestUser = 1,"));
}

// ── casing + prefix/suffix interaction ────────────────────────────

#[test]
fn naming_with_types_prefix_applies_after_casing() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean! }\ntype user_profile { name: String! }")
        .options(PluginOptions {
            types_prefix: Some("I".to_string()),
            naming_convention: Some(NamingConvention::Simple(NamingCase::PascalCase)),
            ..PluginOptions::serde_default()
        })
        .generate();

    // Casing applied first (user_profile → User_Profile), then prefix
    assert!(output.contains("IQuery"));
    assert!(output.contains("IUser_Profile"));
}

#[test]
fn naming_with_enum_prefix_and_casing() {
    let output = TestGen::new()
        .no_base_schema()
        .schema_str("type Query { ok: Boolean }\nenum user_role { ADMIN GUEST }")
        .options(PluginOptions {
            enum_prefix: Some("E".to_string()),
            naming_convention: Some(NamingConvention::Detailed(NamingConventionConfig {
                type_names: Some(NamingCase::PascalCase),
                enum_values: None,
                transform_underscore: true,
            })),
            ..PluginOptions::serde_default()
        })
        .generate();

    // Type casing applied to enum name, then enum prefix added
    assert!(output.contains("EUserRole"));
}
