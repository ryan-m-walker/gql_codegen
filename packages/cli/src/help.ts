import pc from 'picocolors'

import { CLI_OPTIONS, VERSION } from './options.js'

const CONFIG_FORMATS = [
    ['  codegen.ts', 'TypeScript config with full type safety'],
    ['  codegen.js', 'JavaScript config'],
    ['  codegen.mjs', 'ES Module config'],
    ['  codegen.json', 'JSON config'],
    ['  codegen.yaml', 'YAML config (coming soon)'],
]

const EXAMPLES = [
    ['  sgc', 'Run with auto-detected config'],
    ['  sgc -c custom.ts', 'Run with specific config'],
    ['  sgc --watch', 'Watch mode'],
    ['  sgc --check', 'Validate without writing'],
]

type Output = {
    config: string
    description: string
}

export function help() {
    let longestConfig = 0
    const output: Output[] = []

    for (const [name, option] of Object.entries(CLI_OPTIONS)) {
        let config = '  '

        if ('short' in option) {
            config += `-${option.short}, `
        } else {
            config += '    '
        }

        config += '--'
        config += name

        if ('argument' in option) {
            config += ` <${option.argument}>`
        }

        if (config.length > longestConfig) {
            longestConfig = config.length
        }

        output.push({
            config,
            description: option.description,
        })
    }

    console.log(pc.bold('SGC - Speedy GraphQL Codegen'))
    console.log(pc.dim(`Version: ${VERSION}`))
    console.log(
        'SGC is a super fast CLI tool for generating code from GraphQL schemas and documents.',
    )
    console.log()

    console.log('Usage: sgc [options]')
    console.log()

    console.log(pc.bold('Options:'))
    for (const { config, description } of output) {
        const rightPad = longestConfig - config.length
        const bold = pc.bold(config)
        console.log(`${bold}${' '.repeat(rightPad)}  ${description}`)
    }

    console.log()
    console.log(pc.bold('Config file formats (in order of precedence):'))
    for (const [name, description] of CONFIG_FORMATS) {
        const rightPad = longestConfig - name.length
        const bold = pc.bold(name)
        console.log(`${bold}${' '.repeat(rightPad)}  ${description}`)
    }

    console.log()
    console.log(pc.bold('Examples:'))
    for (const [command, description] of EXAMPLES) {
        const rightPad = longestConfig - command.length
        const bold = pc.bold(command)
        console.log(`${bold}${' '.repeat(rightPad)}  ${description}`)
    }
}
