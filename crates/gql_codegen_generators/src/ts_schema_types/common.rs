use std::io::Result;
use std::io::Write;

use apollo_compiler::Node;
use apollo_compiler::ast::Type;

pub(crate) fn render_type(ty: &Type) -> String {
    match ty {
        Type::Named(name) => format!("{name} | null | undefined"),
        Type::NonNullNamed(name) => name.to_string(),
        Type::List(inner) => format!("Array<{}> | null | undefined", render_type(inner)),
        Type::NonNullList(inner) => format!("Array<{}>", render_type(inner)),
    }
}

pub(crate) fn render_description<T: Write>(
    writer: &mut T,
    description: &Option<Node<str>>,
    padding: &str,
) -> Result<()> {
    if let Some(description) = description {
        writeln!(writer, "{padding}/**")?;

        for line in description.lines() {
            writeln!(writer, "{padding} * {line}")?;
        }

        writeln!(writer, "{padding} */")?;
    }

    Ok(())
}
