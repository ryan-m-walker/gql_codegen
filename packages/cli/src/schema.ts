import { createRequire } from 'node:module'
import path from 'node:path'
import { pathToFileURL } from 'node:url'

import type { DocumentNode, GraphQLSchema } from 'graphql'
import { createJiti } from 'jiti'
import { exists } from './utils.js'

// Duck-type checks to avoid cross-realm instanceof issues.
// graphql-js's isSchema/isScalarType use instanceof internally, which
// throws when the schema was loaded from a different module context
// (e.g., jiti, vitest transforms, or multiple graphql versions).

function isSchemaLike(value: unknown): value is GraphQLSchema {
    if (typeof value !== 'object' || value === null) return false
    const v = value as Record<string, unknown>
    return (
        typeof v.getTypeMap === 'function' &&
        typeof v.getQueryType === 'function' &&
        typeof v.getDirectives === 'function'
    )
}

function isScalarTypeLike(type: unknown): boolean {
    if (typeof type !== 'object' || type === null) return false
    const t = type as Record<string, unknown>
    return (
        typeof t.serialize === 'function' && typeof t.parseValue === 'function'
    )
}

// Runtime graphql functions, loaded through jiti for realm compatibility.
// We must NOT use statically imported print/printSchema — they come from
// our module context, but the schema comes from jiti's module context.
// Using same-realm functions avoids the "Cannot use X from another module
// or realm" error that graphql-js throws via internal instanceof checks.
interface GraphQLRuntime {
    print: (ast: DocumentNode) => string
    printSchema: (schema: GraphQLSchema) => string
}

export interface SchemaResult {
    sdl: string
    scalars: Record<string, string | { input: string; output: string }>
}

export interface ResolvedSchemas {
    /** File paths/globs for Rust to handle (.graphql, etc.) */
    schemaPaths: string[]
    /** Pre-resolved SDL strings from programmatic .ts/.js schemas */
    schemaContent: string[]
    /** Scalar type mappings extracted from codegenScalarType extensions */
    scalars: Record<string, string | { input: string; output: string }>
}

/**
 * Resolve schema sources, splitting programmatic (.ts/.js) from static (.graphql).
 *
 * Programmatic schemas are imported and converted to SDL strings.
 * Static paths are passed through for Rust to handle.
 */
export async function resolveSchemas(
    schema: string | string[],
): Promise<ResolvedSchemas> {
    const paths = Array.isArray(schema) ? schema : [schema]

    const result: ResolvedSchemas = {
        schemaPaths: [],
        schemaContent: [],
        scalars: {},
    }

    for (const schemaPath of paths) {
        const loaded = await loadSchema(schemaPath)

        if (loaded) {
            result.schemaContent.push(loaded.sdl)
            Object.assign(result.scalars, loaded.scalars)
        } else {
            result.schemaPaths.push(schemaPath)
        }
    }

    return result
}

const JS_EXTENSIONS = ['.js', '.mjs', '.cjs', '.jsx']
const TS_EXTENSIONS = ['.ts', '.tsx', '.mts', '.cts']

/**
 * Try to load a schema from a .ts/.js file that exports a GraphQLSchema.
 * Returns null if the path isn't a programmatic schema (e.g. .graphql, globs).
 */
export async function loadSchema(
    schemaPath: string,
): Promise<SchemaResult | null> {
    const p = path.parse(schemaPath)

    if (!JS_EXTENSIONS.includes(p.ext) && !TS_EXTENSIONS.includes(p.ext)) {
        return null
    }

    // Rust core handles globs
    if (isGlobPath(p.base)) {
        return null
    }

    const resolved = path.resolve(p.dir, p.base)
    const pathExists = await exists(resolved)

    if (!pathExists) {
        return null
    }

    // Try native import() first — inherits tsx/ts-node loader hooks and
    // the user's registered resolvers. Falls back to jiti for standalone use
    // (no TS runtime registered).
    const [mod, graphql] = await importModule(resolved)

    return resolveSchemaFromModule(mod, schemaPath, graphql)
}

type ImportResult = [mod: unknown, graphql: GraphQLRuntime]

/**
 * Resolve `graphql` from the schema file's node_modules — not from the CLI's.
 * This ensures printSchema/isSchema use the same graphql instance as the schema.
 */
function resolveGraphqlFrom(schemaPath: string): string {
    const req = createRequire(schemaPath)
    return pathToFileURL(req.resolve('graphql')).href
}

async function importModule(resolved: string): Promise<ImportResult> {
    const fileUrl = pathToFileURL(resolved).href
    const graphqlUrl = resolveGraphqlFrom(resolved)

    try {
        // Native import() inherits registered loaders (tsx, ts-node, etc.)
        // and the user's registered resolvers.
        const [mod, graphql] = await Promise.all([
            import(fileUrl),
            import(graphqlUrl) as Promise<typeof import('graphql')>,
        ])
        return [mod, graphql]
    } catch {
        // No TS runtime available — fall back to jiti for transpilation.
        const jiti = createJiti(resolved, { jsx: true })
        const [mod, graphql] = await Promise.all([
            jiti.import(resolved),
            jiti.import('graphql') as Promise<typeof import('graphql')>,
        ])
        return [mod, graphql as GraphQLRuntime]
    }
}

/**
 * Resolve a GraphQL schema from a module's exports.
 *
 * Tries in order:
 * 1. Named `schema` export
 * 2. Default export
 * 3. The module namespace itself
 *
 * Each value is checked for: GraphQLSchema, string SDL, DocumentNode, or
 * a function returning one of those.
 */
function resolveSchemaFromModule(
    mod: unknown,
    sourcePath: string,
    graphql: GraphQLRuntime,
): SchemaResult {
    if (typeof mod !== 'object' || mod === null) {
        throw new Error(
            `Schema module '${sourcePath}' did not return a valid module object.\n` +
                'Make sure the file exports a GraphQLSchema, DocumentNode, SDL string, or a function returning one of these.',
        )
    }

    const record = mod as Record<string, unknown>
    // Try named `schema` export first, then default, then the module itself
    const candidates = [record.schema, record.default, mod]

    for (const candidate of candidates) {
        if (candidate == null) continue

        const resolved =
            typeof candidate === 'function' ? candidate() : candidate
        const result = toSchemaResult(resolved, graphql)
        if (result) return result
    }

    const exportKeys = Object.keys(record).filter(
        (k) => k !== '__esModule' && k !== 'default',
    )
    const exportsHint =
        exportKeys.length > 0
            ? ` Found exports: ${exportKeys.join(', ')}`
            : ' The module has no named exports.'

    throw new Error(
        `Schema module '${sourcePath}' does not export a valid schema.${exportsHint}\n` +
            'Expected: a default or named "schema" export of type GraphQLSchema, DocumentNode, SDL string, or a function returning one of these.',
    )
}

/**
 * Convert a value to a SchemaResult if it's a recognized schema format.
 */
function toSchemaResult(
    value: unknown,
    graphql: GraphQLRuntime,
): SchemaResult | null {
    if (isSchemaLike(value)) {
        return {
            sdl: graphql.printSchema(value),
            scalars: extractCodegenScalarTypes(value),
        }
    }

    if (typeof value === 'string') {
        return { sdl: value, scalars: {} }
    }

    // DocumentNode (from graphql-tag or similar)
    if (isDocumentNode(value)) {
        return { sdl: graphql.print(value), scalars: {} }
    }

    return null
}

/**
 * Extract codegenScalarType extensions from a GraphQLSchema.
 * Libraries like graphql-scalars attach these to provide TypeScript type hints.
 */
function extractCodegenScalarTypes(
    schema: GraphQLSchema,
): Record<string, string | { input: string; output: string }> {
    const scalars: Record<string, string | { input: string; output: string }> =
        {}
    const typeMap = schema.getTypeMap()

    for (const [name, type] of Object.entries(typeMap)) {
        if (!isScalarTypeLike(type) || name.startsWith('__')) continue

        const extensions = type.extensions as Record<string, unknown>
        const codegenType = extensions?.codegenScalarType

        if (typeof codegenType === 'string') {
            scalars[name] = codegenType
        } else if (isInputOutputScalar(codegenType)) {
            scalars[name] = codegenType
        }
    }

    return scalars
}

function isDocumentNode(value: unknown): value is DocumentNode {
    return (
        typeof value === 'object' &&
        value !== null &&
        'kind' in value &&
        (value as Record<string, unknown>).kind === 'Document'
    )
}

function isInputOutputScalar(
    value: unknown,
): value is { input: string; output: string } {
    return (
        typeof value === 'object' &&
        value !== null &&
        'input' in value &&
        typeof (value as Record<string, unknown>).input === 'string' &&
        'output' in value &&
        typeof (value as Record<string, unknown>).output === 'string'
    )
}

function isGlobPath(p: string) {
    return p.includes('*') || p.includes('?')
}

type ConfigWithSchemas = {
    schema: string | string[]
}

export function isConfigWithSchemas(
    config: unknown,
): config is ConfigWithSchemas {
    if (typeof config !== 'object' || config === null) return false
    if (!('schema' in config)) return false

    if (typeof config.schema === 'string') return true

    if (Array.isArray(config.schema)) {
        return config.schema.every((s) => typeof s === 'string')
    }

    return false
}
