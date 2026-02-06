import type { ParseArgsOptionDescriptor } from 'node:util'

export const VERSION = '0.1.0'

export interface CliOption extends ParseArgsOptionDescriptor {
    description: string
    argument?: string
}

export const CLI_OPTIONS = {
    config: {
        type: 'string',
        short: 'c',
        description: 'Path to config file (default: auto-detect)',
        argument: 'path',
    },
    watch: {
        type: 'boolean',
        short: 'w',
        description: 'Watch mode - regenerate on file changes',
    },
    check: {
        type: 'boolean',
        description: 'Check mode - validate without writing files',
    },
    stdout: {
        type: 'boolean',
        description: 'Print generated output to stdout instead of writing',
    },
    'clean-cache': {
        type: 'boolean',
        description: 'Clear the cache directory and exit',
    },
    'no-cache': {
        type: 'boolean',
        description: 'Disable caching (always regenerate)',
    },
    verbose: {
        type: 'boolean',
        short: 'v',
        description: 'Verbose output',
    },
    quiet: {
        type: 'boolean',
        short: 'q',
        description: 'Suppress output (only show errors)',
    },
    timing: {
        type: 'boolean',
        short: 't',
        description: 'Show timing information for performance debugging',
    },
    'max-diagnostics': {
        type: 'string',
        description: 'Max errors to show per group (0 = all, default 3)',
        argument: 'N',
    },
    help: {
        type: 'boolean',
        short: 'h',
        description: 'Show this help message',
    },
    version: {
        type: 'boolean',
        short: 'V',
        description: 'Show version',
    },
} as const satisfies Record<string, CliOption>

type OptionValue<T> = T extends { type: 'string' }
    ? string | undefined
    : boolean | undefined

export type ParsedArgs = {
    [K in keyof typeof CLI_OPTIONS]?: OptionValue<(typeof CLI_OPTIONS)[K]>
}
