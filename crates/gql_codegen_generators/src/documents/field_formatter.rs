use std::mem::take;

pub struct FieldFormatter {
    max_chars_per_line: usize,
}

impl FieldFormatter {
    pub fn new(max_chars_per_line: usize) -> Self {
        Self { max_chars_per_line }
    }

    pub fn format(&mut self, field: String) -> String {
        let len = field.len();

        if len <= self.max_chars_per_line {
            return field;
        }

        let mut buffer = String::with_capacity(len + len / 2);
        self.parse_name(&field, &mut buffer);

        buffer
    }

    fn parse_name(&mut self, field: &str, buffer: &mut String) {
        let mut chars = field.chars();

        for c in chars.by_ref() {
            match c {
                '(' | ')' => {
                    buffer.push(c);
                    buffer.push('\n');
                }
                _ => {
                    buffer.push(c);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format() {
        let mut formatter = FieldFormatter::new(80);

        let input = String::from(
            "mutation mutationWithSuperLongNameAndOnlyOneSingleArgument($input: SuperLongArgumentName!)",
        );
        let field = formatter.format(input);
        assert_eq!(
            field,
            r"mutation mutationWithSuperLongNameAndOnlyOneSingleArgument(
    $input: SuperLongArgumentName!
)"
        );
    }
}
