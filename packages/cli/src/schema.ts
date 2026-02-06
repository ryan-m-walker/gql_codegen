import fs from 'node:fs/promises'
import path from 'node:path'
import { build } from 'esbuild'

import { GraphQLSchema, printSchema } from 'graphql'

// TODO: pretty errors
export function executeProgrammaticSchemas(config: unknown) {
    if (typeof config !== 'object' || config === null) {
        throw new Error('Invalid config')
    }

    if (!('schema' in config)) {
        throw new Error('Missing schema')
    }

    if (typeof config.schema === 'string') {
        // TODO:
        return
    }

    if (Array.isArray(config.schema)) {
        // TODO:
        return
    }

    throw new Error('Invalid schema type')
}

const JS_EXTENSIONS = ['.js', '.mjs', '.cjs', '.jsx']
const TS_EXTENSIONS = ['.ts', '.tsx', '.mts', '.cts']

export async function loadSchema(schema: string) {
    const p = path.parse(schema)

    if (!JS_EXTENSIONS.includes(p.ext) && !TS_EXTENSIONS.includes(p.ext)) {
        return
    }

    // rust core handles globs
    if (isGlobPath(p.base)) {
        return
    }

    const resolved = path.resolve(p.dir, p.base)
    const fileExists = await exists(resolved)

    if (!fileExists) {
        return
    }

    if (JS_EXTENSIONS.includes(p.ext)) {
        const mod = await import(schema)

        if ('schema' in mod && mod.schema instanceof GraphQLSchema) {
            return printSchema(mod.schema)
        }

        if ('default' in mod && mod.default instanceof GraphQLSchema) {
            return printSchema(mod.default)
        }
    }

    if (TS_EXTENSIONS.includes(p.ext)) {
        // TODO: Claude
    }
}

function isGlobPath(p: string) {
    return p.includes('*') || p.includes('?')
}

async function exists(p: string) {
    try {
        await fs.access(p)
        return true
    } catch {
        return false
    }
}
