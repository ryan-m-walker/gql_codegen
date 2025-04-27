pub(crate) enum FileType {
    GraphQL,
    JavaScript,
    TypeScript,
}

pub(crate) fn get_file_type(exension: &str) -> Option<FileType> {
    match exension {
        "graphql" | "gql" => Some(FileType::GraphQL),
        "js" | "mjs" | "jsx" | "cjs" => Some(FileType::JavaScript),
        "ts" | "mts" | "tsx" | "cts" => Some(FileType::TypeScript),
        _ => None,
    }
}
