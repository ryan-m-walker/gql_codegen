/**
 * @sgc/cli - Speedy GraphQL Codegen
 *
 * A fast, Rust-powered GraphQL code generator with TypeScript config support.
 *
 * @example
 * ```ts
 * // codegen.config.ts
 * import { defineConfig } from '@sgc/cli';
 *
 * export default defineConfig({
 *   schema: './schema.graphql',
 *   documents: './src/*.graphql',
 *   outputs: {
 *     './src/generated/types.ts': {
 *       generators: ['schema-types', 'operation-types'],
 *     },
 *   },
 * });
 * ```
 */

// Types
export type {
    CodegenConfig,
    OutputConfig,
    GeneratorConfig,
    GeneratorOptions,
} from './types.js'

// Config helper
export { defineConfig } from './types.js'

// Config loading (for programmatic use)
export { loadConfig, configToJson } from './config.js'

// Binary execution (for programmatic use)
export { runBinary, findBinary } from './binary.js'
export type { CliFlags, RunOptions, RunResult } from './binary.js'
