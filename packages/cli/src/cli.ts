import pc from 'picocolors'
import { dirname } from 'node:path'
import { parseArgs } from 'node:util'
import {
    clearCache,
    type GenerateOptions,
    type GenerateResult,
    generate,
    isNativeAvailable,
    writeFiles,
} from '@sgc/core'
import { configToJson, loadConfig, resolveConfigPaths } from './config.js'
import { help } from './help.js'
import { runHooks } from './hooks.js'
import { CLI_OPTIONS, type ParsedArgs, VERSION } from './options.js'
import { resolveSchemas } from './schema.js'
import type { CodegenConfig } from './types.js'

/**
 * Validate that an unknown config value has the fields Node needs to process.
 * Only checks fields used by the Node wrapper (schema, outputs).
 * Rust core handles deep validation of generator configs and documents.
 */
function validateConfig(value: unknown): CodegenConfig {
    if (typeof value !== 'object' || value === null) {
        throw new Error(
            'Invalid config: expected an object.\n' +
                'Make sure your config file exports a valid configuration.',
        )
    }

    const config = value as Record<string, unknown>

    if (!('schema' in config)) {
        throw new Error(
            'Invalid config: missing "schema" field.\n' +
                'Add a schema path, e.g.: schema: "./schema.graphql"',
        )
    }

    if (typeof config.schema !== 'string' && !Array.isArray(config.schema)) {
        throw new Error(
            'Invalid config: "schema" must be a string or array of strings.',
        )
    }

    if (
        Array.isArray(config.schema) &&
        !config.schema.every((s): s is string => typeof s === 'string')
    ) {
        throw new Error(
            'Invalid config: "schema" array must only contain strings.',
        )
    }

    if (
        !('outputs' in config) ||
        typeof config.outputs !== 'object' ||
        config.outputs === null
    ) {
        throw new Error(
            'Invalid config: missing "outputs" field.\n' +
                'Add output targets, e.g.:\n' +
                '  outputs: { "./src/types.ts": { generators: ["schema-types"] } }',
        )
    }

    // Structural validation done — Rust core validates the rest
    return value as CodegenConfig
}

function parseMaxDiagnostics(value: string | undefined): number | undefined {
    if (value == null) return undefined
    const parsed = Number.parseInt(value, 10)
    if (Number.isNaN(parsed) || parsed < 0) {
        throw new Error(
            `Invalid --max-diagnostics value: '${value}'.\n` +
                'Expected a non-negative integer.',
        )
    }
    return parsed
}

function toGenerateOptions(
    args: ParsedArgs,
    configJson: string,
): GenerateOptions {
    return {
        configJson,
        noCache: args['no-cache'],
        timing: args.timing,
        maxDiagnostics: parseMaxDiagnostics(args['max-diagnostics']),
    }
}

/**
 * Format an unknown error for stderr output.
 * NAPI errors from Rust are already formatted through our diagnostic pipeline,
 * so this mostly passes messages through as-is.
 */
function formatError(error: unknown): string {
    const message = error instanceof Error ? error.message : String(error)
    return message.endsWith('\n') ? message : `${message}\n`
}

function plural(count: number, word: string): string {
    return `${count} ${word}${count === 1 ? '' : 's'}`
}

/**
 * Handle the result of code generation.
 * Responsible for: warnings, --stdout, --check, file writing, and summary.
 */
function handleResult(
    result: GenerateResult,
    args: ParsedArgs,
    config: CodegenConfig,
): void {
    // Warnings always go to stderr, even with --quiet
    for (const warning of result.warnings) {
        process.stderr.write(warning)
    }

    if (result.fresh) {
        if (!args.quiet) {
            console.log('Nothing changed')
        }
        return
    }

    if (args.stdout) {
        for (const file of result.files) {
            process.stdout.write(`// ${file.path}\n`)
            process.stdout.write(file.content)
            process.stdout.write('\n')
        }
        return
    }

    if (args.check) {
        if (!args.quiet) {
            console.log(`Would generate ${plural(result.files.length, 'file')}`)
            for (const file of result.files) {
                console.log(`  ${file.path}`)
            }
        }
        // Non-zero exit signals to CI that files are out of date
        if (result.files.length > 0) {
            process.exitCode = 1
        }
        return
    }

    // Write files via Rust (parallel I/O + skip optimization)
    const writeResult = writeFiles(result.files)

    for (const { path, message } of writeResult.errors) {
        process.stderr.write(`Failed to write ${path}: ${message}\n`)
    }

    if (!args.quiet) {
        for (const path of writeResult.written) {
            console.log(`  ${path}`)
        }

        const total = writeResult.written.length + writeResult.skipped.length
        const skipped =
            writeResult.skipped.length > 0
                ? `, ${plural(writeResult.skipped.length, 'file')} unchanged`
                : ''
        console.log(`Generated ${plural(total, 'file')}${skipped}`)
    }

    // Run lifecycle hooks on written files
    if (writeResult.written.length > 0 && config.hooks?.afterGenerate?.length) {
        runHooks(config.hooks.afterGenerate, writeResult.written)
    }

    if (writeResult.errors.length > 0) {
        process.exitCode = 1
    }
}

export async function run(): Promise<void> {
    // parseArgs returns loosely-typed values — this narrowing is safe because
    // parseArgs only populates values for the options we declared in CLI_OPTIONS
    const { values } = parseArgs({
        options: CLI_OPTIONS,
        strict: false,
        allowPositionals: true,
    })
    const args = values as ParsedArgs

    if (args.help) {
        help()
        process.exit(0)
    }

    if (args.version) {
        console.log(`sgc ${VERSION}`)
        process.exit(0)
    }

    try {
        const { config, configPath } = await loadConfig(args.config)
        const validConfig = validateConfig(config)
        const resolvedConfig = resolveConfigPaths(
            validConfig,
            dirname(configPath),
        )

        // Resolve programmatic schemas (.ts/.js) to SDL, keep static paths for Rust
        const { schemaPaths, schemaContent, scalars } = await resolveSchemas(
            resolvedConfig.schema,
        )
        resolvedConfig.schema = schemaPaths

        if (schemaContent.length > 0) {
            resolvedConfig.schemaContent = schemaContent
        }

        // Merge extracted scalars into output configs (user-defined take precedence)
        if (Object.keys(scalars).length > 0) {
            for (const outputConfig of Object.values(
                resolvedConfig.outputs,
            )) {
                const existing = outputConfig.config?.scalars ?? {}
                outputConfig.config ??= {}
                outputConfig.config.scalars = { ...scalars, ...existing }
            }
        }

        const configJson = configToJson(resolvedConfig)

        // Handle --clean-cache before generating
        if (args['clean-cache']) {
            if (!isNativeAvailable()) {
                throw new Error(
                    'Native module (@sgc/core) not available — cannot clear cache.\n' +
                        'Ensure @sgc/core is installed correctly for your platform.',
                )
            }
            const baseDir = resolvedConfig.baseDir ?? dirname(configPath)
            const cleared = clearCache(baseDir)
            if (!args.quiet) {
                console.log(cleared ? 'Cache cleared' : 'Cache already clean')
            }
            return
        }

        // Require native module for generation
        if (!isNativeAvailable()) {
            throw new Error(
                'Native module (@sgc/core) not available.\n' +
                    'Ensure @sgc/core is installed correctly for your platform.\n' +
                    'Supported: darwin-arm64, darwin-x64, linux-x64, linux-arm64, win32-x64',
            )
        }

        const options = toGenerateOptions(args, configJson)
        const result = generate(options)
        handleResult(result, args, resolvedConfig)
    } catch (error) {
        const formatted = formatError(error)
        process.stderr.write(pc.red('error'))
        process.stderr.write(': ' + formatted)
        process.exitCode = 1
    }
}
