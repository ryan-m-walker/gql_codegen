import { dirname } from 'node:path'
import { parseArgs } from 'node:util'
import type { GenerateOptions } from '@sgc/core'
import { extractSchema, loadConfig, resolveConfigPaths } from './config.js'
import { help } from './help.js'
import { CLI_OPTIONS, type ParsedArgs, VERSION } from './options.js'

function toGenerateOptions(
    args: ParsedArgs,
    configJson: string,
): GenerateOptions {
    return {
        configJson,
        noCache: args['no-cache'],
        timing: args.timing,
    }
}

export async function run() {
    const args = parseArgs({
        options: CLI_OPTIONS,
        strict: false,
        allowPositionals: true,
    })

    const argValues = args.values as ParsedArgs

    if (argValues.help) {
        help()
        process.exit(0)
    }

    if (argValues.version) {
        console.log(`sgc ${VERSION}`)
        process.exit(0)
    }

    const { config, configPath } = await loadConfig(argValues.config)
    const resolvedConfig = resolveConfigPaths(config, dirname(configPath))

    console.log(resolvedConfig)
    const schemas = extractSchema(resolvedConfig)
    console.log(schemas)

    // TODO: schema resolution step (programmatic .ts/.js schemas)
    // TODO: native generate / binary fallback
    // TODO: handle --check, --stdout, --clean-cache, --watch
    // TODO: file writing, warnings, timing output
}
