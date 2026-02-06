# Generator Notes

## avoidOptionals Flag Mapping

Reference: graphql-codegen `visitor-plugin-common` + `typescript` plugin

Each flag in `NormalizedAvoidOptionals` controls a **different code generation context**:

| Flag           | Context                        | JS Location                                  |
|----------------|--------------------------------|----------------------------------------------|
| `field`        | Schema object type fields      | `FieldDefinition()` in visitor.ts            |
| `inputValue`   | Schema input type fields       | `InputValueDefinition()` in visitor.ts       |
| `defaultValue` | Default values on input fields | `InputValueDefinition()` in visitor.ts       |
| `object`       | Operation variable types       | `getAvoidOption()` in variables-to-object.ts |

### field

Controls `?` on object type field definitions (e.g. `type Query { user?: ... }`).

```typescript
// visitor.ts FieldDefinition()
const addOptionalSign = !avoidOptionals.field && type.kind !== NON_NULL_TYPE;
```

### inputValue + defaultValue

Controls `?` on input type field definitions (e.g. `input Foo { bar?: ... }`).

```typescript
// visitor.ts InputValueDefinition()
const addOptionalSign =
  !avoidOptionals.inputValue &&
  (type.kind !== NON_NULL_TYPE ||
    (!avoidOptionals.defaultValue && node.defaultValue !== undefined));
```

Key behavior: a non-null field with a default value is optional **unless**
`avoidOptionals.defaultValue` is true. Nullable fields are optional **unless**
`avoidOptionals.inputValue` is true.

### object

Controls `?` on **operation variable types** (`GetUserQueryVariables`), NOT schema types.
This is used in the variables-to-object transformer, not the schema visitor.

```typescript
// typescript-variables-to-object.ts getAvoidOption()
return ((options.object || !options.defaultValue) && hasDefaultValue)
    || (!options.object && !isNonNullType);
```

When `object = true`: only fields with default values get `?` (nullable alone doesn't).
When `object = false` (default): nullable fields and fields with defaults both get `?`.
