/**
 * Config loading utilities
 *
 * Handles loading configuration from JSON or TypeScript/JavaScript files.
 * TS/JS configs are loaded at runtime using jiti, which handles
 * TypeScript transpilation, ESM/CJS interop, and tsconfig paths.
 */

import fs from 'node:fs/promises'
import path, { join, resolve } from 'node:path'
import { createJiti } from 'jiti'

import type { CodegenConfig } from './types.js'
import { exists } from './utils.js'

const CONFIG_FILES = [
    'codegen.ts',
    'codegen.mts',
    'codegen.cts',
    'codegen.js',
    'codegen.mjs',
    'codegen.cjs',
    'codegen.json',
    // 'codegen.yaml',
]

export interface LoadConfigResult {
    config: unknown
    configPath: string
}

/**
 * Find and load the codegen configuration file.
 * Will use the config path if one is provided, otherwise will attempt to auto-detect.
 * TODO: pretty error messages
 */
export async function loadConfig(
    configPath?: string,
    cwd: string = process.cwd(),
): Promise<LoadConfigResult> {
    const resolvedPath = configPath
        ? resolve(cwd, configPath)
        : await findConfigFile(cwd)

    if (!resolvedPath) {
        throw new Error(
            `No config file found. Create one of: ${CONFIG_FILES.join(', ')}`,
        )
    }

    const pathExists = await exists(resolvedPath)
    if (!pathExists) {
        throw new Error(`Config file not found: ${resolvedPath}`)
    }

    const config = await loadConfigFile(resolvedPath)

    return {
        config,
        configPath: resolvedPath,
    }
}

async function findConfigFile(cwd: string): Promise<string | null> {
    for (const filename of CONFIG_FILES) {
        const fullPath = join(cwd, filename)
        if (await exists(fullPath)) {
            return fullPath
        }
    }
    return null
}

async function loadConfigFile(configPath: string): Promise<unknown> {
    const ext = path.parse(configPath).ext

    if (ext === '.json') {
        return loadJsonConfig(configPath)
    }

    // jiti handles .ts, .mts, .cts, .js, .mjs, .cjs, .jsx, .tsx
    const jiti = createJiti(import.meta.url)
    return jiti.import(configPath, { default: true })
}

async function loadJsonConfig(configPath: string): Promise<unknown> {
    const content = await fs.readFile(configPath, 'utf-8')
    try {
        return JSON.parse(content)
    } catch {
        throw new Error(`Failed to parse JSON config: ${configPath}`)
    }
}

/**
 * Convert config to JSON for passing to the Rust binary.
 */
export function configToJson(config: CodegenConfig): string {
    return JSON.stringify(config, null, 2)
}

/**
 * Resolve all relative paths in the config to absolute paths.
 * This is needed when writing a temp JSON config from a TS config.
 */
export function resolveConfigPaths(
    config: CodegenConfig,
    baseDir: string,
): CodegenConfig {
    const resolvePath = (p: string) =>
        p.startsWith('/') ? p : resolve(baseDir, p)
    const resolvePaths = (paths: string | string[]): string | string[] => {
        if (Array.isArray(paths)) {
            return paths.map(resolvePath)
        }
        return resolvePath(paths)
    }

    const resolved: CodegenConfig = {
        ...config,
        schema: resolvePaths(config.schema),
        documents: resolvePaths(config.documents),
        generates: {},
        // Set baseDir so the Rust CLI knows where the original config was
        baseDir: baseDir,
    }

    // Resolve output paths
    for (const [outputPath, outputConfig] of Object.entries(config.generates)) {
        const resolvedOutputPath = resolvePath(outputPath)
        resolved.generates[resolvedOutputPath] = outputConfig
    }

    return resolved
}

// TODO: does graphql-codegn support multiple programic schemas?
export function extractSchema(config: unknown): string | string[] {
    const output: string[] = []

    // TODO: nice errors
    if (typeof config !== 'object' || config === null) {
        throw new Error('Invalid config')
    }

    if (!('schema' in config)) {
        throw new Error('Missing schema')
    }

    if (Array.isArray(config.schema)) {
        for (const schema of config.schema) {
            if (typeof schema !== 'string') {
                throw new Error('Invalid schema type')
            }

            output.push(schema)
        }

        return output
    }

    if (typeof config.schema === 'string') {
        output.push(config.schema)
        return output
    }

    throw new Error('Invalid schema type')
}
