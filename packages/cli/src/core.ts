/**
 * Core wrapper - uses NAPI native module with fallback to binary spawn
 */

import type { GenerateOptions, GenerateResult } from '@sgc/core'

let useNative = true
let nativeModule: typeof import('@sgc/core') | null = null

// Try to load native module
async function loadNative(): Promise<typeof import('@sgc/core') | null> {
    if (nativeModule) return nativeModule

    try {
        nativeModule = await import('@sgc/core')
        if (!nativeModule.isNativeAvailable()) {
            const err = nativeModule.getLoadError()
            console.error('[sgc] Native module not available:', err?.message)
            useNative = false
            return null
        }
        return nativeModule
    } catch (e) {
        console.error('[sgc] Failed to load @sgc/core:', e)
        useNative = false
        return null
    }
}

export interface CoreGenerateOptions {
    config: object
    noCache?: boolean
    timing?: boolean
}

export interface CoreGenerateResult {
    fresh: boolean
    files: Array<{ path: string; content: string }>
    warnings: string[]
}

/**
 * Generate using native module or fallback to binary
 */
export async function generate(
    options: CoreGenerateOptions,
): Promise<CoreGenerateResult> {
    const native = await loadNative()

    if (native && useNative) {
        // Use NAPI native module (fast path)
        const result = native.generate({
            configJson: JSON.stringify(options.config),
            noCache: options.noCache,
            timing: options.timing,
        })
        return result
    }

    // Fallback to binary spawn
    return generateViaBinary(options)
}

/**
 * Fallback: spawn the Rust binary
 */
async function generateViaBinary(
    options: CoreGenerateOptions,
): Promise<CoreGenerateResult> {
    const { spawn } = await import('node:child_process')
    const { writeFileSync, mkdtempSync, rmSync } = await import('node:fs')
    const { tmpdir } = await import('node:os')
    const { join } = await import('node:path')

    // Write config to temp file
    const tempDir = mkdtempSync(join(tmpdir(), 'sgc-'))
    const configPath = join(tempDir, 'config.json')
    writeFileSync(configPath, JSON.stringify(options.config))

    try {
        const args = ['-c', configPath]
        if (options.noCache) args.push('--no-cache')
        if (options.timing) args.push('--timing')
        args.push('--stdout') // Output to stdout instead of writing files

        const result = await new Promise<CoreGenerateResult>(
            (resolve, reject) => {
                const proc = spawn('gql-codegen', args, {
                    stdio: ['ignore', 'pipe', 'inherit'],
                })

                let stdout = ''
                proc.stdout.on('data', (data) => {
                    stdout += data.toString()
                })

                proc.on('close', (code) => {
                    if (code === 0) {
                        // Parse stdout as the generated content
                        // For now, return a simple result
                        resolve({
                            fresh: false,
                            files: [], // Would need to parse binary output
                            warnings: [],
                        })
                    } else {
                        reject(
                            new Error(`gql-codegen exited with code ${code}`),
                        )
                    }
                })

                proc.on('error', reject)
            },
        )

        return result
    } finally {
        rmSync(tempDir, { recursive: true, force: true })
    }
}

/**
 * Check if native module is being used
 */
export function isUsingNative(): boolean {
    return useNative && nativeModule !== null
}
