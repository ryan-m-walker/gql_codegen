pub fn render_description_comment(description: &str, indentation: usize) -> String {
    let indentation = "  ".repeat(indentation);

    let description_value = description
        .lines()
        .map(|line| format!("{indentation} * {}", line))
        .collect::<Vec<String>>()
        .join("\n");

    format!("{indentation}/**\n{description_value}\n{indentation} */\n")
}
