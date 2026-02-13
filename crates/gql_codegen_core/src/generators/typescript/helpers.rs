use apollo_compiler::Node;

use crate::Result;
use crate::generators::GeneratorContext;

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
    use crate::DeclarationKind;
    use crate::PluginOptions;
    use crate::generators::common::helpers::{render_decl_closing, render_decl_opening};
    use crate::test_utils::TestCtxBuilder;

    use super::*;

    // ── render_decl_opening ──────────────────────────────────────────────

    #[test]
    fn decl_opening_default_is_interface() {
        let ctx = TestCtxBuilder::new();

        let output = ctx.run(|ctx| {
            render_decl_opening(ctx, "Foo", None)?;
            Ok(())
        });

        // SGC default: declaration_kind = Interface
        assert_eq!(output, "export interface Foo {\n");
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

        // SGC default: Interface, no_export removes "export"
        assert_eq!(output, "interface Foo {\n");
    }

    #[test]
    fn decl_opening_default_with_interface() {
        // SGC default is Interface → uses "extends" keyword
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

        assert_eq!(output, "export interface User extends Node {\n");
    }

    #[test]
    fn decl_opening_type_with_interface() {
        let ctx = TestCtxBuilder::new()
            .schema_str(
                "interface Node { id: ID! }
                 type User implements Node { id: ID!, name: String }",
            )
            .options(PluginOptions {
                declaration_kind: Some(DeclarationKind::Type),
                ..Default::default()
            })
            .build();
        let user = ctx.get_object("User");

        let output = ctx.run(|gen_ctx| {
            render_decl_opening(gen_ctx, "User", Some(&user.implements_interfaces))?;
            Ok(())
        });

        assert_eq!(output, "export type User = Node & {\n");
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
    fn decl_opening_default_with_multiple_interfaces() {
        // SGC default is Interface → uses "extends" with comma-separated list
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

        assert_eq!(
            output,
            "export interface User extends Node, Timestamped {\n"
        );
    }

    #[test]
    fn decl_opening_type_with_multiple_interfaces() {
        let ctx = TestCtxBuilder::new()
            .schema_str(
                "interface Node { id: ID! }
                 interface Timestamped { createdAt: String }
                 type User implements Node & Timestamped { id: ID!, createdAt: String }",
            )
            .options(PluginOptions {
                declaration_kind: Some(DeclarationKind::Type),
                ..Default::default()
            })
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
    fn decl_closing_default_no_semicolon() {
        // SGC default: Interface → no trailing semicolon
        let ctx = TestCtxBuilder::new();

        let output = ctx.run(|ctx| {
            render_decl_closing(ctx)?;
            Ok(())
        });

        assert_eq!(output, "}\n");
    }

    #[test]
    fn decl_closing_type_has_semicolon() {
        let ctx = TestCtxBuilder::new().options(PluginOptions {
            declaration_kind: Some(DeclarationKind::Type),
            ..Default::default()
        });

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
    fn full_default_declaration() {
        // SGC default: Interface → no `= {` syntax, no trailing semicolon
        let ctx = TestCtxBuilder::new();

        let output = ctx.run(|ctx| {
            render_decl_opening(ctx, "User", None)?;
            render_decl_closing(ctx)?;
            Ok(())
        });

        assert_eq!(output, "export interface User {\n}\n");
    }

    #[test]
    fn full_type_declaration() {
        let ctx = TestCtxBuilder::new().options(PluginOptions {
            declaration_kind: Some(DeclarationKind::Type),
            ..Default::default()
        });

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
