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
 *   generates: {
 *     './src/generated/types.ts': {
 *       plugins: ['typescript', 'typescript-operations'],
 *     },
 *   },
 * });
 * ```
 */

// Types
export type {
  CodegenConfig,
  OutputConfig,
  PluginConfig,
  PluginOptions,
  FormattingOptions,
} from './types.js';

// Config helper
export { defineConfig } from './types.js';

// Config loading (for programmatic use)
export { loadConfig, configToJson } from './config.js';

// Binary execution (for programmatic use)
export { runBinary, findBinary } from './binary.js';
export type { CliFlags, RunOptions, RunResult } from './binary.js';
