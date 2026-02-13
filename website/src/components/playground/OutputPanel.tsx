import Editor from '@monaco-editor/react'
import type { GenerationResult, OutputTab } from './types'
import { defineCustomTheme, readonlyEditorOptions } from './monaco-themes'
import DiagnosticsView from './DiagnosticsView'

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
    const hasErrors = result?.error
    const hasWarnings =
        result?.warnings && result.warnings.length > 0
    const hasDiagnostics = hasErrors || hasWarnings

    return (
        <div className="w-1/2 flex flex-col">
            <div className="px-4 py-2 border-b border-border-default flex items-center justify-between">
                <div className="flex items-center gap-4">
                    <span className="px-3 py-1 text-xs font-medium rounded bg-green-600 text-white">
                        Output
                    </span>

                    <div className="flex items-center gap-1 ml-4 border-l border-border-muted pl-4">
                        <button
                            type="button"
                            onClick={() => onOutputTabChange('output')}
                            className={`px-2 py-1 text-xs transition-colors ${
                                outputTab === 'output'
                                    ? 'text-text-heading'
                                    : 'text-text-faint hover:text-text-heading'
                            }`}
                        >
                            TypeScript
                        </button>
                        <button
                            type="button"
                            onClick={() => onOutputTabChange('diagnostics')}
                            className={`px-2 py-1 text-xs transition-colors flex items-center gap-1 ${
                                outputTab === 'diagnostics'
                                    ? 'text-text-heading'
                                    : 'text-text-faint hover:text-text-heading'
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
                </div>
                <button
                    type="button"
                    onClick={() =>
                        result &&
                        navigator.clipboard.writeText(result.output)
                    }
                    className="text-xs text-text-faint hover:text-text-heading transition-colors"
                >
                    Copy
                </button>
            </div>

            <div className="flex-1 overflow-hidden">
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
