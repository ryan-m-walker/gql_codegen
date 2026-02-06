import LZString from 'lz-string'
import type { CodegenConfig, PlaygroundState } from './types'
import { DEFAULT_SCHEMA, DEFAULT_OPERATIONS, defaultConfig } from './defaults'

function compress(str: string): string {
    return LZString.compressToEncodedURIComponent(str)
}

function decompress(str: string): string | null {
    return LZString.decompressFromEncodedURIComponent(str)
}

export function encodeStateToParams(state: PlaygroundState): string {
    const params = new URLSearchParams()
    params.set('schema', compress(state.schema))
    params.set('operations', compress(state.operations))
    params.set('config', compress(JSON.stringify(state.config)))
    return params.toString()
}

export function getInitialState(): PlaygroundState | null {
    if (typeof window === 'undefined') return null

    const params = new URLSearchParams(window.location.search)
    const schemaParam = params.get('schema')
    const operationsParam = params.get('operations')
    const configParam = params.get('config')

    if (!schemaParam && !operationsParam && !configParam) return null

    try {
        const schema = schemaParam ? decompress(schemaParam) : null
        const operations = operationsParam ? decompress(operationsParam) : null
        const config: CodegenConfig | null = configParam
            ? JSON.parse(decompress(configParam) || '{}')
            : null

        return {
            schema: schema || DEFAULT_SCHEMA,
            operations: operations || DEFAULT_OPERATIONS,
            config: config || defaultConfig,
        }
    } catch {
        return null
    }
}
