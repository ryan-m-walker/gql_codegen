pub fn render_description_comment(description: &str, indentation: usize) -> String {
    let indentation = "  ".repeat(indentation);
    let mut output = description;

    if description.starts_with("\"\"\"") {
        output = description.strip_prefix("\"\"\"").unwrap_or("");
    }

    if description.ends_with("\"\"\"") {
        output = output.strip_suffix("\"\"\"").unwrap_or("");
    }

    let description_value = output
        .trim()
        .lines()
        .map(|line| format!("{indentation} * {}", line.trim()))
        .collect::<Vec<String>>()
        .join("\n");

    format!("{indentation}/**\n{description_value}\n{indentation} */\n")
}
