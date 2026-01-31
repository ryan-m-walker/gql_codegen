# SGC Incremental Caching Design

## Goals

1. **Correctness**: Never serve stale results
2. **Speed**: Skip as much work as possible on incremental builds
3. **Simplicity**: Easy to understand and debug

## Architecture Overview

```
.sgc/
├── inputs/
│   ├── schema.hash              # Combined schema files hash
│   ├── config.hash              # Config options hash
│   └── documents/
│       ├── {hash}.meta          # Document metadata + extracted ops
│       └── ...
├── outputs/
│   ├── {output_path_hash}.hash  # Content hash of each output
│   └── ...
└── artifacts/
    ├── {op_hash}.ts             # Cached generated code per operation
    └── ...
```

## Multi-Level Caching Strategy

### Level 1: File Metadata Check (Fast)
```
mtime + size → if unchanged, skip to cached result
```
- Cheapest check (~1ms for hundreds of files)
- Falls through to Level 2 if metadata changed

### Level 2: Content Hash Check
```
file content → xxhash → compare to stored hash
```
- Only runs if metadata changed
- Determines if actual content changed

### Level 3: Parsed/Generated Artifacts
```
content hash → lookup cached AST/generated code
```
- Most expensive to compute, so we cache aggressively
- Content-addressable: same content = same cache key

## Cache Invalidation Rules

| Change Type | Action |
|-------------|--------|
| Schema file changed | Regenerate ALL outputs |
| Config options changed | Regenerate ALL outputs |
| Document file changed | Regenerate outputs containing ops from that doc |
| Output file missing | Regenerate that output |
| Output content matches | Skip write (preserve mtime) |

## Dependency Tracking

### Document → Operations Map
```rust
struct DocumentCache {
    path: PathBuf,
    content_hash: u64,
    operations: Vec<OperationRef>,  // Names of ops defined here
    fragments: Vec<FragmentRef>,    // Names of fragments defined here
}
```

### Operation → Output Map
```rust
struct OutputDeps {
    output_path: PathBuf,
    operations: HashSet<String>,    // Ops that contribute to this output
    fragments: HashSet<String>,     // Fragments used by those ops
}
```

### Invalidation Flow
```
document changed
  → which ops/fragments changed?
    → which outputs use those ops/fragments?
      → regenerate only those outputs
```

## Implementation Phases

### Phase 1: Basic Correctness (Current Goal)
- [x] Hash config options (not raw content)
- [ ] Hash schema files
- [ ] Hash document files
- [ ] Skip write if output unchanged

### Phase 2: Per-Output Tracking
- [ ] Track which documents contribute to which outputs
- [ ] Only regenerate affected outputs when doc changes
- [ ] Separate hash files per output

### Phase 3: Operation-Level Memoization
- [ ] Cache parsed operations by content hash
- [ ] Cache generated TypeScript per operation
- [ ] Compose outputs from cached fragments

### Phase 4: Advanced Optimizations
- [ ] File metadata pre-check (mtime + size)
- [ ] Parallel hash computation
- [ ] Watch mode with incremental updates
- [ ] Remote cache support (optional)

## Key Design Decisions

### Why separate hash files vs single JSON?
- **Atomic writes**: Each output's hash is independent
- **Parallel safe**: Multiple writers don't conflict
- **Simpler updates**: No JSON parse/modify/serialize

### Why content-addressable for artifacts?
- **Deduplication**: Same operation in different files = one cache entry
- **Portable**: Cache can be shared across branches/machines
- **Simple invalidation**: Hash mismatch = regenerate

### Why mtime check before content hash?
- **Performance**: mtime check is ~1000x faster than hashing
- **Common case**: Most files don't change between builds
- **Correctness**: Fall back to hash if mtime unreliable

## Cache Key Composition

```rust
// Schema cache key
schema_key = hash(schema_file_contents...)

// Config cache key (options only, not paths)
config_key = hash(
    plugins,
    scalars,
    immutable_types,
    enums_as_types,
    // ... other options
)

// Document cache key
doc_key = hash(file_content)

// Operation cache key
op_key = hash(
    operation_text,
    schema_key,      // Schema affects type resolution
    config_key,      // Config affects output format
)

// Output cache key
output_key = hash(
    sorted(operation_keys...),
    output_config,
)
```

## Garbage Collection

### Strategy: Automatic Mark-and-Sweep + Size Cap

```
.sgc/
├── gc.lock              # Prevents concurrent GC
├── gc.meta              # Last GC timestamp + stats
└── artifacts/
    └── {hash}.artifact  # Access time tracked via filesystem
```

### GC Triggers

| Trigger | Condition |
|---------|-----------|
| **Time-based** | Last GC > 7 days ago |
| **Size-based** | Cache dir > 500MB |
| **Manual** | `sgc --gc` flag |

### GC Algorithm

```
1. LOCK: Acquire gc.lock (skip if already locked)
2. MARK: During normal build, "touch" all used artifacts
3. SWEEP: Delete artifacts with atime > 30 days old
4. CAP: If still > 500MB, delete oldest until under limit
5. UPDATE: Write new timestamp to gc.meta
6. UNLOCK: Release gc.lock
```

### Configuration (future)

```json
{
  "cache": {
    "maxAge": "30d",      // Max artifact age
    "maxSize": "500MB",   // Max cache size
    "gcInterval": "7d"    // Auto-GC frequency
  }
}
```

### Edge Cases

- **Concurrent builds**: Lock file prevents race conditions
- **CI environments**: Consider `SGC_CACHE_DIR` env var for shared cache
- **Branch switching**: Artifacts from other branches kept until GC sweep

---

## Open Questions

1. **Fragment dependencies**: If FragmentA uses FragmentB, and FragmentB changes,
   do we detect that FragmentA's output needs regeneration?
   - Option A: Track fragment→fragment deps (complex)
   - Option B: Invalidate all outputs using any changed fragment (simpler)

2. **Schema granularity**: If only one type in schema changes, can we skip
   regenerating operations that don't use that type?
   - Probably not worth the complexity for v1

3. **Watch mode**: How do we handle rapid successive changes?
   - Debounce? Coalesce? Queue?

## References

- [Relay Compiler Architecture](https://relay.dev/docs/principles-and-architecture/compiler-architecture/)
- [Turborepo Caching](https://turborepo.dev/docs/crafting-your-repository/caching)
- [esbuild Incremental API](https://esbuild.github.io/api/#incremental)
- [Nx Caching](https://nx.dev/docs/concepts/how-caching-works)
