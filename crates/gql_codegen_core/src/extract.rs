//! Fast, resilient, zero-copy GraphQL extraction from source files.
//!
//! Supported patterns:
//! - `gql`...``
//! - `graphql`...``
//! - `/* GraphQL */`...``
//!
//! Inspired by Relay's approach:
//! https://github.com/facebook/relay/blob/main/compiler/crates/extract-graphql/src/lib.rs

use std::path::Path;

/// Configuration for the extractor
#[derive(Debug, Clone)]
pub struct ExtractConfig {
    /// Tag identifiers to look for (default: ["gql", "graphql"])
    pub tags: Vec<String>,
    /// Magic comments to look for, case-insensitive (default: ["GraphQL"])
    pub magic_comments: Vec<String>,
}

impl Default for ExtractConfig {
    fn default() -> Self {
        Self {
            tags: vec!["gql".into(), "graphql".into()],
            magic_comments: vec!["GraphQL".into()],
        }
    }
}

/// An extracted GraphQL document with source location (zero-copy)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Extracted<'a> {
    /// The GraphQL source text (slice into original source)
    pub text: &'a str,
    /// 1-indexed line number where the GraphQL starts
    pub line: usize,
    /// 1-indexed column number
    pub column: usize,
}

/// Owned version of Extracted for when you need to store results
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedOwned {
    pub text: String,
    pub line: usize,
    pub column: usize,
}

impl<'a> Extracted<'a> {
    /// Convert to owned version (allocates)
    pub fn to_owned(&self) -> ExtractedOwned {
        ExtractedOwned {
            text: self.text.to_string(),
            line: self.line,
            column: self.column,
        }
    }
}

impl<'a> From<Extracted<'a>> for ExtractedOwned {
    fn from(e: Extracted<'a>) -> Self {
        e.to_owned()
    }
}

/// Extract all GraphQL documents from source text (zero-copy).
pub fn extract<'a>(source: &'a str, config: &ExtractConfig) -> Vec<Extracted<'a>> {
    // Quick bail - if none of our markers exist, skip the file entirely
    let has_tag = config.tags.iter().any(|tag| source.contains(tag.as_str()));
    let has_magic = config.magic_comments.iter().any(|mc| {
        // Case-insensitive check without allocation for common case
        source
            .to_ascii_lowercase()
            .contains(&mc.to_ascii_lowercase())
    });

    if !has_tag && !has_magic {
        return Vec::new();
    }

    let mut results = Vec::new();
    let mut scanner = Scanner::new(source);

    while !scanner.is_eof() {
        let ch = scanner.peek();

        match ch {
            // Skip string literals (avoid false positives inside strings)
            '"' => {
                scanner.advance();
                scanner.skip_string_double();
            }
            '\'' => {
                scanner.advance();
                scanner.skip_string_single();
            }

            // Handle comments - skip line comments, check block comments for magic
            '/' => {
                scanner.advance();

                match scanner.peek() {
                    '/' => scanner.skip_line_comment(),
                    '*' => {
                        scanner.advance(); // consume *
                        let comment_start_line = scanner.line;
                        let comment_start_col = scanner.column.saturating_sub(2);

                        if let Some(content) = scanner.read_block_comment() {
                            // Check for magic comment: /* GraphQL */ etc
                            let trimmed = content.trim();
                            let is_magic = config
                                .magic_comments
                                .iter()
                                .any(|mc| trimmed.eq_ignore_ascii_case(mc));

                            if is_magic {
                                // Look for template literal after comment
                                scanner.skip_whitespace();
                                if scanner.peek() == '`' {
                                    scanner.advance();
                                    if let Some(text) = scanner.read_template_literal() {
                                        results.push(Extracted {
                                            text,
                                            line: comment_start_line,
                                            column: comment_start_col,
                                        });
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Check for tag identifiers
            c if c.is_ascii_alphabetic() => {
                let start_line = scanner.line;
                let start_col = scanner.column;
                let ident = scanner.read_identifier();

                // Check if this identifier is one of our tags
                if config.tags.iter().any(|tag| tag == ident) {
                    scanner.skip_whitespace();
                    if scanner.peek() == '`' {
                        scanner.advance();
                        if let Some(text) = scanner.read_template_literal() {
                            results.push(Extracted {
                                text,
                                line: start_line,
                                column: start_col,
                            });
                        }
                    }
                }
            }

            // Skip template literals that aren't tagged (avoid confusion)
            '`' => {
                scanner.advance();
                scanner.skip_template_literal();
            }

            _ => {
                scanner.advance();
            }
        }
    }

    results
}

/// Extract and convert to owned (for file operations where source doesn't outlive)
pub fn extract_owned(source: &str, config: &ExtractConfig) -> Vec<ExtractedOwned> {
    extract(source, config)
        .into_iter()
        .map(|e| e.to_owned())
        .collect()
}

/// Convenience function for files - reads and extracts with default config
pub fn extract_from_file(path: &Path) -> std::io::Result<Vec<ExtractedOwned>> {
    let source = std::fs::read_to_string(path)?;
    Ok(extract_owned(&source, &ExtractConfig::default()))
}

/// Convenience function for files with custom config
pub fn extract_from_file_with_config(
    path: &Path,
    config: &ExtractConfig,
) -> std::io::Result<Vec<ExtractedOwned>> {
    let source = std::fs::read_to_string(path)?;
    Ok(extract_owned(&source, config))
}

/// A single-pass character scanner with position tracking (zero-copy)
struct Scanner<'a> {
    source: &'a str,
    bytes: &'a [u8],
    pos: usize,
    line: usize,
    column: usize,
}

impl<'a> Scanner<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            bytes: source.as_bytes(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    #[inline]
    fn is_eof(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    #[inline]
    fn peek(&self) -> char {
        if self.is_eof() {
            '\0'
        } else {
            self.bytes[self.pos] as char
        }
    }

    #[inline]
    fn advance(&mut self) -> char {
        if self.is_eof() {
            return '\0';
        }

        let ch = self.bytes[self.pos] as char;
        self.pos += 1;

        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }

        ch
    }

    fn skip_whitespace(&mut self) {
        while !self.is_eof() && self.peek().is_ascii_whitespace() {
            self.advance();
        }
    }

    fn skip_string_double(&mut self) {
        while !self.is_eof() {
            match self.advance() {
                '\\' => {
                    self.advance();
                }
                '"' => break,
                _ => {}
            }
        }
    }

    fn skip_string_single(&mut self) {
        while !self.is_eof() {
            match self.advance() {
                '\\' => {
                    self.advance();
                }
                '\'' => break,
                _ => {}
            }
        }
    }

    fn skip_line_comment(&mut self) {
        while !self.is_eof() && self.advance() != '\n' {}
    }

    /// Read block comment content (after /*), returns slice
    fn read_block_comment(&mut self) -> Option<&'a str> {
        let start = self.pos;

        while !self.is_eof() {
            let ch = self.advance();

            if ch == '*' && self.peek() == '/' {
                let end = self.pos - 1; // exclude the *
                self.advance(); // consume /
                return Some(&self.source[start..end]);
            }
        }

        None // unclosed
    }

    /// Read identifier, returns slice
    fn read_identifier(&mut self) -> &'a str {
        let start = self.pos;

        while !self.is_eof() {
            let ch = self.peek();

            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '$' {
                self.advance();
            } else {
                break;
            }
        }

        &self.source[start..self.pos]
    }

    /// Read template literal content (after opening backtick), returns slice
    ///
    /// Returns the raw content between backticks. If there are interpolations
    /// (${...}), they are included in the slice - let the GraphQL parser handle errors.
    fn read_template_literal(&mut self) -> Option<&'a str> {
        let start = self.pos;

        while !self.is_eof() {
            let ch = self.advance();

            match ch {
                '`' => {
                    let end = self.pos - 1; // exclude closing backtick
                    return Some(&self.source[start..end]);
                }
                '\\' => {
                    // Skip escaped character
                    self.advance();
                }
                '$' if self.peek() == '{' => {
                    // Skip interpolation - still include it in the slice
                    self.advance(); // {
                    self.skip_interpolation();
                }
                _ => {}
            }
        }

        None // unclosed
    }

    /// Skip a template literal without capturing content
    fn skip_template_literal(&mut self) {
        while !self.is_eof() {
            let ch = self.advance();

            match ch {
                '`' => return,
                '\\' => {
                    self.advance();
                }
                '$' if self.peek() == '{' => {
                    self.advance();
                    self.skip_interpolation();
                }
                _ => {}
            }
        }
    }

    fn skip_interpolation(&mut self) {
        let mut depth = 1;

        while !self.is_eof() && depth > 0 {
            let ch = self.advance();

            match ch {
                '{' => depth += 1,
                '}' => depth -= 1,
                '`' => self.skip_template_literal(),
                '"' => self.skip_string_double(),
                '\'' => self.skip_string_single(),
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn extract_default(source: &str) -> Vec<Extracted> {
        extract(source, &ExtractConfig::default())
    }

    #[test]
    fn test_gql_tag() {
        let source = r#"const q = gql`query GetUser { user { id } }`;"#;
        let results = extract_default(source);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].text, "query GetUser { user { id } }");
    }

    #[test]
    fn test_graphql_tag() {
        let source = r#"const q = graphql`query { user }`;"#;
        let results = extract_default(source);
        assert_eq!(results.len(), 1);
        assert!(results[0].text.contains("query { user }"));
    }

    #[test]
    fn test_magic_comment() {
        let source = r#"const q = /* GraphQL */ `query { user }`;"#;
        let results = extract_default(source);
        assert_eq!(results.len(), 1);
        assert!(results[0].text.contains("query { user }"));
    }

    #[test]
    fn test_magic_comment_case_insensitive() {
        let source = r#"const q = /* graphql */ `query { user }`;"#;
        let results = extract_default(source);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_multiple_documents() {
        let source = r#"
            const a = gql`query A { a }`;
            const b = graphql`query B { b }`;
            const c = /* GraphQL */ `query C { c }`;
        "#;
        let results = extract_default(source);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_skip_string_false_positive() {
        let source = r#"
            const s = "gql`not a query`";
            const q = gql`query Real { id }`;
        "#;
        let results = extract_default(source);
        assert_eq!(results.len(), 1);
        assert!(results[0].text.contains("query Real"));
    }

    #[test]
    fn test_skip_single_quote_string() {
        let source = r#"
            const s = 'gql`not a query`';
            const q = gql`query Real { id }`;
        "#;
        let results = extract_default(source);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_skip_line_comment() {
        let source = r#"
            // gql`not a query`
            const q = gql`query Real { id }`;
        "#;
        let results = extract_default(source);
        assert_eq!(results.len(), 1);
        assert!(results[0].text.contains("query Real"));
    }

    #[test]
    fn test_skip_block_comment_non_magic() {
        let source = r#"
            /* gql`not a query` */
            const q = gql`query Real { id }`;
        "#;
        let results = extract_default(source);
        assert_eq!(results.len(), 1);
        assert!(results[0].text.contains("query Real"));
    }

    #[test]
    fn test_untagged_template_literal_skipped() {
        let source = r#"
            const s = `just a string`;
            const q = gql`query Real { id }`;
        "#;
        let results = extract_default(source);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_broken_js_still_extracts() {
        // Invalid JS syntax but GraphQL should still be extracted
        let source = r#"
            import { missing from
            const q = gql`query Valid { id }`;
            const x = {{{ totally broken
        "#;
        let results = extract_default(source);
        assert_eq!(results.len(), 1);
        assert!(results[0].text.contains("query Valid"));
    }

    #[test]
    fn test_no_graphql_early_bail() {
        let source = r#"
            const x = 1;
            const y = "hello";
            function foo() { return 42; }
        "#;
        let results = extract_default(source);
        assert!(results.is_empty());
    }

    #[test]
    fn test_multiline_query() {
        let source = r#"
            const q = gql`
                query GetUser($id: ID!) {
                    user(id: $id) {
                        id
                        name
                        email
                    }
                }
            `;
        "#;
        let results = extract_default(source);
        assert_eq!(results.len(), 1);
        assert!(results[0].text.contains("query GetUser"));
        assert!(results[0].text.contains("user(id: $id)"));
    }

    #[test]
    fn test_custom_tags() {
        let config = ExtractConfig {
            tags: vec!["sql".into(), "myGql".into()],
            magic_comments: vec!["GraphQL".into()],
        };

        let source = r#"
            const a = sql`SELECT * FROM users`;
            const b = myGql`query { user }`;
            const c = gql`query { ignored }`;
        "#;
        let results = extract(source, &config);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_position_tracking() {
        let source = "const q = gql`query { user }`;";
        let results = extract_default(source);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line, 1);
        assert_eq!(results[0].column, 11); // 'gql' starts at column 11
    }

    #[test]
    fn test_whitespace_between_tag_and_backtick() {
        let source = r#"const q = gql   `query { user }`;"#;
        let results = extract_default(source);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_escaped_backtick_in_string() {
        let source = r#"
            const s = "test \` string";
            const q = gql`query { user }`;
        "#;
        let results = extract_default(source);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_gql_in_identifier_not_matched() {
        // "mygql" shouldn't match, only exact "gql"
        let source = r#"const q = mygql`not graphql`;"#;
        let results = extract_default(source);
        assert!(results.is_empty());
    }

    #[test]
    fn test_fragment() {
        let source = r#"
            const f = gql`
                fragment UserFields on User {
                    id
                    name
                }
            `;
        "#;
        let results = extract_default(source);
        assert_eq!(results.len(), 1);
        assert!(results[0].text.contains("fragment UserFields"));
    }

    #[test]
    fn test_multiple_magic_comments() {
        let config = ExtractConfig {
            tags: vec![],
            magic_comments: vec!["GraphQL".into(), "GQL".into()],
        };

        let source = r#"
            const a = /* GraphQL */ `query A { a }`;
            const b = /* GQL */ `query B { b }`;
            const c = /* SQL */ `SELECT * FROM users`;
        "#;
        let results = extract(source, &config);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_zero_copy_same_memory() {
        let source = "const q = gql`query { user }`;";
        let results = extract_default(source);

        // The extracted text should point into the original source
        let text_ptr = results[0].text.as_ptr();
        let source_ptr = source.as_ptr();

        // text should be within source's memory range
        assert!(text_ptr >= source_ptr);
        assert!(text_ptr < unsafe { source_ptr.add(source.len()) });
    }

    #[test]
    fn test_to_owned() {
        let source = "const q = gql`query { user }`;";
        let results = extract_default(source);
        let owned = results[0].to_owned();

        assert_eq!(owned.text, "query { user }");
        assert_eq!(owned.line, 1);
    }
}
