//! Naming convention and case transformation utilities.
//!
//! Provides types and functions for transforming GraphQL type names
//! to different casing styles (PascalCase, camelCase, CONSTANT_CASE, etc.).

use std::borrow::Cow;

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

    /// When true, underscores are removed and treated as word boundaries
    /// When false (default), underscores are preserved in output
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
    /// Apply this naming case transformation to a string.
    pub fn apply<'a>(&self, s: &'a str, transform_underscore: bool) -> Cow<'a, str> {
        match self {
            Self::Keep => Cow::Borrowed(s),
            Self::PascalCase => Cow::Owned(to_pascal_case(s, transform_underscore)),
            Self::CamelCase => Cow::Owned(to_camel_case(s, transform_underscore)),
            Self::ConstantCase => Cow::Owned(to_constant_case(s)),
            Self::SnakeCase => Cow::Owned(to_snake_case(s)),
            Self::Lowercase => Cow::Owned(s.to_lowercase()),
            Self::Uppercase => Cow::Owned(s.to_uppercase()),
        }
    }
}

/// A word segment with info about whether it was preceded by an underscore
struct Word {
    text: String,
    preceded_by_underscore: bool,
}

/// Split string into words at underscores and case boundaries (e.g., "userId" -> ["user", "Id"])
fn split_into_words(s: &str) -> Vec<Word> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut preceded_by_underscore = false;

    for c in s.chars() {
        if c == '_' {
            if !current.is_empty() {
                words.push(Word {
                    text: current,
                    preceded_by_underscore,
                });
                current = String::new();
            }
            preceded_by_underscore = true;
        } else if c.is_uppercase() && !current.is_empty() {
            // Split on case transition: lowercase -> uppercase
            if current.chars().last().is_some_and(|ch| ch.is_lowercase()) {
                words.push(Word {
                    text: current,
                    preceded_by_underscore,
                });
                current = String::new();
                preceded_by_underscore = false;
            }
            current.push(c);
        } else {
            current.push(c);
        }
    }

    if !current.is_empty() {
        words.push(Word {
            text: current,
            preceded_by_underscore,
        });
    }

    words
}

fn to_pascal_case(s: &str, transform_underscore: bool) -> String {
    let words = split_into_words(s);
    let mut result = String::with_capacity(s.len());

    for word in words {
        if word.preceded_by_underscore && !transform_underscore {
            result.push('_');
        }
        // Capitalize first char, lowercase rest
        let mut chars = word.text.chars();
        if let Some(first) = chars.next() {
            result.extend(first.to_uppercase());
            for c in chars {
                result.push(c.to_ascii_lowercase());
            }
        }
    }

    result
}

fn to_camel_case(s: &str, transform_underscore: bool) -> String {
    let words = split_into_words(s);
    let mut result = String::with_capacity(s.len());

    for (i, word) in words.iter().enumerate() {
        if word.preceded_by_underscore && !transform_underscore {
            result.push('_');
        }

        let mut chars = word.text.chars();
        if let Some(first) = chars.next() {
            if i == 0 {
                // First word: all lowercase
                result.extend(first.to_lowercase());
            } else {
                // Subsequent words: capitalize first char
                result.extend(first.to_uppercase());
            }
            for c in chars {
                result.push(c.to_ascii_lowercase());
            }
        }
    }

    result
}

fn to_constant_case(s: &str) -> String {
    split_into_words(s)
        .iter()
        .map(|w| w.text.to_uppercase())
        .collect::<Vec<_>>()
        .join("_")
}

fn to_snake_case(s: &str) -> String {
    split_into_words(s)
        .iter()
        .map(|w| w.text.to_lowercase())
        .collect::<Vec<_>>()
        .join("_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_into_words() {
        let words =
            |s| -> Vec<String> { split_into_words(s).into_iter().map(|w| w.text).collect() };

        assert_eq!(words("userId"), vec!["user", "Id"]);
        assert_eq!(words("UserProfile"), vec!["User", "Profile"]);
        assert_eq!(words("user_name"), vec!["user", "name"]);
        assert_eq!(words("HTTP_STATUS"), vec!["HTTP", "STATUS"]);
        assert_eq!(words("XMLParser"), vec!["XMLParser"]); // No split in all-caps
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
        assert_eq!(to_camel_case("UserProfile", true), "userProfile");

        // With transform_underscore = false
        assert_eq!(to_camel_case("user_name", false), "user_Name");
        assert_eq!(to_camel_case("HTTP_STATUS", false), "http_Status");
    }

    #[test]
    fn test_constant_case() {
        assert_eq!(to_constant_case("userName"), "USER_NAME");
        assert_eq!(to_constant_case("UserProfile"), "USER_PROFILE");
        assert_eq!(to_constant_case("user_name"), "USER_NAME");
    }

    #[test]
    fn test_snake_case() {
        assert_eq!(to_snake_case("userName"), "user_name");
        assert_eq!(to_snake_case("UserProfile"), "user_profile");
        assert_eq!(to_snake_case("HTTP_STATUS"), "http_status");
    }

    #[test]
    fn test_cow_borrowed_for_keep() {
        let input = "SomeTypeName";
        let result = NamingCase::Keep.apply(input, false);
        assert!(matches!(result, Cow::Borrowed(_)));
    }
}
