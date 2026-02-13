export type InputTab = 'schema' | 'operations' | 'config'
export type OutputTab = 'output' | 'diagnostics'

// Matches SGC config structure
export interface OutputConfig {
    plugins: string[]
    config?: Record<string, unknown>
}

export interface CodegenConfig {
    generates: {
        [outputPath: string]: OutputConfig
    }
}

export interface GenerationResult {
    output: string
    timeMs: number
    error?: string
    warnings: string[]
}

// WASM module types
export interface WasmGenerateResult {
    output: string
    error?: string
    warnings: string[]
}

export interface WasmModule {
    generate: (
        schema: string | string[],
        operations: string | string[],
        config: unknown,
    ) => WasmGenerateResult
    getConfigSchema: () => string
}

// URL state management
export interface PlaygroundState {
    schema: string
    operations: string
    config: CodegenConfig
}
