/**
 * TypeScript config types for gql-codegen
 *
 * Example codegen.ts:
 * ```ts
 * import { defineConfig } from '@gql-codegen/cli'
 *
 * export default defineConfig({
 *   schema: './schema.graphql',
 *   documents: ['./src/**\/*.tsx', './src/**\/*.graphql'],
 *   generates: {
 *     './src/generated/types.ts': {
 *       plugins: ['typescript', 'typescript-operations'],
 *     },
 *     // Or use dynamic paths for near-operation-file pattern
 *     './${filepath}/${filename}.generated.ts': {
 *       plugins: ['typescript-operations'],
 *       documentsOnly: true, // Only include operations from matched documents
 *     },
 *   },
 * })
 * ```
 */

export interface CodegenConfig {
  /**
   * Path to GraphQL schema file(s) or URL for introspection
   */
  schema: string | string[];

  /**
   * Glob patterns for documents (operations & fragments)
   * Supports .graphql files and extracting from .ts/.tsx/.js/.jsx
   */
  documents: string | string[];

  /**
   * Output file configurations
   * Keys can include path variables: ${filepath}, ${filename}, ${operationName}, ${operationType}
   */
  generates: Record<string, OutputConfig>;

  /**
   * Hooks to run at various lifecycle points
   */
  hooks?: {
    beforeGenerate?: () => void | Promise<void>;
    afterGenerate?: () => void | Promise<void>;
    afterFileWrite?: (path: string) => void | Promise<void>;
  };
}

export interface OutputConfig {
  /**
   * Plugins to run for this output
   */
  plugins: PluginConfig[];

  /**
   * Content to prepend to the generated file
   */
  prelude?: string;

  /**
   * Shared config that applies to all plugins
   */
  config?: PluginOptions;

  /**
   * Only include documents (operations/fragments), skip schema types
   */
  documentsOnly?: boolean;
}

export type PluginConfig =
  | PluginName
  | { [name: string]: PluginOptions };

export type PluginName =
  | 'typescript'           // Schema types
  | 'typescript-operations' // Operation result types
  | 'typescript-documents'  // Document constants with gql tag
  | (string & {});          // Allow custom plugins

export interface PluginOptions {
  /**
   * Custom scalar mappings: GraphQL scalar -> TypeScript type
   * @example { DateTime: 'Date', JSON: 'unknown' }
   */
  scalars?: Record<string, string>;

  /**
   * Add `readonly` modifier to all fields
   * @default false
   */
  immutableTypes?: boolean;

  /**
   * Generate enums as string unions instead of TS enums
   * @default true
   */
  enumsAsTypes?: boolean;

  /**
   * Add `| "%future added value"` to enum unions
   * @default false
   */
  futureProofEnums?: boolean;

  /**
   * Skip __typename in generated types
   * @default false
   */
  skipTypename?: boolean;

  // TODO: comment
  avoidOptionals?: boolean;

  /**
   * GraphQL tag for document generation: 'gql' | 'graphql' | 'none'
   * @default 'gql'
   */
  graphqlTag?: 'gql' | 'graphql' | 'none';

  /**
   * Formatting options
   */
  formatting?: {
    indentWidth?: number;
    useTabs?: boolean;
    singleQuote?: boolean;
    semicolons?: boolean;
  };
}

/**
 * Helper to define config with type checking
 */
export function defineConfig(config: CodegenConfig): CodegenConfig {
  return config;
}

/**
 * Result returned from generate()
 */
export interface GenerateResult {
  files: GeneratedFile[];
  duration: number;
}

export interface GeneratedFile {
  path: string;
  content: string;
}
