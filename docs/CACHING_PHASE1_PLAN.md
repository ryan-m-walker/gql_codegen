# Phase 1 Caching Implementation Plan

## Goal
Achieve **correct cache invalidation** and **avoid unnecessary writes**.

---

## Current State

### What works:
- Schema files are hashed → `inputs_hash`
- Config options are hashed → `config_hash`
- Cache stored in `.sgc/cache.json`

### What's broken:
- Document files are NOT hashed (changing a `.graphql` doc doesn't invalidate cache)
- All outputs are always written (even if content unchanged)
- Cache location was wrong when using Node CLI (fixed)

---

## Tasks

### Task 1: Add Document File Hashing

**Files to modify:**
- `crates/gql_codegen_core/src/cache/utils.rs`

**Changes:**
1. Add `expand_globs()` function to resolve document patterns to file paths
2. Update `compute_hashes()` to include document files in `inputs_hash`
3. Sort file paths before hashing for deterministic ordering

**Dependencies:**
- Uses `globset` and `walkdir` (already in Cargo.toml)

**Test cases:**
- [ ] Changing a document file invalidates cache
- [ ] Adding a new document file invalidates cache
- [ ] Deleting a document file invalidates cache
- [ ] Renaming a document file invalidates cache

---

### Task 2: Skip Unchanged Output Writes

**Files to modify:**
- `crates/gql_codegen_core/src/cache/mod.rs` (add new trait method)
- `crates/gql_codegen_core/src/cache/fs.rs` (implement for FsCache)
- `crates/gql_codegen_core/src/cache/noop.rs` (implement for NoCache)
- `crates/gql_codegen_core/src/cache/memory.rs` (implement for MemoryCache)
- `crates/gql_codegen_cli/src/main.rs` (use new method before writing)

**New trait method:**
```rust
trait Cache {
    // ... existing methods ...

    /// Check if output content has changed. Returns true if file should be written.
    fn should_write_output(&self, path: &Path, content: &[u8]) -> bool;

    /// Record the hash of a written output file.
    fn record_output(&mut self, path: &Path, content: &[u8]);
}
```

**Cache storage change:**
```
.sgc/
├── cache.json          # inputs_hash + config_hash (existing)
└── outputs/
    └── {path_hash}.hash  # u64 content hash per output file
```

**Alternative (simpler):** Just read existing output file and compare content directly:
```rust
fn should_write(path: &Path, new_content: &[u8]) -> bool {
    match fs::read(path) {
        Ok(existing) => existing != new_content,
        Err(_) => true, // File doesn't exist, must write
    }
}
```
This avoids maintaining separate hash files but requires reading the file.

**Decision needed:** Hash file vs direct comparison?
- Hash file: Faster for large files, persists across `--clean`
- Direct comparison: Simpler, no extra files, always correct

**Test cases:**
- [ ] Output file with same content is not rewritten (mtime preserved)
- [ ] Output file with different content is rewritten
- [ ] Missing output file is written
- [ ] New output file is written

---

### Task 3: Update Cache File Structure (Optional)

**Current:**
```
.sgc/
└── cache.json    # Everything in one JSON file
```

**Proposed:**
```
.sgc/
├── inputs.hash     # Just the u64 (or hex string)
├── config.hash     # Just the u64
└── outputs/
    ├── types_ts.hash
    └── documents_ts.hash
```

**Benefits:**
- Atomic writes per file
- No JSON parsing overhead
- Can update individual hashes independently

**Tradeoffs:**
- More files to manage
- Slightly more complex code

**Decision needed:** Is this worth doing now or defer to Phase 2?

---

### Task 4: Add Basic GC Infrastructure

**Files to modify:**
- `crates/gql_codegen_core/src/cache/fs.rs`
- `crates/gql_codegen_cli/src/main.rs` (handle `--gc` flag)

**Changes:**
1. Track last GC time in `.sgc/gc.meta`
2. On build, check if GC needed (time-based trigger)
3. GC implementation: delete output hashes for files that no longer exist

**For now, keep it simple:**
- Only clean up orphaned output hashes
- Defer artifact caching to Phase 3

**Test cases:**
- [ ] `--gc` flag cleans up orphaned hashes
- [ ] Auto-GC triggers after configured interval
- [ ] GC doesn't delete valid hashes

---

## Revised Architecture (Zero Double-Reads)

**Key insight:** Hash from `SourceCache` after loading, not separately.

```
┌─────────────────────────────────────────────────────────┐
│ PHASE 1: Metadata Check (FAST - no file reads)          │
├─────────────────────────────────────────────────────────┤
│ 1. Expand globs → list of file paths                    │
│ 2. stat() each file → mtime + size                      │
│ 3. Compare to cached metadata                           │
│ 4. If ALL match AND cached hashes exist → CACHE HIT     │
│    └── Exit early, no file reads at all!                │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼ (metadata mismatch)
┌─────────────────────────────────────────────────────────┐
│ PHASE 2: Load & Hash (only on cache miss)               │
├─────────────────────────────────────────────────────────┤
│ 1. load_sources() → reads files into SourceCache        │
│ 2. Hash from SourceCache.iter() (already in memory!)    │
│ 3. Compare content hashes to cached                     │
│ 4. If hashes match → CACHE HIT (rare edge case)         │
│ 5. If hashes differ → continue to generation            │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│ PHASE 3: Generate & Write                               │
├─────────────────────────────────────────────────────────┤
│ 1. Parse documents from SourceCache                     │
│ 2. Generate outputs                                     │
│ 3. Compare output to existing file (skip if unchanged)  │
│ 4. Write changed outputs                                │
│ 5. Update cache with new metadata + hashes              │
└─────────────────────────────────────────────────────────┘
```

**I/O Summary:**
- Cache hit (common): stat() only, ZERO file reads
- Cache miss: ONE read per file (into SourceCache, reused for hashing + parsing)

---

## Implementation Order

```
1. Task 1: Restructure cache check flow
   └── Split into metadata-check vs content-hash phases
   └── Add hash_from_source_cache() function
   └── Update CLI to use new flow

2. Task 2: Document hashing (correctness)
   └── Include documents in cache, not just schema
   └── Tests pass, cache invalidates on doc changes

3. Task 3: Skip unchanged writes (avoids triggering watchers)
   └── Compare output to existing file before write
   └── Tests pass, unchanged files preserve mtime

4. Task 4: Basic GC (prevent unbounded growth)
   └── Lazy cleanup during check
```

---

## Questions to Resolve Before Implementation

1. **Output comparison strategy:**
   - A) Store content hash in `.sgc/outputs/{name}.hash`
   - B) Read existing file and compare bytes directly
   - C) Both (hash for quick check, bytes as fallback)

2. **Cache file structure:**
   - A) Keep single `cache.json` (simpler)
   - B) Separate files per hash type (more atomic)

3. **GC scope for Phase 1:**
   - A) Just `--gc` manual cleanup
   - B) Auto-trigger on time interval
   - C) Defer entirely to later phase

4. **Error handling:**
   - What if a document file in the glob doesn't exist?
   - What if we can't read an existing output file?
   - Should cache errors be fatal or warnings?

---

## Performance Considerations

**CRITICAL: Caching must not slow down the happy path!**

### Overhead Budget
```
Cache check overhead should be < 50ms for:
- 1 schema file
- 100 document files
- 10 output files
```

### Lazy GC (Free Cleanup)
When checking cache, if a hash is stale → clean it up immediately.
No separate GC pass needed for most cases.

```rust
// Pseudo-code
fn check_output_hash(path: &Path, new_hash: u64) -> bool {
    let old_hash = read_hash_file(path);
    if old_hash != new_hash {
        // Stale hash - will be overwritten anyway
        // This IS our GC - no separate pass needed
    }
    old_hash == Some(new_hash)
}
```

### Background GC for Heavy Cleanup
For `--gc` or periodic deep clean:
- Spawn on separate thread
- Don't block main generation
- Use `std::thread::spawn` or `rayon::spawn`

```rust
// Non-blocking GC
if should_run_gc() {
    std::thread::spawn(|| {
        run_gc();  // Doesn't block generation
    });
}
```

### Performance Safeguards

| Operation | Target | Mitigation if slow |
|-----------|--------|-------------------|
| Glob expansion | < 10ms | Cache glob results |
| File hashing | < 20ms | Parallel with rayon |
| Hash comparison | < 1ms | Simple file read |
| Output comparison | < 5ms | Hash first, bytes if needed |

### Two-Level Input Check (esbuild pattern)

```rust
struct FileMeta {
    mtime: SystemTime,
    size: u64,
    content_hash: Option<u64>,  // Only computed if metadata changed
}

fn file_changed(path: &Path, cached: &FileMeta) -> bool {
    let meta = fs::metadata(path);

    // Level 1: Metadata check (FAST - no file read)
    if meta.mtime == cached.mtime && meta.size == cached.size {
        return false;  // Definitely unchanged
    }

    // Level 2: Content hash (SLOW - must read file)
    // Only runs if metadata changed (rare case)
    let new_hash = hash_file(path);
    new_hash != cached.content_hash
}
```

**Why this matters:**
- Metadata check: ~0.01ms per file (just stat syscall)
- Content hash: ~1-10ms per file (read + hash)
- For 100 files: 1ms vs 100-1000ms

### What NOT to do
- Don't read all document contents just to hash (use mtime+size first)
- Don't do synchronous GC during build
- Don't hash outputs before checking if inputs changed

### Fast Path (Cache Hit)
```
1. Read inputs.hash file (1 file read)
2. Read config.hash file (1 file read)
3. Compare to computed hashes
4. If match → EXIT EARLY, no generation
```
Total: ~5ms for cache hit

### Slow Path (Cache Miss)
```
1. Hash all input files (parallel)
2. Generate all outputs
3. For each output: compare to existing, skip write if same
4. Update cache files
```
Caching overhead: ~20-50ms on top of generation time

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Glob expansion is slow for large dirs | Use same ignore rules as document loading, parallel walk |
| Hash collisions | Use 64-bit hash (collision probability ~1 in 10^19) |
| Race conditions on cache files | Single-threaded cache access (parallel only for file hashing) |
| Breaking existing caches | Bump cache version, invalidate old format |
| **Caching slower than regenerating** | **Benchmark! Add `--no-cache` timing comparison** |

---

## Success Criteria

- [ ] `cargo test` passes
- [ ] Changing a document file triggers regeneration
- [ ] Unchanged outputs are not rewritten (check mtime)
- [ ] Cache works correctly from both Rust CLI and Node CLI
- [ ] `--gc` cleans up orphaned data
