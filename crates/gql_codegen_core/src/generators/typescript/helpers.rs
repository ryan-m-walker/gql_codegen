use apollo_compiler::Node;
use apollo_compiler::collections::IndexSet;
use apollo_compiler::schema::ComponentName;

use crate::generators::GeneratorContext;
use crate::{DeclarationKind, Result};

/// Returns the export keyword based if exports are enabled.
pub(crate) fn get_export_kw(ctx: &GeneratorContext) -> &'static str {
    if ctx.options.no_export { "" } else { "export " }
}

/// Returns the readonly keyword based on immutability configuration.
pub(crate) fn get_readonly_kw(ctx: &GeneratorContext) -> &'static str {
    if ctx.options.immutable_types {
        "readonly "
    } else {
        ""
    }
}

/// Map built-in GraphQL scalar types to TypeScript types.
/// Returns None for custom scalars that need separate handling.
pub(crate) fn gql_scalar_to_ts(name: &str) -> Option<&'static str> {
    match name {
        "String" | "ID" => Some("string"),
        "Int" | "Float" => Some("number"),
        "Boolean" => Some("boolean"),
        _ => None,
    }
}

/// TODO: make this less strict, allowing strings which we parse or fallback to default
pub(crate) fn get_decl_kind_kw(ctx: &GeneratorContext) -> &'static str {
    match ctx.options.declaration_kind {
        Some(DeclarationKind::Type) | None => "type",
        Some(DeclarationKind::Interface) => "interface",
        Some(DeclarationKind::Class) => "class",
        Some(DeclarationKind::AbstractClass) => "abstract class",
    }
}

/// Renders the opening of a GraphQL object type declaration.
pub(crate) fn render_decl_opening(
    ctx: &mut GeneratorContext,
    name: &str,
    implements_interfaces: Option<&IndexSet<ComponentName>>,
) -> Result<()> {
    let export = get_export_kw(ctx);
    let decl_kind = get_decl_kind_kw(ctx);

    let separator = match ctx.options.declaration_kind {
        Some(DeclarationKind::Type) | None => " = ",
        _ => " ",
    };

    write!(ctx.writer, "{export}{decl_kind} {name}{separator}")?;

    if let Some(interfaces) = implements_interfaces {
        if !interfaces.is_empty() {
            match ctx.options.declaration_kind {
                Some(DeclarationKind::Type) | None => {
                    for interface in interfaces {
                        write!(ctx.writer, "{interface}")?;
                        write!(ctx.writer, " & ")?;
                    }
                }
                _ => {
                    write!(ctx.writer, "implements ")?;

                    for (i, interface) in interfaces.iter().enumerate() {
                        write!(ctx.writer, "{interface}")?;
                        if i < interfaces.len() - 1 {
                            write!(ctx.writer, ", ")?;
                        }
                    }

                    write!(ctx.writer, " ")?;
                }
            }
        }
    }

    writeln!(ctx.writer, "{{")?;

    Ok(())
}

pub(crate) fn render_decl_closing(ctx: &mut GeneratorContext) -> Result<()> {
    let semi = match ctx.options.declaration_kind {
        Some(DeclarationKind::Type) | None => ";",
        _ => "",
    };

    writeln!(ctx.writer, "}}{semi}")?;
    Ok(())
}

/// Convert a GraphQL description to a TypeScript doc comment.
///
/// **Example:**
/// ``` graphql
/// """
/// This is a description
/// """
///
/// ```
/// **Output:**
/// ``` typescript
/// /** This is a description */
/// ```
pub(crate) fn render_description(
    ctx: &mut GeneratorContext,
    description: &Option<Node<str>>,
    indent_level: usize,
) -> Result<()> {
    if ctx.options.disable_descriptions {
        return Ok(());
    }

    let Some(description) = description else {
        return Ok(());
    };

    if description.is_empty() {
        return Ok(());
    }

    let indent = if indent_level > 0 {
        " ".repeat(indent_level * 2)
    } else {
        "".to_string()
    };

    if description.lines().count() > 1 {
        writeln!(ctx.writer, "{indent}/**")?;
        for line in description.lines() {
            writeln!(ctx.writer, "{indent} * {line}")?;
        }
        writeln!(ctx.writer, "{indent} */")?;

        return Ok(());
    }

    writeln!(ctx.writer, "{indent}/** {description} */")?;

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::PluginOptions;
    use crate::test_utils::TestCtxBuilder;

    use super::*;

    // ── render_decl_opening ──────────────────────────────────────────────

    #[test]
    fn decl_opening_type_default() {
        let ctx = TestCtxBuilder::new();

        let output = ctx.run(|ctx| {
            render_decl_opening(ctx, "Foo", None)?;
            Ok(())
        });

        assert_eq!(output, "export type Foo = {\n");
    }

    #[test]
    fn decl_opening_interface() {
        let ctx = TestCtxBuilder::new().options(PluginOptions {
            declaration_kind: Some(DeclarationKind::Interface),
            ..Default::default()
        });

        let output = ctx.run(|ctx| {
            render_decl_opening(ctx, "Foo", None)?;
            Ok(())
        });

        assert_eq!(output, "export interface Foo {\n");
    }

    #[test]
    fn decl_opening_class() {
        let ctx = TestCtxBuilder::new().options(PluginOptions {
            declaration_kind: Some(DeclarationKind::Class),
            ..Default::default()
        });

        let output = ctx.run(|ctx| {
            render_decl_opening(ctx, "Foo", None)?;
            Ok(())
        });

        assert_eq!(output, "export class Foo {\n");
    }

    #[test]
    fn decl_opening_abstract_class() {
        let ctx = TestCtxBuilder::new().options(PluginOptions {
            declaration_kind: Some(DeclarationKind::AbstractClass),
            ..Default::default()
        });

        let output = ctx.run(|ctx| {
            render_decl_opening(ctx, "Foo", None)?;
            Ok(())
        });

        assert_eq!(output, "export abstract class Foo {\n");
    }

    #[test]
    fn decl_opening_no_export() {
        let ctx = TestCtxBuilder::new().options(PluginOptions {
            no_export: true,
            ..Default::default()
        });

        let output = ctx.run(|ctx| {
            render_decl_opening(ctx, "Foo", None)?;
            Ok(())
        });

        assert_eq!(output, "type Foo = {\n");
    }

    #[test]
    fn decl_opening_type_with_interface() {
        let ctx = TestCtxBuilder::new()
            .schema_str(
                "interface Node { id: ID! }
                 type User implements Node { id: ID!, name: String }",
            )
            .build();
        let user = ctx.get_object("User");

        let output = ctx.run(|gen_ctx| {
            render_decl_opening(gen_ctx, "User", Some(&user.implements_interfaces))?;
            Ok(())
        });

        assert_eq!(output, "export type User = Node & {\n");
    }

    #[test]
    fn decl_opening_interface_with_implements() {
        let ctx = TestCtxBuilder::new()
            .schema_str(
                "interface Node { id: ID! }
                 type User implements Node { id: ID!, name: String }",
            )
            .options(PluginOptions {
                declaration_kind: Some(DeclarationKind::Interface),
                ..Default::default()
            })
            .build();
        let user = ctx.get_object("User");

        let output = ctx.run(|gen_ctx| {
            render_decl_opening(gen_ctx, "User", Some(&user.implements_interfaces))?;
            Ok(())
        });

        assert_eq!(output, "export interface User implements Node {\n");
    }

    #[test]
    fn decl_opening_class_with_multiple_interfaces() {
        let ctx = TestCtxBuilder::new()
            .schema_str(
                "interface Node { id: ID! }
                 interface Timestamped { createdAt: String }
                 type User implements Node & Timestamped { id: ID!, createdAt: String }",
            )
            .options(PluginOptions {
                declaration_kind: Some(DeclarationKind::Class),
                ..Default::default()
            })
            .build();
        let user = ctx.get_object("User");

        let output = ctx.run(|gen_ctx| {
            render_decl_opening(gen_ctx, "User", Some(&user.implements_interfaces))?;
            Ok(())
        });

        assert_eq!(output, "export class User implements Node, Timestamped {\n");
    }

    #[test]
    fn decl_opening_type_with_multiple_interfaces() {
        let ctx = TestCtxBuilder::new()
            .schema_str(
                "interface Node { id: ID! }
                 interface Timestamped { createdAt: String }
                 type User implements Node & Timestamped { id: ID!, createdAt: String }",
            )
            .build();
        let user = ctx.get_object("User");

        let output = ctx.run(|gen_ctx| {
            render_decl_opening(gen_ctx, "User", Some(&user.implements_interfaces))?;
            Ok(())
        });

        assert_eq!(output, "export type User = Node & Timestamped & {\n");
    }

    // ── render_decl_closing ──────────────────────────────────────────────

    #[test]
    fn decl_closing_type_has_semicolon() {
        let ctx = TestCtxBuilder::new();

        let output = ctx.run(|ctx| {
            render_decl_closing(ctx)?;
            Ok(())
        });

        assert_eq!(output, "};\n");
    }

    #[test]
    fn decl_closing_interface_no_semicolon() {
        let ctx = TestCtxBuilder::new().options(PluginOptions {
            declaration_kind: Some(DeclarationKind::Interface),
            ..Default::default()
        });

        let output = ctx.run(|ctx| {
            render_decl_closing(ctx)?;
            Ok(())
        });

        assert_eq!(output, "}\n");
    }

    #[test]
    fn decl_closing_class_no_semicolon() {
        let ctx = TestCtxBuilder::new().options(PluginOptions {
            declaration_kind: Some(DeclarationKind::Class),
            ..Default::default()
        });

        let output = ctx.run(|ctx| {
            render_decl_closing(ctx)?;
            Ok(())
        });

        assert_eq!(output, "}\n");
    }

    // ── render_description ───────────────────────────────────────────────

    #[test]
    fn description_none_outputs_nothing() {
        let ctx = TestCtxBuilder::new();

        let output = ctx.run(|ctx| {
            render_description(ctx, &None, 0)?;
            Ok(())
        });

        assert_eq!(output, "");
    }

    #[test]
    fn description_single_line() {
        let ctx = TestCtxBuilder::new()
            .schema_str(
                r#""""A user in the system"""
                type User { id: ID! }"#,
            )
            .build();
        let desc = &ctx.get_object("User").description;

        let output = ctx.run(|gen_ctx| {
            render_description(gen_ctx, desc, 0)?;
            Ok(())
        });
        assert_eq!(output, "/** A user in the system */\n");
    }

    #[test]
    fn description_multi_line() {
        let ctx = TestCtxBuilder::new()
            .schema_str(
                r#"
                """
                A user in the system.
                Has an ID and a name.
                """
                type User { id: ID! }
                "#,
            )
            .build();
        let desc = &ctx.get_object("User").description;

        let output = ctx.run(|gen_ctx| {
            render_description(gen_ctx, desc, 0)?;
            Ok(())
        });

        assert!(output.contains("/**\n"));
        assert!(output.contains(" * A user in the system.\n"));
        assert!(output.contains(" * Has an ID and a name.\n"));
        assert!(output.contains(" */\n"));
    }

    #[test]
    fn description_with_indent() {
        let ctx = TestCtxBuilder::new()
            .schema_str(
                r#""""A field description"""
                type User { id: ID! }"#,
            )
            .build();
        let desc = &ctx.get_object("User").description;

        let output = ctx.run(|gen_ctx| {
            render_description(gen_ctx, desc, 1)?;
            Ok(())
        });

        assert_eq!(output, "  /** A field description */\n");
    }

    #[test]
    fn description_with_deeper_indent() {
        let ctx = TestCtxBuilder::new()
            .schema_str(
                r#""""Nested"""
                type User { id: ID! }"#,
            )
            .build();
        let desc = &ctx.get_object("User").description;

        let output = ctx.run(|gen_ctx| {
            render_description(gen_ctx, desc, 3)?;
            Ok(())
        });

        assert_eq!(output, "      /** Nested */\n");
    }

    #[test]
    fn description_disabled() {
        let ctx = TestCtxBuilder::new()
            .schema_str(
                r#""""Should not appear"""
                type User { id: ID! }"#,
            )
            .options(PluginOptions {
                disable_descriptions: true,
                ..Default::default()
            })
            .build();
        let desc = &ctx.get_object("User").description;

        let output = ctx.run(|gen_ctx| {
            render_description(gen_ctx, desc, 0)?;
            Ok(())
        });

        assert_eq!(output, "");
    }

    #[test]
    fn description_absent_outputs_nothing() {
        let ctx = TestCtxBuilder::new()
            .schema_str("type User { id: ID! }")
            .build();
        let desc = &ctx.get_object("User").description;

        let output = ctx.run(|gen_ctx| {
            render_description(gen_ctx, desc, 0)?;
            Ok(())
        });

        assert_eq!(output, "");
    }

    // ── full declaration round-trip ──────────────────────────────────────

    #[test]
    fn full_type_declaration() {
        let ctx = TestCtxBuilder::new();

        let output = ctx.run(|ctx| {
            render_decl_opening(ctx, "User", None)?;
            render_decl_closing(ctx)?;
            Ok(())
        });

        assert_eq!(output, "export type User = {\n};\n");
    }

    #[test]
    fn full_interface_declaration() {
        let ctx = TestCtxBuilder::new().options(PluginOptions {
            declaration_kind: Some(DeclarationKind::Interface),
            ..Default::default()
        });

        let output = ctx.run(|ctx| {
            render_decl_opening(ctx, "User", None)?;
            render_decl_closing(ctx)?;
            Ok(())
        });

        assert_eq!(output, "export interface User {\n}\n");
    }
}
