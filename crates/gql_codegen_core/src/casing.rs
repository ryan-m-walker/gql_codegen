//! Naming convention and case transformation utilities.
//!
//! Provides types and functions for transforming GraphQL type names
//! to different casing styles (PascalCase, camelCase, CONSTANT_CASE, etc.).

use serde::{Deserialize, Serialize};

/// Naming convention configuration
/// Supports both simple string format and detailed object configuration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NamingConvention {
    /// Single convention applied to all names (e.g., "keep", "pascalCase")
    Simple(NamingCase),
    /// Detailed configuration with separate conventions for different outputs
    Detailed(NamingConventionConfig),
}

impl Default for NamingConvention {
    fn default() -> Self {
        Self::Simple(NamingCase::PascalCase)
    }
}

/// Detailed naming convention configuration
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NamingConventionConfig {
    /// Convention for type names (interfaces, types, enums)
    #[serde(default)]
    pub type_names: Option<NamingCase>,

    /// Convention for enum values
    #[serde(default)]
    pub enum_values: Option<NamingCase>,

    /// Transform underscores in names (default: false)
    /// When true, underscores are removed and treated as word boundaries
    /// When false, underscores are preserved in output but still used for casing
    #[serde(default)]
    pub transform_underscore: bool,
}

/// Available naming case transformations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NamingCase {
    /// Keep original name unchanged
    Keep,
    /// PascalCase (e.g., MyTypeName)
    PascalCase,
    /// camelCase (e.g., myTypeName)
    CamelCase,
    /// CONSTANT_CASE (e.g., MY_TYPE_NAME)
    ConstantCase,
    /// snake_case (e.g., my_type_name)
    SnakeCase,
    /// lowercase (e.g., mytypename)
    #[serde(alias = "lowerCase")]
    Lowercase,
    /// UPPERCASE (e.g., MYTYPENAME)
    #[serde(alias = "upperCase")]
    Uppercase,
}

impl Default for NamingCase {
    fn default() -> Self {
        Self::PascalCase
    }
}

impl NamingCase {
    /// Apply this naming case transformation to a string
    pub fn apply(&self, s: &str, transform_underscore: bool) -> String {
        match self {
            Self::Keep => s.to_string(),
            Self::PascalCase => to_pascal_case(s, transform_underscore),
            Self::CamelCase => to_camel_case(s, transform_underscore),
            Self::ConstantCase => to_constant_case(s, transform_underscore),
            Self::SnakeCase => to_snake_case(s, transform_underscore),
            Self::Lowercase => s.to_lowercase(),
            Self::Uppercase => s.to_uppercase(),
        }
    }
}

/// A word segment with information about whether it was preceded by an underscore
struct WordSegment {
    word: String,
    preceded_by_underscore: bool,
}

/// Convert string to PascalCase
/// When transform_underscore is true, underscores are removed
/// When false, underscores are preserved where they originally existed
fn to_pascal_case(s: &str, transform_underscore: bool) -> String {
    let segments = split_words(s);
    let mut result = String::new();

    for seg in segments {
        // Add underscore if it existed originally and we're not transforming
        if seg.preceded_by_underscore && !transform_underscore {
            result.push('_');
        }

        // PascalCase the word
        let mut chars = seg.word.chars();
        if let Some(first) = chars.next() {
            result.extend(first.to_uppercase());
            result.extend(chars.map(|c| c.to_ascii_lowercase()));
        }
    }

    result
}

/// Convert string to camelCase
fn to_camel_case(s: &str, transform_underscore: bool) -> String {
    let segments = split_words(s);
    let mut result = String::new();

    for (i, seg) in segments.iter().enumerate() {
        // Add underscore if it existed originally and we're not transforming
        if seg.preceded_by_underscore && !transform_underscore {
            result.push('_');
        }

        // camelCase: first word lowercase, rest PascalCase
        let mut chars = seg.word.chars();
        if let Some(first) = chars.next() {
            if i == 0 {
                result.extend(first.to_lowercase());
                result.extend(chars.map(|c| c.to_ascii_lowercase()));
            } else {
                result.extend(first.to_uppercase());
                result.extend(chars.map(|c| c.to_ascii_lowercase()));
            }
        }
    }

    result
}

/// Convert string to CONSTANT_CASE (always uses underscores between words)
fn to_constant_case(s: &str, _transform_underscore: bool) -> String {
    let segments = split_words(s);
    segments
        .iter()
        .map(|seg| seg.word.to_uppercase())
        .collect::<Vec<_>>()
        .join("_")
}

/// Convert string to snake_case (always uses underscores between words)
fn to_snake_case(s: &str, _transform_underscore: bool) -> String {
    let segments = split_words(s);
    segments
        .iter()
        .map(|seg| seg.word.to_lowercase())
        .collect::<Vec<_>>()
        .join("_")
}

/// Split a string into word segments, tracking underscore positions
fn split_words(s: &str) -> Vec<WordSegment> {
    let mut segments = Vec::new();
    let mut current_word = String::new();
    let mut preceded_by_underscore = false;

    for c in s.chars() {
        if c == '_' {
            // Underscore is a word boundary
            if !current_word.is_empty() {
                segments.push(WordSegment {
                    word: current_word,
                    preceded_by_underscore,
                });
                current_word = String::new();
            }
            // Next word will be preceded by underscore
            preceded_by_underscore = true;
        } else if c.is_uppercase() && !current_word.is_empty() {
            // Check if previous char was lowercase (word boundary like "userId" -> ["user", "Id"])
            let last_was_lower = current_word
                .chars()
                .last()
                .is_some_and(|ch| ch.is_lowercase());
            if last_was_lower {
                segments.push(WordSegment {
                    word: current_word,
                    preceded_by_underscore,
                });
                current_word = String::new();
                preceded_by_underscore = false; // Case-based split, no underscore
            }
            current_word.push(c);
        } else {
            current_word.push(c);
        }
    }

    if !current_word.is_empty() {
        segments.push(WordSegment {
            word: current_word,
            preceded_by_underscore,
        });
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_words() {
        // camelCase
        let words: Vec<_> = split_words("userId").iter().map(|s| s.word.clone()).collect();
        assert_eq!(words, vec!["user", "Id"]);

        // PascalCase
        let words: Vec<_> = split_words("UserProfile").iter().map(|s| s.word.clone()).collect();
        assert_eq!(words, vec!["User", "Profile"]);

        // snake_case
        let words: Vec<_> = split_words("user_name").iter().map(|s| s.word.clone()).collect();
        assert_eq!(words, vec!["user", "name"]);

        // CONSTANT_CASE
        let words: Vec<_> = split_words("HTTP_STATUS").iter().map(|s| s.word.clone()).collect();
        assert_eq!(words, vec!["HTTP", "STATUS"]);
    }

    #[test]
    fn test_pascal_case() {
        // With transform_underscore = true (remove underscores)
        assert_eq!(to_pascal_case("user_name", true), "UserName");
        assert_eq!(to_pascal_case("HTTP_STATUS", true), "HttpStatus");
        assert_eq!(to_pascal_case("UserProfile", true), "UserProfile");

        // With transform_underscore = false (preserve underscores)
        assert_eq!(to_pascal_case("user_name", false), "User_Name");
        assert_eq!(to_pascal_case("HTTP_STATUS", false), "Http_Status");
        assert_eq!(to_pascal_case("UserProfile", false), "UserProfile");
    }

    #[test]
    fn test_camel_case() {
        // With transform_underscore = true
        assert_eq!(to_camel_case("user_name", true), "userName");
        assert_eq!(to_camel_case("HTTP_STATUS", true), "httpStatus");

        // With transform_underscore = false
        assert_eq!(to_camel_case("user_name", false), "user_Name");
        assert_eq!(to_camel_case("HTTP_STATUS", false), "http_Status");
    }

    #[test]
    fn test_constant_case() {
        assert_eq!(to_constant_case("userName", false), "USER_NAME");
        assert_eq!(to_constant_case("UserProfile", false), "USER_PROFILE");
        assert_eq!(to_constant_case("user_name", false), "USER_NAME");
    }

    #[test]
    fn test_snake_case() {
        assert_eq!(to_snake_case("userName", false), "user_name");
        assert_eq!(to_snake_case("UserProfile", false), "user_profile");
        assert_eq!(to_snake_case("HTTP_STATUS", false), "http_status");
    }
}
