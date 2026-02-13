import { useState, useCallback, useEffect, useRef } from 'react'
import { loader } from '@monaco-editor/react'

import type { CodegenConfig, InputTab, OutputTab } from './types'
import { DEFAULT_SCHEMA, DEFAULT_OPERATIONS, defaultConfig } from './defaults'
import { runSGC } from './generators'
import { encodeStateToParams, getInitialState } from './url-state'
import Toolbar from './Toolbar'
import InputPanel from './InputPanel'
import OutputPanel from './OutputPanel'

// Configure Monaco to use CDN
loader.config({
    paths: {
        vs: 'https://cdn.jsdelivr.net/npm/monaco-editor@0.55.1/min/vs',
    },
})

export default function Playground() {
    const initialState =
        typeof window !== 'undefined' ? getInitialState() : null

    const [schema, setSchema] = useState(initialState?.schema ?? DEFAULT_SCHEMA)
    const [operations, setOperations] = useState(
        initialState?.operations ?? DEFAULT_OPERATIONS,
    )
    const [config, setConfig] = useState<CodegenConfig>(
        initialState?.config ?? defaultConfig,
    )
    const [configJson, setConfigJson] = useState(() =>
        JSON.stringify(initialState?.config ?? defaultConfig, null, 2),
    )
    const [configError, setConfigError] = useState<string | null>(null)
    const [inputTab, setInputTab] = useState<InputTab>('schema')
    const [outputTab, setOutputTab] = useState<OutputTab>('output')
    const [isGenerating, setIsGenerating] = useState(false)
    const [isMounted, setIsMounted] = useState(false)
    const [shareMessage, setShareMessage] = useState<string | null>(null)
    const [result, setResult] = useState<import('./types').GenerationResult | null>(null)

    // Theme-aware Monaco: track whether we're in dark mode
    const [monacoTheme, setMonacoTheme] = useState('sgc-dark')

    const debounceRef = useRef<NodeJS.Timeout | null>(null)
    const urlUpdateRef = useRef<NodeJS.Timeout | null>(null)

    // Track client-side mount + observe theme changes
    useEffect(() => {
        setIsMounted(true)

        // Set initial theme from DOM
        const isDark = document.documentElement.classList.contains('dark')
        setMonacoTheme(isDark ? 'sgc-dark' : 'sgc-light')

        // Watch for class changes on <html> to detect theme toggles
        const observer = new MutationObserver(() => {
            const dark = document.documentElement.classList.contains('dark')
            setMonacoTheme(dark ? 'sgc-dark' : 'sgc-light')
        })
        observer.observe(document.documentElement, {
            attributes: true,
            attributeFilter: ['class'],
        })

        return () => observer.disconnect()
    }, [])

    // Update URL query params when state changes (debounced)
    useEffect(() => {
        if (!isMounted) return

        if (urlUpdateRef.current) clearTimeout(urlUpdateRef.current)

        urlUpdateRef.current = setTimeout(() => {
            const params = encodeStateToParams({ schema, operations, config })
            window.history.replaceState(null, '', `?${params}`)
        }, 500)

        return () => {
            if (urlUpdateRef.current) clearTimeout(urlUpdateRef.current)
        }
    }, [isMounted, schema, operations, config])

    // Auto-generate with debounce when inputs change
    useEffect(() => {
        if (!isMounted) return

        if (debounceRef.current) clearTimeout(debounceRef.current)

        debounceRef.current = setTimeout(() => {
            handleGenerate()
        }, 400)

        return () => {
            if (debounceRef.current) clearTimeout(debounceRef.current)
        }
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [isMounted, schema, operations, config])

    const handleShare = useCallback(() => {
        const params = encodeStateToParams({ schema, operations, config })
        const url = `${window.location.origin}${window.location.pathname}?${params}`
        navigator.clipboard.writeText(url).then(() => {
            setShareMessage('Link copied!')
            setTimeout(() => setShareMessage(null), 2000)
        })
    }, [schema, operations, config])

    const handleConfigJsonChange = useCallback(
        (value: string | undefined) => {
            const json = value || ''
            setConfigJson(json)
            try {
                const parsed = JSON.parse(json) as CodegenConfig
                setConfig(parsed)
                setConfigError(null)
            } catch (e) {
                setConfigError(e instanceof Error ? e.message : 'Invalid JSON')
            }
        },
        [],
    )

    const handleGenerate = useCallback(async () => {
        setIsGenerating(true)
        const sgcResult = await runSGC(schema, operations, config)
        setResult(sgcResult)
        setIsGenerating(false)
    }, [schema, operations, config])

    return (
        <div className="flex flex-col h-full">
            <Toolbar
                isGenerating={isGenerating}
                result={result}
                onShare={handleShare}
                shareMessage={shareMessage}
            />

            <div className="flex flex-1 overflow-hidden">
                <InputPanel
                    inputTab={inputTab}
                    onTabChange={setInputTab}
                    schema={schema}
                    operations={operations}
                    configJson={configJson}
                    configError={configError}
                    onSchemaChange={setSchema}
                    onOperationsChange={setOperations}
                    onConfigChange={handleConfigJsonChange}
                    isMounted={isMounted}
                    monacoTheme={monacoTheme}
                />

                <OutputPanel
                    outputTab={outputTab}
                    onOutputTabChange={setOutputTab}
                    result={result}
                    isMounted={isMounted}
                    monacoTheme={monacoTheme}
                />
            </div>

            <div className="px-4 py-2 border-t border-border-default text-xs text-text-faint flex items-center justify-between">
                <span>
                    Edit the schema and operations to see generated TypeScript types.
                </span>
                <span>Powered by SGC (Rust/WASM) — runs entirely in your browser.</span>
            </div>
        </div>
    )
}
