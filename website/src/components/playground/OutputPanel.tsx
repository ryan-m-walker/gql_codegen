import { useCallback } from 'react'
import Editor from '@monaco-editor/react'
import type { GenerationResult, OutputTab } from './types'
import { defineCustomTheme, readonlyEditorOptions } from './monaco-themes'
import DiagnosticsView from './DiagnosticsView'
import CopyButton from './CopyButton'

interface OutputPanelProps {
    outputTab: OutputTab
    onOutputTabChange: (tab: OutputTab) => void
    result: GenerationResult | null
    isMounted: boolean
    monacoTheme: string
}

export default function OutputPanel({
    outputTab,
    onOutputTabChange,
    result,
    isMounted,
    monacoTheme,
}: OutputPanelProps) {
    const getCopyText = useCallback(() => {
        if (!result) return ''
        if (outputTab === 'diagnostics') {
            const parts: string[] = []
            if (result.error) parts.push(`Error:\n${result.error}`)
            if (result.warnings.length > 0) parts.push(`Warnings:\n${result.warnings.join('\n')}`)
            if (parts.length === 0) parts.push('No errors or warnings')
            return parts.join('\n\n')
        }
        return result.output
    }, [outputTab, result])

    const hasErrors = result?.error
    const hasWarnings =
        result?.warnings && result.warnings.length > 0
    const hasDiagnostics = hasErrors || hasWarnings

    return (
        <div className="flex flex-col min-w-0">
            <div className="flex items-center justify-between h-10 px-3 my-1">
                <div className="flex items-center gap-1">
                    <span className="text-sm font-bold text-text-heading mr-2 pr-3 border-r border-border-muted">Output</span>
                    <button
                        type="button"
                        onClick={() => onOutputTabChange('output')}
                        className={`px-3 py-1 text-xs font-medium rounded ${
                            outputTab === 'output'
                                ? 'bg-amber-400 text-neutral-900'
                                : 'bg-transparent text-text-muted hover:text-text-heading hover:bg-surface-raised'
                        }`}
                    >
                        TypeScript
                    </button>
                    <button
                        type="button"
                        onClick={() => onOutputTabChange('diagnostics')}
                        className={`px-3 py-1 text-xs font-medium rounded flex items-center gap-1 ${
                            outputTab === 'diagnostics'
                                ? 'bg-amber-400 text-neutral-900'
                                : 'bg-transparent text-text-muted hover:text-text-heading hover:bg-surface-raised'
                        }`}
                    >
                        Diagnostics
                        {hasDiagnostics && (
                            <span
                                className={`w-2 h-2 rounded-full ${hasErrors ? 'bg-red-500' : 'bg-yellow-500'}`}
                            />
                        )}
                    </button>
                </div>
                <CopyButton getText={getCopyText} />
            </div>

            <div className="flex-1 overflow-hidden mx-3 mb-3 rounded-lg bg-surface-raised border border-border-default">
                {outputTab === 'diagnostics' ? (
                    <DiagnosticsView result={result} />
                ) : result?.error ? (
                    <div className="p-4 text-red-400 font-mono text-sm overflow-auto h-full">
                        <div className="font-semibold mb-2">Error:</div>
                        <pre className="whitespace-pre-wrap">
                            {result.error}
                        </pre>
                    </div>
                ) : !isMounted ? (
                    <div className="p-4 text-text-faint font-mono text-sm">
                        Loading editor...
                    </div>
                ) : (
                    <Editor
                        height="100%"
                        defaultLanguage="typescript"
                        value={
                            result?.output ||
                            '// Edit schema or operations to see generated output...'
                        }
                        theme={monacoTheme}
                        options={readonlyEditorOptions}
                        beforeMount={defineCustomTheme}
                        loading={
                            <div className="p-4 text-text-faint">
                                Loading editor...
                            </div>
                        }
                    />
                )}
            </div>
        </div>
    )
}
