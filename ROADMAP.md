# gql_codegen Roadmap

## Current State Summary

**What's Working:**
- JSON-based configuration with glob patterns
- Schema parsing via apollo-compiler
- Document extraction from `.graphql` and JS/TS files (via OXC)
- Three generators: `ts_schema_types`, `ts_operation_types`, `documents`
- Parallel file processing with Rayon
- Basic path templating (`${filepath}`, `${filename}`, `${operation_name}`, `${operation_type}`)
- Configurable formatting (indent, quotes, semicolons)

**Immediate Issues (Build Errors):**
- `selection_refs` field renamed to `root_selection_refs`/`type_selection_refs` but not updated everywhere
- Unused imports and mut warnings

---

## Roadmap

### Phase 0: Stabilization (Pre-requisite)
> Fix current build errors and clean up debug artifacts

- [ ] Fix `selection_refs` field references in `ts_operation_types/mod.rs`
- [ ] Remove debug panics (`if fragment.name.as_str() == "UserPhoneSettingsContentSection"`)
- [ ] Remove `dbg!()` calls
- [ ] Fix unused imports/mut warnings
- [ ] Add basic error handling (replace `.unwrap()` with proper error propagation)

---

### Phase 0.5: Relay-Style GraphQL Extraction
> Replace OXC-based extraction with resilient string scanning

**Why this matters:**
The current OXC approach requires valid JS/TS syntax. If you're mid-edit with a broken
import or missing semicolon, the entire file fails to parse and you lose codegen for
that file. Relay's approach extracts GraphQL even from syntactically invalid files.

**Reference:** https://github.com/facebook/relay/blob/main/compiler/crates/extract-graphql/src/lib.rs

**Algorithm:**
1. Quick check: if source doesn't contain "graphql" or "gql", skip entirely
2. Scan character-by-character with position tracking
3. When `g` found, try to match `graphql` or `gql`
4. Skip whitespace, find opening backtick
5. Extract until closing backtick (handling escapes)
6. Skip string literals and comments to avoid false positives

**Tasks:**
- [ ] Create new `gql_codegen_extract` crate (or rename `gql_codegen_js`)
- [ ] Implement `CharReader` with line/column tracking
- [ ] Implement string literal skipping (single, double, template)
- [ ] Implement comment skipping (`//` and `/* */`)
- [ ] Implement `graphql`/`gql` tag detection
- [ ] Implement backtick content extraction
- [ ] Return `Vec<ExtractedDocument>` with source locations
- [ ] Add tests with intentionally broken JS/TS files
- [ ] Benchmark against OXC approach

**Benefits:**
- Works during active editing (huge DX win for watch mode)
- Faster (no AST construction)
- Simpler code (~200 lines vs full parser dep)
- Smaller binary (can drop OXC dependency)

---

### Phase 1: GraphQL Codegen Compatibility
> Align configuration format and generator naming with The Guild's graphql-codegen

#### 1.1 Config Format Alignment

**Current format:**
```json
{
  "src": ".",
  "schemas": ["./schema.graphql"],
  "documents": "./src/**/*.graphql",
  "outputs": {
    "./generated/types.ts": {
      "generators": [{ "name": "ts_schema_types", "config": {} }]
    }
  }
}
```

**Target format (graphql-codegen compatible):**
```yaml
schema: "./schema.graphql"
documents: "./src/**/*.graphql"
generates:
  ./generated/types.ts:
    plugins:
      - typescript
      - typescript-operations
    config:
      scalars:
        DateTime: Date
```

**Tasks:**
- [ ] Rename `outputs` → `generates`
- [ ] Rename `generators` → `plugins`
- [ ] Support single schema string or array
- [ ] Add YAML config support (via `serde_yaml`)
- [ ] Rename generators to match graphql-codegen:
  - `ts_schema_types` → `typescript`
  - `ts_operation_types` → `typescript-operations`
  - `documents` → `typed-document-node` (or similar)

#### 1.2 Plugin Config Compatibility

Match the config options from graphql-codegen's typescript plugin:
- [ ] `enumsAsTypes` (currently `use_native_enums` inverted)
- [ ] `futureProofEnums` (already have this)
- [ ] `immutableTypes` (currently `readonly`)
- [ ] `maybeValue` - customize nullable type wrapper
- [ ] `avoidOptionals` - use `T | null` instead of `T?`
- [ ] `onlyOperationTypes` - skip schema types
- [ ] `preResolveTypes` - inline fragment types
- [ ] `skipTypename` - omit __typename fields
- [ ] `nonOptionalTypename` - __typename always required
- [ ] `arrayInputCoercion` - handle array input coercion

#### 1.3 Additional Plugins to Implement

- [ ] `typescript-resolvers` - generate resolver type signatures
- [ ] `fragment-matcher` - generate introspection result for Apollo Client
- [ ] `near-operation-file-preset` - co-locate generated types with operations
- [ ] `add` plugin - prepend/append content to output

---

### Phase 2: TypeScript Config Support
> Allow `codegen.config.ts` as configuration file

#### Research: How Other Tools Do It

| Tool | Approach | Pros | Cons |
|------|----------|------|------|
| **Vite** | Uses `esbuild` to transpile config, then `import()` | Fast, native ESM | Requires Node.js runtime |
| **graphql-codegen** | Uses `cosmiconfig` + `ts-node` or `tsx` | Ecosystem standard | Heavy deps, slow startup |
| **Rollup** | Bundles with `@rollup/plugin-typescript` | Works everywhere | Complex |
| **NAPI-RS** | Compile Rust to Node native module | True Rust performance | Requires Node.js, complex build |
| **Biome/oxlint** | JSON-only config | Simple, fast | No TS config |
| **swc** | JSON config with `--config-file` | Fast | No TS config |

**Recommended Approach: Hybrid**

Option A: **Standalone Rust binary + Optional Node.js wrapper**
```
gql-codegen (Rust binary) ← reads JSON/YAML only
@gql-codegen/cli (npm package) ← wraps binary, handles TS config
```

The npm wrapper would:
1. Check if config is `.ts`
2. If yes: transpile with esbuild/swc, eval, write temp JSON
3. Call Rust binary with JSON config
4. Clean up temp file

Option B: **NAPI-RS for full Node.js integration**
- Compile Rust core to native Node module
- Export `generateFromConfig(config: Config)` function
- Let Node.js handle all config loading

**Tasks:**
- [ ] Research: Decide between Option A (wrapper) vs Option B (NAPI)
- [ ] If Option A:
  - [ ] Create `@gql-codegen/cli` npm package
  - [ ] Add esbuild as dep for TS config transpilation
  - [ ] Create binary distribution (via `@gql-codegen/cli-darwin-arm64` etc.)
- [ ] If Option B:
  - [ ] Add napi-rs to workspace
  - [ ] Create `gql_codegen_node` crate
  - [ ] Define TypeScript types for config
  - [ ] Build and publish to npm

**TypeScript Config Benefits:**
```typescript
// codegen.config.ts
import type { CodegenConfig } from '@gql-codegen/cli'
import {
  near-operation-file-preset
} from '@gql-codegen/near-operation-file-preset'

const config: CodegenConfig = {
  schema: process.env.SCHEMA_URL ?? './schema.graphql',
  documents: ['./src/**/*.tsx', '!./src/generated/**'],
  generates: {
    './src/': {
      preset: nearOperationFilePreset,
      plugins: ['typescript-operations'],
    }
  },
  hooks: {
    afterAllFileWrite: ['prettier --write']
  }
}

export default config
```

---

### Phase 3: Smart Dynamic Path Variables
> Advanced path templating for flexible output organization

#### 3.1 Enhanced Path Variables

**Current variables:**
- `${filepath}` - directory of source file
- `${filename}` - filename without extension
- `${operation_name}` - operation name
- `${operation_type}` - query/mutation/subscription

**Proposed additions:**
- `${fragment_name}` - for fragment files
- `${relative_path}` - path relative to documents root
- `${schema_name}` - when using multiple schemas (federated)
- `${hash}` - content hash for cache busting
- `${date}` - generation date

#### 3.2 Path Transforms

```json
{
  "generates": {
    "./${filepath}/__generated__/${filename}.generated.ts": {
      "transforms": {
        "filepath": "kebab-case",
        "filename": "PascalCase"
      }
    }
  }
}
```

**Tasks:**
- [ ] Add `${fragment_name}` variable
- [ ] Add `${relative_path}` variable (relative to documents glob root)
- [ ] Add path transform functions (kebab-case, PascalCase, camelCase)
- [ ] Add conditional path segments: `${operation_type === 'mutation' ? 'mutations' : 'queries'}`
- [ ] Support glob in output paths for "near-operation-file" pattern

#### 3.3 Near-Operation-File Preset

The killer feature of smart paths - co-locate generated types with source:

```
src/
  components/
    UserProfile/
      UserProfile.tsx          # contains `gql` query
      UserProfile.generated.ts # generated types (auto)
```

**Config:**
```json
{
  "generates": {
    "./${filepath}/${filename}.generated.ts": {
      "plugins": ["typescript-operations"],
      "documentsGlob": "./${filepath}/${filename}.{ts,tsx}"
    }
  }
}
```

**Tasks:**
- [ ] Implement per-operation output files (currently aggregates all to one file)
- [ ] Add `documentsGlob` per-output to scope which documents go where
- [ ] Handle fragment deduplication across multiple output files
- [ ] Add import generation for cross-file fragment references

---

### Phase 3.5: Incremental Caching
> Only regenerate what changed - prerequisite for efficient watch mode

**Current State:**
- All-or-nothing caching: if any file changes, regenerate everything
- Hash ALL files matching globs, even those without GraphQL documents
- No tracking of which inputs affect which outputs

**Problems:**
- False cache invalidations when non-GraphQL code changes in matched files
- For incremental caching, storing metadata for non-GraphQL files is wasteful bloat
- No dependency graph to know what to regenerate

**Design:**

1. **Track which files actually contain GraphQL**
   - After extraction, record which files yielded documents
   - Only hash content for files that contributed
   - Still track metadata for ALL files (to detect new GraphQL additions)

2. **Build dependency graph**
   ```
   UserQuery (UserProfile.tsx:15)
     → uses FragmentA (fragments.ts:10)
     → contributes to: types.ts, documents.ts
   ```

3. **On change detection:**
   - Phase 1: stat ALL matched files (fast metadata check)
   - Phase 2: For changed files only, re-extract documents
   - Phase 3: Diff extracted documents against cached
   - Phase 4: Regenerate only affected outputs

**Cache structure:**
```rust
pub struct IncrementalCacheData {
    // Metadata for ALL matched files (detect new GraphQL)
    pub file_meta: HashMap<PathBuf, FileMeta>,

    // Content tracking ONLY for files with documents
    pub document_sources: HashMap<PathBuf, DocumentSourceInfo>,

    // Dependency graph
    pub dependencies: DependencyGraph,

    // Per-output tracking
    pub outputs: HashMap<PathBuf, OutputInfo>,
}

pub struct DocumentSourceInfo {
    pub content_hash: u64,
    pub document_names: Vec<Name>,  // operations/fragments extracted
}

pub struct DependencyGraph {
    // fragment → operations that use it
    pub fragment_dependents: HashMap<Name, Vec<Name>>,
    // output file → documents that contribute to it
    pub output_sources: HashMap<PathBuf, Vec<Name>>,
}
```

**Tasks:**
- [ ] Track which files actually yield documents during extraction
- [ ] Only store content hashes for files with GraphQL (reduce bloat)
- [ ] Build dependency graph (operations → fragments they use)
- [ ] Track which documents affect which outputs
- [ ] Implement partial regeneration (only affected outputs)
- [ ] Handle fragment changes propagating to dependent operations
- [ ] Test incremental correctness (no stale outputs)
- [ ] Benchmark incremental vs full regeneration

---

### Phase 4: Developer Experience
> Watch mode, better errors, IDE integration, benchmarking

#### 4.0 Benchmarking vs Node.js graphql-codegen
- [ ] Create `benchmarks/` directory with shared fixtures
- [ ] Set up hyperfine for CLI comparison
- [ ] Generate scale test fixtures (100-500+ operations)
- [ ] Create benchmark script comparing SGC vs `npx graphql-codegen`
- [ ] Document benchmark results in README
- [ ] Add CI job to track performance regressions

**Benchmark setup:**
```bash
hyperfine \
  --warmup 3 \
  --min-runs 10 \
  --export-markdown bench.md \
  'sgc -c codegen.json --no-cache' \
  'npx graphql-codegen --config codegen.ts'
```

#### 4.1 Watch Mode
> Depends on Phase 3.5 (Incremental Caching) for efficient regeneration

- [ ] Add `--watch` flag
- [ ] Use `notify` crate for filesystem watching
- [ ] Wire up to incremental cache for partial regeneration
- [ ] Debounce rapid changes
- [ ] Clear terminal and show generation summary on each run

#### 4.2 Error Messages
- [ ] Add source locations to all errors
- [ ] Pretty-print errors with code snippets (miette crate)
- [ ] Validation errors with suggestions
- [ ] Schema/document compatibility checking

#### 4.3 IDE Integration
- [ ] VSCode extension for config validation
- [ ] Language server for GraphQL documents
- [ ] Go-to-definition for generated types

---

### Phase 5: Advanced Features
> Features beyond basic codegen

#### 5.1 Fragment Colocation
```typescript
// Auto-generate fragment types next to components
export const UserFragment = gql`
  fragment UserFragment on User {
    id
    name
  }
`
export type UserFragment = {
  id: string
  name: string
}
```

#### 5.2 Client Presets
- [ ] Apollo Client preset (includes cache policies)
- [ ] urql preset
- [ ] React Query preset (with hooks generation)
- [ ] SWR preset

#### 5.3 Schema Stitching / Federation Support
- [ ] Multiple schema sources
- [ ] Remote schema introspection
- [ ] Federation directive handling

#### 5.4 Custom Plugin API
```typescript
// plugins/my-plugin.ts
import { Plugin } from '@gql-codegen/core'

export const myPlugin: Plugin = {
  name: 'my-plugin',
  generate({ schema, documents, config }) {
    return `// Custom generated code`
  }
}
```

**Tasks:**
- [ ] Design plugin interface (Rust trait + TypeScript types)
- [ ] If NAPI: plugins can be pure JS/TS
- [ ] If wrapper: plugins compile to WASM or run in Node

#### 5.5 Cache-Informed Allocation Hints

Use cache metadata from previous runs to pre-size allocators, reducing allocation overhead on subsequent runs.

**Concept:**
```rust
// Store size hints after generation
pub struct SizeHints {
    pub total_output_bytes: usize,
    pub operation_count: usize,
    pub fragment_count: usize,
    pub source_files_total_bytes: usize,
}

// On next run, use hints to pre-allocate
let mut output = String::with_capacity(hints.total_output_bytes + 1024);
let mut operations = Vec::with_capacity(hints.operation_count);
```

**Potential applications:**
- Pre-size output `String` buffers (avoid reallocation during generation)
- Pre-size `Vec` for operations/fragments
- Pre-allocate `SourceCache` capacity based on expected file count/sizes
- If using `bumpalo` arena allocator, pre-size the arena chunk

**Similar patterns in other tools:**
- V8/SpiderMonkey inline caches adapt based on runtime profiling
- rustc incremental compilation caches dependency graph sizes
- Cap'n Proto pre-computes message sizes for zero-copy serialization

**Tasks:**
- [ ] Add `SizeHints` to `CacheData`
- [ ] Collect metrics after generation (output size, counts)
- [ ] Apply hints on cache hit to pre-size allocators
- [ ] Benchmark to measure actual improvement
- [ ] Consider bump allocator (`bumpalo`) for AST nodes if hints show benefit

---

## Priority Recommendation

| Phase | Effort | Impact | Priority |
|-------|--------|--------|----------|
| Phase 0 | Low | High (unblocks everything) | **P0** |
| Phase 0.5 | Low-Medium | Very High (DX, smaller binary) | **P0** |
| Phase 1.1-1.2 | Medium | High (compatibility) | **P1** |
| Phase 2 (Option A) | Medium | High (DX) | **P1** |
| Phase 3.3 | Medium | Very High (killer feature) | **P1** |
| Phase 3.5 (Incremental) | Medium-High | Very High (watch mode prereq) | **P1** |
| Phase 1.3 | High | Medium | P2 |
| Phase 4.1 | Medium | High (depends on 3.5) | P2 |
| Phase 2 (Option B) | High | Medium | P3 |
| Phase 5.1-5.4 | Very High | Medium | P3 |
| Phase 5.5 (Cache Hints) | Low | Low-Medium (perf optimization) | P3 |

---

## Recommended First Steps

1. **Fix build errors** (Phase 0) - unblocks everything
2. **Relay-style extraction** (Phase 0.5) - big DX win, removes OXC dep, simpler code
3. **Rename to match graphql-codegen** (Phase 1.1) - easy win for familiarity
4. **Create npm wrapper with TS config** (Phase 2 Option A) - unlocks TS configs fast
5. **Implement near-operation-file** (Phase 3.3) - the feature that makes this tool worth switching to

---

## Open Questions

1. **Binary distribution strategy**: GitHub releases? npm binary packages? Homebrew?
2. **Plugin extensibility**: Do we want arbitrary plugins or a curated set?
3. **Backwards compatibility**: Support both old and new config formats during transition?
4. **Name**: Keep `gql_codegen` or rebrand for npm publishing?
