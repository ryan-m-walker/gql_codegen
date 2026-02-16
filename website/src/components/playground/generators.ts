// Browser polyfills - MUST be before any graphql-codegen imports
import '../../lib/process-polyfill'

import { parse } from 'graphql'
import { codegen } from '@graphql-codegen/core'
import * as typescriptPlugin from '@graphql-codegen/typescript'
import * as typescriptOperationsPlugin from '@graphql-codegen/typescript-operations'

import type {
    CodegenConfig,
    GenerationResult,
    OutputConfig,
    WasmModule,
} from './types'

// Helper to extract first output config
function getOutputConfig(config: CodegenConfig): OutputConfig | null {
    const outputs = Object.values(config.outputs || {})
    return outputs[0] || null
}

// Lazy-load WASM module
let wasmModule: WasmModule | null = null
let wasmLoadPromise: Promise<WasmModule> | null = null

export async function loadWasm(): Promise<WasmModule> {
    if (wasmModule) return wasmModule
    if (wasmLoadPromise) return wasmLoadPromise

    wasmLoadPromise = (async () => {
        const wasm = await import('../../lib/wasm/gql_codegen_wasm.js')
        await wasm.default()
        wasmModule = wasm as unknown as WasmModule
        return wasmModule
    })()

    return wasmLoadPromise
}

export async function runGraphQLCodegen(
    schemaStr: string,
    operationsStr: string,
    config: CodegenConfig,
): Promise<GenerationResult> {
    const start = performance.now()

    try {
        const outputConfig = getOutputConfig(config)
        if (!outputConfig || !outputConfig.generators?.length) {
            return {
                output: '// No generators configured',
                timeMs: performance.now() - start,
                warnings: [],
            }
        }

        const schema = parse(schemaStr)
        const documents = operationsStr.trim()
            ? [{ document: parse(operationsStr) }]
            : []

        // Map SGC generator names to graphql-codegen plugins
        const plugins: Array<Record<string, unknown>> = []
        const pluginMap: Record<string, unknown> = {}

        for (const generatorName of outputConfig.generators) {
            if (generatorName === 'schema-types' || generatorName === 'typescript') {
                plugins.push({ typescript: {} })
                pluginMap.typescript = typescriptPlugin
            } else if (generatorName === 'operation-types' || generatorName === 'typescript-operations') {
                plugins.push({ 'typescript-operations': {} })
                pluginMap['typescript-operations'] = typescriptOperationsPlugin
            }
        }

        if (plugins.length === 0) {
            return {
                output: '// No supported generators configured',
                timeMs: performance.now() - start,
                warnings: [],
            }
        }

        const output = await codegen({
            schema,
            documents,
            filename: 'types.ts',
            config: outputConfig.config || {},
            plugins,
            pluginMap,
        })

        const timeMs = performance.now() - start
        return { output, timeMs, warnings: [] }
    } catch (e) {
        const timeMs = performance.now() - start
        return {
            output: '',
            timeMs,
            error: e instanceof Error ? e.message : String(e),
            warnings: [],
        }
    }
}

export async function runSGC(
    schemaStr: string,
    operationsStr: string,
    config: CodegenConfig,
): Promise<GenerationResult> {
    const start = performance.now()

    const outputConfig = getOutputConfig(config)
    if (!outputConfig || !outputConfig.generators?.length) {
        return {
            output: '// No generators configured',
            timeMs: performance.now() - start,
            warnings: [],
        }
    }

    try {
        const wasm = await loadWasm()
        const result = wasm.generate(schemaStr, operationsStr, config)
        const timeMs = performance.now() - start

        if (result.error) {
            return {
                output: '',
                timeMs,
                error: result.error,
                warnings: result.warnings || [],
            }
        }

        return {
            output: result.output,
            timeMs,
            warnings: result.warnings || [],
        }
    } catch (e) {
        const timeMs = performance.now() - start
        return {
            output: '',
            timeMs,
            error: e instanceof Error ? e.message : String(e),
            warnings: [],
        }
    }
}
