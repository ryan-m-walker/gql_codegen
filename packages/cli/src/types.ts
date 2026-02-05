/**
 * SGC Configuration Types
 *
 * These types match the Rust CodegenConfig structure.
 */

export interface CodegenConfig {
  /** Path to GraphQL schema file(s) */
  schema: string | string[];

  /** Glob patterns for document files */
  documents: string | string[];

  /** Output configurations keyed by output path */
  generates: Record<string, OutputConfig>;

  /**
   * Base directory for resolving paths.
   * @internal Set automatically by the CLI - do not set manually.
   */
  baseDir?: string;
}

export interface OutputConfig {
  /** Plugins to run for this output */
  plugins: PluginConfig[];

  /** Content to prepend to the output */
  prelude?: string;

  /** Shared config for all plugins */
  config?: PluginOptions;

  /** Only generate for documents, skip schema types */
  documentsOnly?: boolean;
}

export type PluginConfig = string | Record<string, PluginOptions>;

export interface PluginOptions {
  /** Custom scalar type mappings */
  scalars?: Record<string, string>;

  /** Add readonly modifier to generated types */
  immutableTypes?: boolean;

  /** Generate enums as string union types */
  enumsAsTypes?: boolean;

  /** Add future-proof unknown value to enums */
  futureProofEnums?: boolean;

  /** Skip __typename field in generated types */
  skipTypename?: boolean;

  // TODO: 
  avoidOptionals?: boolean;

  /** GraphQL tag style for documents */
  graphqlTag?: 'gql' | 'graphql' | 'none';

  /** Formatting options */
  formatting?: FormattingOptions;
}

export interface FormattingOptions {
  /** Number of spaces per indent level (default: 2) */
  indentWidth?: number;

  /** Use tabs instead of spaces */
  useTabs?: boolean;

  /** Use single quotes (default: true) */
  singleQuote?: boolean;

  /** Add semicolons (default: true) */
  semicolons?: boolean;
}

/**
 * Define your codegen configuration with full type safety.
 *
 * @example
 * ```ts
 * // codegen.config.ts
 * import { defineConfig } from '@sgc/cli';
 *
 * export default defineConfig({
 *   schema: './schema.graphql',
 *   documents: './src/** /*.graphql',
 *   generates: {
 *     './src/generated/types.ts': {
 *       plugins: ['typescript', 'typescript-operations'],
 *     },
 *   },
 * });
 * ```
 */
export function defineConfig(config: CodegenConfig): CodegenConfig {
  return config;
}
