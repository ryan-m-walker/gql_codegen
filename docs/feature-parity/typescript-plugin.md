# TypeScript Plugin Feature Parity

Comparison of SGC (Speedy GraphQL Codegen) options vs [graphql-codegen TypeScript plugin](https://github.com/dotansimha/graphql-code-generator/tree/master/packages/plugins/typescript/typescript).

Last updated: 2026-02-04

## Summary

| Status             | Count |
|--------------------|-------|
| ✅ Supported       | 26    |
| 🔶 Partial         | 2     |
| ❌ Not Implemented | 12    |

## Scalar Configuration

| Option              | SGC | graphql-codegen | Notes                             |
|---------------------|-----|-----------------|-----------------------------------|
| `scalars`           | ✅  | ✅              | Custom scalar type mappings       |
| `strictScalars`     | ✅  | ✅              | Error on unmapped scalars         |
| `defaultScalarType` | ✅  | ✅              | Fallback type for unknown scalars |

## Type Generation

| Option                       | SGC | graphql-codegen | Notes                        |
|------------------------------|-----|-----------------|------------------------------|
| `declarationKind`            | ✅  | ✅              | `interface` vs `type`        |
| `immutableTypes`             | ✅  | ✅              | Add `readonly` modifier      |
| `typesPrefix`                | ✅  | ✅              | Prefix for all type names    |
| `typesSuffix`                | ✅  | ✅              | Suffix for all type names    |
| `noExport`                   | ✅  | ✅              | Skip `export` keyword        |
| `wrapEntireFieldDefinitions` | ❌  | ✅              | Wrap entire field definition |
| `entireFieldWrapperValue`    | ❌  | ✅              | Wrapper template             |
| `wrapFieldDefinitions`       | ❌  | ✅              | Wrap field definitions       |
| `fieldWrapperValue`          | ❌  | ✅              | Field wrapper template       |

## Enum Configuration

| Option                 | SGC | graphql-codegen | Notes                                            |
|------------------------|-----|-----------------|--------------------------------------------------|
| `enumsAsTypes`         | ✅  | ✅              | String union instead of enum                     |
| `enumsAsConst`         | 🔶  | ✅              | `as const` objects (config exists, gen partial)  |
| `futureProofEnums`     | ✅  | ✅              | Add `'%future added value'`                      |
| `constEnums`           | ✅  | ✅              | Use `const enum`                                 |
| `enumPrefix`           | ✅  | ✅              | Prefix for enum names                            |
| `enumSuffix`           | ✅  | ✅              | Suffix for enum names                            |
| `onlyEnums`            | ❌  | ✅              | Only generate enums                              |
| `allowEnumStringTypes` | ❌  | ✅              | Allow string enum values                         |
| `enumValues`           | ❌  | ✅              | Custom enum value mappings                       |

## Nullability / Optionals

| Option            | SGC | graphql-codegen | Notes                      |
|-------------------|-----|-----------------|----------------------------|
| `avoidOptionals`  | ✅  | ✅              | Use `null` instead of `?`  |
| `maybeValue`      | ✅  | ✅              | Custom Maybe type template |
| `inputMaybeValue` | ✅  | ✅              | Separate Maybe for inputs  |

## Typename Configuration

| Option               | SGC | graphql-codegen | Notes                     |
|----------------------|-----|-----------------|---------------------------|
| `skipTypename`       | ✅  | ✅              | Omit `__typename` field   |
| `nonOptionalTypename`| ✅  | ✅              | Make `__typename` required|

## Naming Conventions

| Option                                 | SGC | graphql-codegen | Notes                          |
|----------------------------------------|-----|-----------------|--------------------------------|
| `namingConvention`                     | ✅  | ✅              | Case transformation            |
| `namingConvention.typeNames`           | ✅  | ✅              | Convention for type names      |
| `namingConvention.enumValues`          | ✅  | ✅              | Convention for enum values     |
| `namingConvention.transformUnderscore` | ✅  | ✅              | Remove underscores in transform|

**Supported cases:** `keep`, `pascalCase`, `camelCase`, `constantCase`, `snakeCase`, `lowercase`, `uppercase`

## Union Configuration

| Option             | SGC | graphql-codegen | Notes                             |
|--------------------|-----|-----------------|-----------------------------------|
| `futureProofUnions`| ✅  | ❌              | SGC-specific: future-proof unions |

## Import/Export

| Option            | SGC | graphql-codegen | Notes                            |
|-------------------|-----|-----------------|---------------------------------|
| `useTypeImports`  | ✅  | ✅              | Use `import type` syntax        |
| `preResolveTypes` | ❌  | ✅              | Resolve types before generation |

## Document/Operation Options

| Option                         | SGC | graphql-codegen | Notes                        |
|--------------------------------|-----|-----------------|------------------------------|
| `onlyOperationTypes`           | ✅  | 🔶              | SGC has improved implementation (see below)|

### `onlyOperationTypes` Implementation Differences

SGC's implementation of `onlyOperationTypes` is significantly more sophisticated than graphql-codegen's:

**graphql-codegen approach:**
Simply skips entire type categories (objects, interfaces, unions, inputs) and only keeps enums and scalars - regardless of whether they're actually used. This has [known issues](https://github.com/dotansimha/graphql-code-generator/issues/4562) with users reporting 2MB+ generated files.

**SGC approach:**
Performs transitive closure analysis to include only types genuinely referenced by operations:
1. Collects types directly referenced in operations and fragments
2. Collects operation variable input types
3. Transitively expands to include all field return types
4. Includes union members and interface implementers

This is what users actually want, as evidenced by:
- [Feature request #4562](https://github.com/dotansimha/graphql-code-generator/issues/4562) - "Improve onlyOperationTypes implementation"
- [Bug report #9665](https://github.com/dotansimha/graphql-code-generator/issues/9665) - "onlyOperationTypes generates unused input types"
- [Third-party plugin](https://github.com/Stonepaw/graphql-codegen-typescript-operation-types) created to work around the limitation

| Type Category | graphql-codegen | SGC |
|---------------|-----------------|-----|
| Objects       | ❌ Skip all     | ✅ If used |
| Interfaces    | ❌ Skip all     | ✅ If used |
| Unions        | ❌ Skip all     | ✅ If used |
| Inputs        | ❌ Skip all     | ✅ If used |
| Enums         | ✅ Keep all     | ✅ If used |
| Scalars       | ✅ Keep all     | ✅ If used |
| `dedupeFragments`              | 🔶  | ✅              | We have `dedupeSelections`   |
| `externalFragments`            | ❌  | ✅              | External fragment definitions|
| `fragmentVariableSuffix`       | ❌  | ✅              | Suffix for fragment variables|
| `exportFragmentSpreadSubTypes` | ❌  | ✅              | Export fragment spread types |
| `addUnderscoreToArgsType`      | ❌  | ✅              | Underscore prefix for args   |

## Other

| Option                                    | SGC | graphql-codegen | Notes                          |
|-------------------------------------------|-----|-----------------|--------------------------------|
| `noSchemaStitching`                       | ❌  | ✅              | Disable schema stitching types |
| `skipDocumentsValidation`                 | ❌  | ✅              | Skip document validation       |
| `directiveArgumentAndInputFieldMappings`  | ❌  | ✅              | Directive mappings             |

## SGC-Specific Options

These options are unique to SGC:

| Option             | Description                                        |
|--------------------|----------------------------------------------------|
| `inlineFragments`  | Inline fragment spreads into document text         |
| `dedupeSelections` | Remove duplicate field selections                  |
| `graphqlTag`       | GraphQL tag style (`gql`, `graphql`, `none`)       |
| `formatting`       | Code formatting options (indent, tabs, quotes)     |
| `futureProofUnions`| Add future-proof entry to union types              |

## Priority Roadmap

### High Priority (commonly used)
1. ~~`enumPrefix` / `enumSuffix`~~ ✅ Done
2. ~~`constEnums`~~ ✅ Done
3. ~~`noExport`~~ ✅ Done
4. ~~`onlyOperationTypes`~~ ✅ Done

### Medium Priority
5. `enumValues` - Custom enum mappings
6. `preResolveTypes` - Type resolution control
7. `addUnderscoreToArgsType` - Naming convention

### Low Priority (niche use cases)
8. Field wrapper options
9. `externalFragments`
10. Schema stitching options
