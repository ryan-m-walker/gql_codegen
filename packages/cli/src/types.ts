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
    outputs: Record<string, OutputConfig>

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
    /** Generators to run for this output (defaults to all three when omitted) */
    generators?: GeneratorConfig[]

    /** Content to prepend to the output */
    prelude?: string

    /** Shared config for all generators */
    config?: GeneratorOptions
}

export type GeneratorConfig = string | Record<string, GeneratorOptions>

export interface GeneratorOptions {
    /** Custom scalar type mappings */
    scalars?: Record<string, string | { input: string; output: string }>
    /** Whether to generate readonly types */
    immutableTypes?: boolean
    /** Use string union types instead of TS enums */
    enumsAsTypes?: boolean
    /** Add '%future added value' to enum unions */
    futureProofEnums?: boolean
    /** Add '%other' fallback to union types */
    futureProofUnions?: boolean
    /** Declaration kind: 'type', 'interface', 'class', or 'abstract class' */
    declarationKind?: 'type' | 'interface' | 'class' | 'abstract class'
    /** Prefix for generated type names */
    typeNamePrefix?: string
    /** Suffix for generated type names */
    typeNameSuffix?: string

    strictScalars?: boolean

    onlyReferencedTypes?: boolean
}

export interface HooksConfig {
    /** Commands to run after generation completes (receives all written file paths) */
    afterGenerate?: string[]
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
 *   documents: './src/**\/*.graphql',
 *   outputs: {
 *     './src/generated/types.ts': {
 *       generators: ['schema-types', 'operation-types'],
 *     },
 *   },
 * });
 * ```
 */
export function defineConfig(config: CodegenConfig): CodegenConfig {
    return config
}
