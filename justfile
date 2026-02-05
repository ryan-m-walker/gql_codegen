# SGC Build Commands
# Install just: cargo install just (or brew install just)

# Default: show available commands
default:
    @just --list

# Build everything
all: cli napi wasm

# Build CLI (release)
cli:
    cargo build --release -p gql_codegen_cli

# Build CLI (debug, faster)
cli-debug:
    cargo build -p gql_codegen_cli

# Build NAPI module for Node.js
napi:
    cd packages/core && pnpm build:native

# Build NAPI module (debug, faster)
napi-debug:
    cd packages/core && pnpm build:native:debug

# Build WASM for website
wasm:
    wasm-pack build crates/gql_codegen_wasm --target web --out-dir ../../website/src/lib/wasm

# Build WASM (debug, faster but larger)
wasm-debug:
    wasm-pack build crates/gql_codegen_wasm --target web --out-dir ../../website/src/lib/wasm --dev

# Start website dev server
dev:
    cd website && pnpm dev

# Rebuild WASM and start website dev server
dev-wasm: wasm
    cd website && pnpm dev

# Build website for production
build-website: wasm
    cd website && pnpm build

# Run all tests
test:
    cargo test --workspace

# Run tests with output
test-verbose:
    cargo test --workspace -- --nocapture

# Check code compiles (fast)
check:
    cargo check --workspace

# Format code
fmt:
    cargo fmt --all

# Lint code
lint:
    cargo clippy --workspace -- -D warnings

# Clean all build artifacts
clean:
    cargo clean
    rm -rf website/src/lib/wasm
    rm -rf crates/gql_codegen_napi/*.node
