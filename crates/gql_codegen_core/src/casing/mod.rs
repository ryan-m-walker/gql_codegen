//! Naming convention and case transformation utilities.
//!
//! Provides types and functions for transforming GraphQL type names
//! to different casing styles (PascalCase, camelCase, CONSTANT_CASE, etc.).
//!
//! Supports the `change-case-all#functionName` string format from graphql-codegen
//! for backwards compatibility.

mod transforms;
mod words;

use std::borrow::Cow;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use transforms::*;

/// Naming convention configuration.
/// Supports both simple string format and detailed object configuration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum NamingConvention {
    /// Single convention applied to all names (e.g., "keep", "pascalCase")
    Simple(NamingCase),
    /// Detailed configuration with separate conventions for different outputs
    Detailed(NamingConventionConfig),
}

/// Custom deserializer that preserves our error messages.
/// `#[serde(untagged)]` swallows inner errors — this checks if the value is
/// a string (→ NamingCase) or object (→ NamingConventionConfig) explicitly.
impl<'de> Deserialize<'de> for NamingConvention {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de;

        struct NamingConventionVisitor;

        impl<'de> de::Visitor<'de> for NamingConventionVisitor {
            type Value = NamingConvention;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a string (e.g., \"pascalCase\") or an object (e.g., {\"typeNames\": \"pascalCase\"})")
            }

            fn visit_str<E: de::Error>(self, s: &str) -> Result<Self::Value, E> {
                match parse_naming_case(s) {
                    Ok((case, Some(warning))) => {
                        eprintln!("warning: {warning}");
                        Ok(NamingConvention::Simple(case))
                    }
                    Ok((case, None)) => Ok(NamingConvention::Simple(case)),
                    Err(err) => Err(de::Error::custom(err)),
                }
            }

            fn visit_string<E: de::Error>(self, s: String) -> Result<Self::Value, E> {
                self.visit_str(&s)
            }

            fn visit_map<M: de::MapAccess<'de>>(self, map: M) -> Result<Self::Value, M::Error> {
                let config = NamingConventionConfig::deserialize(
                    de::value::MapAccessDeserializer::new(map),
                )?;
                Ok(NamingConvention::Detailed(config))
            }
        }

        deserializer.deserialize_any(NamingConventionVisitor)
    }
}

impl Default for NamingConvention {
    fn default() -> Self {
        Self::Simple(NamingCase::PascalCase)
    }
}

/// Detailed naming convention configuration.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NamingConventionConfig {
    /// Convention for type names (interfaces, types, enums)
    #[serde(default)]
    pub type_names: Option<NamingCase>,

    /// Convention for enum values
    #[serde(default)]
    pub enum_values: Option<NamingCase>,

    /// When true, underscores are removed and treated as word boundaries.
    /// When false (default), underscores are preserved in output.
    #[serde(default)]
    pub transform_underscore: bool,
}

/// Available naming case transformations.
///
/// First-class cases (recommended):
/// - `keep`, `pascalCase`, `camelCase`, `constantCase`, `snakeCase`,
///   `upperCase`, `lowerCase`
///
/// Compat cases (supported for graphql-codegen compatibility):
/// - `capitalCase`, `dotCase`, `headerCase`, `paramCase`, `pathCase`, `noCase`
///
/// Also accepts `change-case-all#functionName` format strings.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum NamingCase {
    /// Keep original name unchanged
    Keep,
    /// PascalCase (e.g., MyTypeName)
    #[default]
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
    // Compat cases
    /// Capital Case (e.g., My Type Name)
    CapitalCase,
    /// dot.case (e.g., my.type.name)
    DotCase,
    /// Header-Case (e.g., My-Type-Name)
    HeaderCase,
    /// no case (e.g., my type name)
    NoCase,
    /// param-case / kebab-case (e.g., my-type-name)
    ParamCase,
    /// path/case (e.g., my/type/name)
    PathCase,
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
            Self::CapitalCase => Cow::Owned(to_capital_case(s)),
            Self::DotCase => Cow::Owned(to_dot_case(s)),
            Self::HeaderCase => Cow::Owned(to_header_case(s)),
            Self::NoCase => Cow::Owned(to_no_case(s)),
            Self::ParamCase => Cow::Owned(to_param_case(s)),
            Self::PathCase => Cow::Owned(to_path_case(s)),
        }
    }
}

/// Mapping of deprecated change-case-all function names to their closest supported case.
const DEPRECATED_CASES: &[(&str, NamingCase, &str)] = &[
    ("sentenceCase", NamingCase::CapitalCase, "capitalCase"),
    ("titleCase", NamingCase::CapitalCase, "capitalCase"),
    ("spongeCase", NamingCase::Keep, "keep"),
    ("localeLowerCase", NamingCase::Lowercase, "lowerCase"),
    ("localeUpperCase", NamingCase::Uppercase, "upperCase"),
    ("lowerCaseFirst", NamingCase::CamelCase, "camelCase"),
    ("upperCaseFirst", NamingCase::PascalCase, "pascalCase"),
];

/// Parse a naming case string, supporting both direct names and `change-case-all#fn` format.
/// Returns the NamingCase and an optional deprecation warning message.
fn parse_naming_case(s: &str) -> Result<(NamingCase, Option<String>), String> {
    // Strip `change-case-all#` or similar `module#` prefix
    let case_name = s.rsplit_once('#').map(|(_, name)| name).unwrap_or(s);

    // Check first-class + compat cases
    let case = match case_name {
        "keep" => return Ok((NamingCase::Keep, None)),
        "pascalCase" | "PascalCase" => return Ok((NamingCase::PascalCase, None)),
        "camelCase" => return Ok((NamingCase::CamelCase, None)),
        "constantCase" | "CONSTANT_CASE" => return Ok((NamingCase::ConstantCase, None)),
        "snakeCase" | "snake_case" => return Ok((NamingCase::SnakeCase, None)),
        "lowerCase" | "lowercase" => return Ok((NamingCase::Lowercase, None)),
        "upperCase" | "uppercase" | "UPPERCASE" => return Ok((NamingCase::Uppercase, None)),
        "capitalCase" => return Ok((NamingCase::CapitalCase, None)),
        "dotCase" => return Ok((NamingCase::DotCase, None)),
        "headerCase" | "Header-Case" => return Ok((NamingCase::HeaderCase, None)),
        "noCase" => return Ok((NamingCase::NoCase, None)),
        "paramCase" | "kebabCase" | "kebab-case" => return Ok((NamingCase::ParamCase, None)),
        "pathCase" => return Ok((NamingCase::PathCase, None)),
        _ => case_name,
    };

    // Check deprecated cases
    for (deprecated_name, mapped_case, suggested_name) in DEPRECATED_CASES {
        if case == *deprecated_name {
            let warning = format!(
                "naming convention '{case_name}' is deprecated and will be removed in a future version, \
                 using '{suggested_name}' instead"
            );
            return Ok((*mapped_case, Some(warning)));
        }
    }

    Err(format!(
        "unknown naming convention '{case_name}'. Valid options: \
         keep, camelCase, pascalCase, constantCase, snakeCase, upperCase, lowerCase, \
         capitalCase, dotCase, headerCase, noCase, paramCase, pathCase"
    ))
}

/// Custom deserializer for NamingCase that handles the `change-case-all#fn` format
/// and emits deprecation warnings to stderr.
impl<'de> Deserialize<'de> for NamingCase {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        match parse_naming_case(&s) {
            Ok((case, Some(warning))) => {
                eprintln!("warning: {warning}");
                Ok(case)
            }
            Ok((case, None)) => Ok(case),
            Err(err) => Err(serde::de::Error::custom(err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cow_borrowed_for_keep() {
        let input = "SomeTypeName";
        let result = NamingCase::Keep.apply(input, false);
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn test_parse_direct_names() {
        assert_eq!(
            parse_naming_case("pascalCase").unwrap().0,
            NamingCase::PascalCase
        );
        assert_eq!(
            parse_naming_case("camelCase").unwrap().0,
            NamingCase::CamelCase
        );
        assert_eq!(
            parse_naming_case("constantCase").unwrap().0,
            NamingCase::ConstantCase
        );
        assert_eq!(
            parse_naming_case("snakeCase").unwrap().0,
            NamingCase::SnakeCase
        );
        assert_eq!(parse_naming_case("keep").unwrap().0, NamingCase::Keep);
        assert_eq!(
            parse_naming_case("capitalCase").unwrap().0,
            NamingCase::CapitalCase
        );
        assert_eq!(
            parse_naming_case("paramCase").unwrap().0,
            NamingCase::ParamCase
        );
        assert_eq!(
            parse_naming_case("kebabCase").unwrap().0,
            NamingCase::ParamCase
        );
    }

    #[test]
    fn test_parse_change_case_all_format() {
        assert_eq!(
            parse_naming_case("change-case-all#pascalCase").unwrap().0,
            NamingCase::PascalCase
        );
        assert_eq!(
            parse_naming_case("change-case-all#constantCase").unwrap().0,
            NamingCase::ConstantCase
        );
        assert_eq!(
            parse_naming_case("change-case-all#snakeCase").unwrap().0,
            NamingCase::SnakeCase
        );
    }

    #[test]
    fn test_parse_arbitrary_module_prefix() {
        // Any `module#fn` format should work
        assert_eq!(
            parse_naming_case("my-custom-lib#pascalCase").unwrap().0,
            NamingCase::PascalCase
        );
    }

    #[test]
    fn test_parse_deprecated_cases() {
        let (case, warning) = parse_naming_case("spongeCase").unwrap();
        assert_eq!(case, NamingCase::Keep);
        assert!(warning.unwrap().contains("deprecated"));

        let (case, warning) = parse_naming_case("titleCase").unwrap();
        assert_eq!(case, NamingCase::CapitalCase);
        assert!(warning.unwrap().contains("capitalCase"));

        let (case, warning) = parse_naming_case("change-case-all#lowerCaseFirst").unwrap();
        assert_eq!(case, NamingCase::CamelCase);
        assert!(warning.unwrap().contains("camelCase"));
    }

    #[test]
    fn test_parse_unknown_case() {
        let err = parse_naming_case("totallyFakeCase").unwrap_err();
        assert!(err.contains("unknown naming convention 'totallyFakeCase'"));
        assert!(err.contains("Valid options:"));
        assert!(err.contains("pascalCase"));
    }

    #[test]
    fn test_serde_roundtrip() {
        let json = r#""pascalCase""#;
        let case: NamingCase = serde_json::from_str(json).unwrap();
        assert_eq!(case, NamingCase::PascalCase);
    }

    #[test]
    fn test_serde_change_case_format() {
        let json = r#""change-case-all#constantCase""#;
        let case: NamingCase = serde_json::from_str(json).unwrap();
        assert_eq!(case, NamingCase::ConstantCase);
    }

    #[test]
    fn test_serde_in_convention() {
        let json = r#"{"typeNames": "change-case-all#snakeCase", "enumValues": "constantCase"}"#;
        let config: NamingConventionConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.type_names, Some(NamingCase::SnakeCase));
        assert_eq!(config.enum_values, Some(NamingCase::ConstantCase));
    }
}
