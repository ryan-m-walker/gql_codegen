# SGC (Speedy GraphQL Codegen) Roadmap

## Current State

**Working:**
- JSON + TypeScript config support (via Node.js wrapper)
- Schema parsing via apollo-compiler
- Relay-style GraphQL extraction (works with broken JS/TS syntax)
- Three generators: `typescript`, `typescript-operations`, `documents`
- Parallel file processing with Rayon
- Two-phase caching (metadata check → content hash)
- Writer trait abstraction (`FsWriter`, `MemoryWriter`, `StdoutWriter`)
- CLI flags: `--check`, `--stdout`, `--no-cache`, `--clean`
- Glob negation patterns (`!node_modules/**`)
- `with_capacity` pre-allocation optimizations

---

## Roadmap

### Phase 1: Config Compatibility
> Align with graphql-codegen where it makes sense

#### 1.1 Plugin Config Options
- [ ] `maybeValue` - customize nullable type wrapper
- [ ] `avoidOptionals` - use `T | null` instead of `T?`
- [ ] `preResolveTypes` - inline fragment types
- [ ] `nonOptionalTypename` - __typename always required

#### 1.2 Additional Plugins
- [ ] `typescript-resolvers` - generate resolver type signatures
- [ ] `fragment-matcher` - generate introspection result for Apollo Client
- [ ] `add` plugin - prepend/append content to output

---

### Phase 2: Dynamic Path Variables
> Advanced path templating for flexible output organization

#### 2.1 Path Variables
**Current:** Static output paths only

**Proposed:**
- `{{opName}}` - operation name
- `{{opType}}` - query/mutation/subscription
- `{{fragmentName}}` - fragment name
- `{{sourceDir}}` - directory of source file
- `{{sourceFile}}` - source filename (no extension)

#### 2.2 Near-Operation-File Pattern
```
src/
  components/
    UserProfile/
      UserProfile.tsx          # contains gql query
      UserProfile.generated.ts # generated types (auto)
```

**Tasks:**
- [ ] Implement path template parsing (`{{var}}` syntax)
- [ ] Implement per-document output files
- [ ] Handle fragment deduplication across files
- [ ] Generate import statements for cross-file references

---

### Phase 3: Incremental Caching
> Only regenerate what changed - prerequisite for efficient watch mode

**Current Problems:**
- All-or-nothing: if any file changes, regenerate everything
- Hash ALL files matching globs, even those without GraphQL
- No dependency graph to know what to regenerate

**Design:**

1. **Track which files contain GraphQL**
   - Only hash content for files that yielded documents
   - Still track metadata for ALL files (detect new GraphQL)

2. **Build dependency graph**
   ```
   UserQuery (UserProfile.tsx)
     → uses UserFieldsFragment
     → contributes to: types.ts, documents.ts
   ```

3. **On change:**
   - Phase 1: stat all matched files
   - Phase 2: re-extract only changed files
   - Phase 3: diff documents against cache
   - Phase 4: regenerate only affected outputs

**Tasks:**
- [ ] Track which files yield documents during extraction
- [ ] Only store content hashes for GraphQL files
- [ ] Build dependency graph (operations → fragments)
- [ ] Implement partial regeneration
- [ ] Handle fragment change propagation

---

### Phase 4: Developer Experience

#### 4.1 Zero-Config Mode
> Just run `sgc` and it works

- [ ] Auto-discover schema files (`**/schema.graphql`, `**/*.graphql`)
- [ ] Auto-discover operation files (`src/**/*.graphql`, `**/*.tsx`, etc.)
- [ ] Default output to `__generated__/types.ts`
- [ ] Detect common project patterns (Next.js, Apollo, Relay)

#### 4.2 Init Command
> Easy onboarding with `sgc init`

- [ ] Interactive prompts for schema/documents/output paths
- [ ] Preset selection (sgc vs graphql-codegen compat)
- [ ] Generate `codegen.json` with sensible defaults
- [ ] Option for non-interactive mode with defaults
- [ ] Detect existing `.graphqlrc` and offer migration

#### 4.3 Watch Mode
> Depends on incremental caching

- [ ] Add `--watch` flag
- [ ] Use `notify` crate for filesystem watching
- [ ] Debounce rapid changes
- [ ] Clear terminal and show summary on each run

#### 4.4 Error Messages
- [ ] Pretty-print errors with code snippets (miette)
- [ ] Validation errors with suggestions
- [ ] Schema/document compatibility checking

#### 4.3 Benchmarking
- [ ] Create benchmark fixtures (100-500+ operations)
- [ ] Compare against `npx graphql-codegen`
- [ ] Document results in README

---

### Phase 5: Advanced Features

#### 5.1 Client Presets
- [ ] Apollo Client preset
- [ ] urql preset
- [ ] React Query preset (hooks generation)

#### 5.2 Schema Features
- [x] Programmatic schema support (.ts/.js exports → Node imports via esbuild, SDL passed to Rust)
- [x] `codegenScalarType` extension extraction from programmatic schemas
- [ ] Rust `schema_content` field — accept pre-resolved SDL alongside file paths
- [ ] Remote schema introspection (URL → introspection query → SDL, Rust-native)
- [ ] JSON introspection file as schema input
- [ ] Schema glob expansion in Rust (like documents)
- [ ] Custom schema loaders (.ts/.js files exporting a loader function)
- [ ] `ignoreEnumValuesFromSchema` config (only meaningful with programmatic schemas — no-op for SDL)
- [ ] Multiple schema sources
- [ ] Federation directive handling

#### 5.2.1 Node-side Config Caching
- [ ] Cache entire resolved config JSON in `.sgc/config-cache.json`
- [ ] Invalidation via mtime+size of all input files (config, schema sources, loaders)
- [ ] Use esbuild `metafile` to get full dependency graph of .ts/.js schema files
- [ ] On cache hit: skip config loading, esbuild, schema resolution — pass cached JSON directly to core

#### 5.3 Custom Plugin API
```typescript
export const myPlugin: Plugin = {
  name: 'my-plugin',
  generate({ schema, documents, config }) {
    return `// Custom generated code`
  }
}
```

#### 5.4 Cache-Informed Allocation
Use previous run metrics to pre-size allocators:
```rust
pub struct SizeHints {
    pub total_output_bytes: usize,
    pub operation_count: usize,
    pub fragment_count: usize,
}
```

---

## Priority

| Phase | Effort | Impact | Priority |
|-------|--------|--------|----------|
| Phase 4.1-4.2 (Zero-Config/Init) | Low-Medium | Very High | **P1** |
| Phase 2 (Dynamic Paths) | Medium | Very High | **P1** |
| Phase 3 (Incremental) | Medium-High | Very High | **P1** |
| Phase 4.3 (Watch) | Medium | High | **P1** |
| Phase 1 (Config Compat) | Low | Medium | P2 |
| Phase 4.4-4.5 | Medium | Medium | P2 |
| Phase 5 | High | Medium | P3 |

---

## Open Questions

1. **Binary distribution**: GitHub releases? npm binary packages? Homebrew?
2. **Plugin extensibility**: Curated set vs arbitrary plugins?
3. **Naming**: Keep `sgc` or rebrand?


DX: Warning

- Show warning when using conflicting options
- Show warning when writing files to wrong ext (gql -> ts for example)
