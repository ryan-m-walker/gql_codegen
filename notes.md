``` json
"generates": {
    "<filename>.graphql.ts": {
        //
    },
    "{{filepath}}/__generated__/{{name}}.graphql.ts": {

    }
}
```


⏺ String interning is a technique where you store only one copy of each unique string value in memory and use references/handles to that shared
  copy instead of duplicating strings.

  How it works:

  // Without interning - each string is allocated separately
  let name1 = "User".to_string();
  let name2 = "User".to_string(); // Duplicate allocation
  let name3 = "User".to_string(); // Another duplicate

  // With interning - only one "User" string exists in memory
  let interner = StringInterner::new();
  let user_id = interner.intern("User");   // Returns ID/handle
  let user_id2 = interner.intern("User");  // Returns same ID
  assert_eq!(user_id, user_id2); // Same handle, no new allocation

  Benefits for your codebase:

  Looking at your code, you repeatedly use the same GraphQL type names, field names, and generated code patterns:

  // Your current code does this a lot:
  field_name.to_string()           // main.rs:148
  fragment.name.clone()            // main.rs:223
  node.name.to_string()           // ts_schema_types/mod.rs:182

  With interning, these would become cheap ID lookups instead of string allocations.

  Popular Rust crates:

  - string-interner - Most common, simple API
  - lasso - High performance, good for hot paths
  - internment - Automatic deduplication

  Example implementation:

  use string_interner::StringInterner;

  struct CodeGenerator {
      interner: StringInterner,
      // ... other fields
  }

  impl CodeGenerator {
      fn generate_type_name(&mut self, name: &str) -> Symbol {
          self.interner.get_or_intern(name)
      }

      fn get_string(&self, symbol: Symbol) -> &str {
          self.interner.resolve(symbol).unwrap()
      }
  }

  For your GraphQL codegen where you're processing the same type/field names repeatedly across multiple files, this could reduce memory usage by
   50-80% and speed up string comparisons significantly.
