/// A word segment with info about whether it was preceded by an underscore.
pub(crate) struct Word {
    pub text: String,
    pub preceded_by_underscore: bool,
}

/// Split string into words at underscores and case boundaries.
///
/// Examples:
/// - `"userId"` → `["user", "Id"]`
/// - `"UserProfile"` → `["User", "Profile"]`
/// - `"user_name"` → `["user", "name"]`
/// - `"HTTP_STATUS"` → `["HTTP", "STATUS"]`
pub(crate) fn split_into_words(s: &str) -> Vec<Word> {
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
}
