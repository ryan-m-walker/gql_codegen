use std::path::{Path, PathBuf};

use apollo_compiler::{Name, ast::OperationType};

pub fn expand_path(
    input_path: &Path,
    current_path: &Path,
    operation_name: &Name,
    operation_type: &OperationType,
) -> PathBuf {
    let input_str = input_path.to_string_lossy();

    let filepath = current_path
        .parent()
        .map(|p| p.to_string_lossy())
        .unwrap_or_else(|| "".into());

    let filename = current_path
        .file_stem()
        .map(|s| s.to_string_lossy())
        .unwrap_or_else(|| "".into());

    let operation_type_str = operation_type.to_string().to_lowercase();

    let expanded_str = input_str
        .replace("${filepath}", &filepath)
        .replace("${filename}", &filename)
        .replace("${operation_name}", operation_name)
        .replace("${operation_type}", &operation_type_str);

    PathBuf::from(expanded_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filepath() {
        let input_path = Path::new("./${filepath}/__generated__/types.ts");
        let current_path = Path::new("src/components/TestComponent.tsx");
        let operation_name = Name::new("MyQuery").unwrap();
        let operation_type = OperationType::Query;

        let new_path = expand_path(input_path, current_path, &operation_name, &operation_type);

        assert_eq!(
            new_path,
            Path::new("./src/components/__generated__/types.ts")
        );
    }

    #[test]
    fn test_filename() {
        let input_path = Path::new("./__generated__/${filename}.graphql.ts");
        let current_path = Path::new("src/components/TestComponent.tsx");
        let operation_name = Name::new("MyQuery").unwrap();
        let operation_type = OperationType::Query;

        let new_path = expand_path(input_path, current_path, &operation_name, &operation_type);

        assert_eq!(
            new_path,
            Path::new("./__generated__/TestComponent.graphql.ts")
        );
    }

    #[test]
    fn test_operation_name() {
        let input_path = Path::new("./generated/${operation_name}.ts");
        let current_path = Path::new("src/queries/user.graphql");
        let operation_name = Name::new("GetUserProfile").unwrap();
        let operation_type = OperationType::Query;

        let new_path = expand_path(input_path, current_path, &operation_name, &operation_type);

        assert_eq!(new_path, Path::new("./generated/GetUserProfile.ts"));
    }

    #[test]
    fn test_operation_type_query() {
        let input_path = Path::new("./${operation_type}s/generated.ts");
        let current_path = Path::new("src/test.graphql");
        let operation_name = Name::new("TestQuery").unwrap();
        let operation_type = OperationType::Query;

        let new_path = expand_path(input_path, current_path, &operation_name, &operation_type);

        assert_eq!(new_path, Path::new("./querys/generated.ts"));
    }

    #[test]
    fn test_operation_type_mutation() {
        let input_path = Path::new("./${operation_type}s/generated.ts");
        let current_path = Path::new("src/test.graphql");
        let operation_name = Name::new("CreateUser").unwrap();
        let operation_type = OperationType::Mutation;

        let new_path = expand_path(input_path, current_path, &operation_name, &operation_type);

        assert_eq!(new_path, Path::new("./mutations/generated.ts"));
    }

    #[test]
    fn test_operation_type_subscription() {
        let input_path = Path::new("./${operation_type}s/generated.ts");
        let current_path = Path::new("src/test.graphql");
        let operation_name = Name::new("MessageAdded").unwrap();
        let operation_type = OperationType::Subscription;

        let new_path = expand_path(input_path, current_path, &operation_name, &operation_type);

        assert_eq!(new_path, Path::new("./subscriptions/generated.ts"));
    }

    #[test]
    fn test_multiple_variables_combined() {
        let input_path = Path::new(
            "${filepath}/__generated__/${operation_type}/${filename}_${operation_name}.ts",
        );
        let current_path = Path::new("src/components/UserProfile.tsx");
        let operation_name = Name::new("GetUserData").unwrap();
        let operation_type = OperationType::Query;

        let new_path = expand_path(input_path, current_path, &operation_name, &operation_type);

        assert_eq!(
            new_path,
            Path::new("src/components/__generated__/query/UserProfile_GetUserData.ts")
        );
    }

    #[test]
    fn test_all_variables_in_path() {
        let input_path =
            Path::new("${filepath}/${operation_type}/${filename}.${operation_name}.generated.ts");
        let current_path = Path::new("app/mutations/user.graphql");
        let operation_name = Name::new("CreateUserMutation").unwrap();
        let operation_type = OperationType::Mutation;

        let new_path = expand_path(input_path, current_path, &operation_name, &operation_type);

        assert_eq!(
            new_path,
            Path::new("app/mutations/mutation/user.CreateUserMutation.generated.ts")
        );
    }

    #[test]
    fn test_no_variables() {
        let input_path = Path::new("./static/generated.ts");
        let current_path = Path::new("src/test.graphql");
        let operation_name = Name::new("TestQuery").unwrap();
        let operation_type = OperationType::Query;

        let new_path = expand_path(input_path, current_path, &operation_name, &operation_type);

        assert_eq!(new_path, Path::new("./static/generated.ts"));
    }

    #[test]
    fn test_file_without_extension() {
        let input_path = Path::new("./${filename}.ts");
        let current_path = Path::new("src/queries/user");
        let operation_name = Name::new("GetUser").unwrap();
        let operation_type = OperationType::Query;

        let new_path = expand_path(input_path, current_path, &operation_name, &operation_type);

        assert_eq!(new_path, Path::new("./user.ts"));
    }

    #[test]
    fn test_file_at_root() {
        let input_path = Path::new("${filepath}/${filename}.generated.ts");
        let current_path = Path::new("schema.graphql");
        let operation_name = Name::new("RootQuery").unwrap();
        let operation_type = OperationType::Query;

        let new_path = expand_path(input_path, current_path, &operation_name, &operation_type);

        // When parent is None, filepath should be empty
        assert_eq!(new_path, Path::new("/schema.generated.ts"));
    }

    #[test]
    fn test_nested_deep_path() {
        let input_path = Path::new("${filepath}/generated/${filename}_${operation_type}.ts");
        let current_path = Path::new("src/very/deeply/nested/path/component.tsx");
        let operation_name = Name::new("DeepQuery").unwrap();
        let operation_type = OperationType::Query;

        let new_path = expand_path(input_path, current_path, &operation_name, &operation_type);

        assert_eq!(
            new_path,
            Path::new("src/very/deeply/nested/path/generated/component_query.ts")
        );
    }

    #[test]
    fn test_special_characters_in_names() {
        let input_path = Path::new("${filename}_${operation_name}.ts");
        let current_path = Path::new("src/test-component.tsx");
        let operation_name = Name::new("Get_User$Data").unwrap();
        let operation_type = OperationType::Query;

        let new_path = expand_path(input_path, current_path, &operation_name, &operation_type);

        assert_eq!(new_path, Path::new("test-component_Get_User$Data.ts"));
    }

    #[test]
    fn test_empty_operation_name() {
        let input_path = Path::new("${operation_name}.ts");
        let current_path = Path::new("src/test.graphql");
        let operation_name = Name::new("").unwrap();
        let operation_type = OperationType::Query;

        let new_path = expand_path(input_path, current_path, &operation_name, &operation_type);

        assert_eq!(new_path, Path::new(".ts"));
    }

    #[test]
    fn test_absolute_paths() {
        let input_path = Path::new("/absolute/${filepath}/${filename}.ts");
        let current_path = Path::new("/project/src/components/Header.tsx");
        let operation_name = Name::new("HeaderQuery").unwrap();
        let operation_type = OperationType::Query;

        let new_path = expand_path(input_path, current_path, &operation_name, &operation_type);

        assert_eq!(
            new_path,
            Path::new("/absolute//project/src/components/Header.ts")
        );
    }

    #[test]
    fn test_duplicate_variables() {
        let input_path = Path::new("${filename}_${filename}_${operation_name}.ts");
        let current_path = Path::new("src/test.tsx");
        let operation_name = Name::new("DuplicateTest").unwrap();
        let operation_type = OperationType::Query;

        let new_path = expand_path(input_path, current_path, &operation_name, &operation_type);

        assert_eq!(new_path, Path::new("test_test_DuplicateTest.ts"));
    }
}
