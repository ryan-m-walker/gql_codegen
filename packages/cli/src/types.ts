/**
 * SGC Configuration Types
 *
 * These types match the Rust CodegenConfig structure.
 */

export interface CodegenConfig {
    /** Path to GraphQL schema file(s) */
    schema: string | string[]

    /** Glob patterns for document files */
    documents: string | string[]

    /** Output configurations keyed by output path */
    generates: Record<string, OutputConfig>

    /** Lifecycle hooks — shell commands run after generation */
    hooks?: HooksConfig

    /**
     * Base directory for resolving paths.
     * @internal Set automatically by the CLI - do not set manually.
     */
    baseDir?: string

    /**
     * Pre-resolved schema SDL strings from programmatic .ts/.js schemas.
     * When set, these are passed directly to the Rust core alongside any
     * remaining file paths in `schema`.
     * @internal Set automatically by the CLI - do not set manually.
     */
    schemaContent?: string[]
}

export interface OutputConfig {
    /** Plugins to run for this output */
    plugins: PluginConfig[]

    /** Content to prepend to the output */
    prelude?: string

    /** Shared config for all plugins */
    config?: PluginOptions

    /** Only generate for documents, skip schema types */
    documentsOnly?: boolean

    /** Lifecycle hooks — shell commands run after this output is written */
    hooks?: HooksConfig
}

export type PluginConfig = string | Record<string, PluginOptions>

export interface PluginOptions {
    /** Custom scalar type mappings */
    scalars?: Record<string, string | { input: string; output: string }>

    /** Add readonly modifier to generated types */
    immutableTypes?: boolean

    /** Generate enums as string union types */
    enumsAsTypes?: boolean

    /** Add future-proof unknown value to enums */
    futureProofEnums?: boolean

    /** Controls how __typename is emitted in generated types */
    typenamePolicy?: 'always' | 'as-selected' | 'skip'

    /** @deprecated Use typenamePolicy: 'skip' instead */
    skipTypename?: boolean

    // TODO:
    avoidOptionals?: boolean

    /** GraphQL tag style for documents */
    graphqlTag?: 'gql' | 'graphql' | 'none'
}

export interface HooksConfig {
    /** Commands to run after each file is written (receives single file path) */
    afterOneFileWrite?: string[]

    /** Commands to run after all files are written (receives all file paths) */
    afterAllFileWrite?: string[]
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
    return config
}
