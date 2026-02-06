//! Case transformation functions.
//!
//! Each function takes a string and returns the transformed version.
//! Most follow the pattern: split into words → transform each → join with separator.

use super::words::split_into_words;

// ─────────────────────────────────────────────────────────────────────────────
// First-class transforms (documented, recommended)
// ─────────────────────────────────────────────────────────────────────────────

/// PascalCase: `UserProfile`, `HttpStatus`
pub(crate) fn to_pascal_case(s: &str, transform_underscore: bool) -> String {
    let words = split_into_words(s);
    let mut result = String::with_capacity(s.len());

    for word in words {
        if word.preceded_by_underscore && !transform_underscore {
            result.push('_');
        }

        capitalize_word(&word.text, &mut result);
    }

    result
}

/// camelCase: `userProfile`, `httpStatus`
pub(crate) fn to_camel_case(s: &str, transform_underscore: bool) -> String {
    let words = split_into_words(s);
    let mut result = String::with_capacity(s.len());

    for (i, word) in words.iter().enumerate() {
        if word.preceded_by_underscore && !transform_underscore {
            result.push('_');
        }

        if i == 0 {
            lowercase_word(&word.text, &mut result);
        } else {
            capitalize_word(&word.text, &mut result);
        }
    }

    result
}

/// CONSTANT_CASE: `USER_PROFILE`, `HTTP_STATUS`
pub(crate) fn to_constant_case(s: &str) -> String {
    join_words(s, "_", |text, _| text.to_uppercase())
}

/// snake_case: `user_profile`, `http_status`
pub(crate) fn to_snake_case(s: &str) -> String {
    join_words(s, "_", |text, _| text.to_lowercase())
}

// ─────────────────────────────────────────────────────────────────────────────
// Supported transforms (for compat, less prominently documented)
// ─────────────────────────────────────────────────────────────────────────────

/// Capital Case: `User Profile`, `Http Status`
pub(crate) fn to_capital_case(s: &str) -> String {
    join_words(s, " ", |text, _| {
        let mut result = String::with_capacity(text.len());
        capitalize_word(text, &mut result);
        result
    })
}

/// dot.case: `user.profile`, `http.status`
pub(crate) fn to_dot_case(s: &str) -> String {
    join_words(s, ".", |text, _| text.to_lowercase())
}

/// Header-Case: `User-Profile`, `Http-Status`
pub(crate) fn to_header_case(s: &str) -> String {
    join_words(s, "-", |text, _| {
        let mut result = String::with_capacity(text.len());
        capitalize_word(text, &mut result);
        result
    })
}

/// no case: `user profile`, `http status`
pub(crate) fn to_no_case(s: &str) -> String {
    join_words(s, " ", |text, _| text.to_lowercase())
}

/// param-case (kebab-case): `user-profile`, `http-status`
pub(crate) fn to_param_case(s: &str) -> String {
    join_words(s, "-", |text, _| text.to_lowercase())
}

/// path/case: `user/profile`, `http/status`
pub(crate) fn to_path_case(s: &str) -> String {
    join_words(s, "/", |text, _| text.to_lowercase())
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Shared helper: split words, transform each, join with separator.
fn join_words(s: &str, separator: &str, transform: impl Fn(&str, usize) -> String) -> String {
    split_into_words(s)
        .iter()
        .enumerate()
        .map(|(i, w)| transform(&w.text, i))
        .collect::<Vec<_>>()
        .join(separator)
}

/// Capitalize first char, lowercase rest → push into result.
fn capitalize_word(text: &str, result: &mut String) {
    let mut chars = text.chars();
    if let Some(first) = chars.next() {
        result.extend(first.to_uppercase());
        for c in chars {
            result.push(c.to_ascii_lowercase());
        }
    }
}

/// Lowercase all chars → push into result.
fn lowercase_word(text: &str, result: &mut String) {
    for c in text.chars() {
        result.push(c.to_ascii_lowercase());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pascal_case() {
        assert_eq!(to_pascal_case("user_name", true), "UserName");
        assert_eq!(to_pascal_case("HTTP_STATUS", true), "HttpStatus");
        assert_eq!(to_pascal_case("UserProfile", true), "UserProfile");

        // Preserve underscores
        assert_eq!(to_pascal_case("user_name", false), "User_Name");
        assert_eq!(to_pascal_case("HTTP_STATUS", false), "Http_Status");
    }

    #[test]
    fn test_camel_case() {
        assert_eq!(to_camel_case("user_name", true), "userName");
        assert_eq!(to_camel_case("HTTP_STATUS", true), "httpStatus");
        assert_eq!(to_camel_case("UserProfile", true), "userProfile");

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
    fn test_capital_case() {
        assert_eq!(to_capital_case("userName"), "User Name");
        assert_eq!(to_capital_case("user_name"), "User Name");
    }

    #[test]
    fn test_dot_case() {
        assert_eq!(to_dot_case("userName"), "user.name");
        assert_eq!(to_dot_case("UserProfile"), "user.profile");
    }

    #[test]
    fn test_header_case() {
        assert_eq!(to_header_case("userName"), "User-Name");
        assert_eq!(to_header_case("user_name"), "User-Name");
    }

    #[test]
    fn test_param_case() {
        assert_eq!(to_param_case("userName"), "user-name");
        assert_eq!(to_param_case("UserProfile"), "user-profile");
    }

    #[test]
    fn test_path_case() {
        assert_eq!(to_path_case("userName"), "user/name");
        assert_eq!(to_path_case("UserProfile"), "user/profile");
    }

    #[test]
    fn test_no_case() {
        assert_eq!(to_no_case("userName"), "user name");
        assert_eq!(to_no_case("UserProfile"), "user profile");
    }
}
