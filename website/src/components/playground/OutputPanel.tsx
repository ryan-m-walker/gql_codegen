import Editor from '@monaco-editor/react'
import type { GenerationResult, OutputTab } from './types'
import { defineCustomTheme, readonlyEditorOptions } from './monaco-themes'
import DiagnosticsView from './DiagnosticsView'

interface OutputPanelProps {
    generatorView: 'codegen' | 'sgc'
    onGeneratorViewChange: (view: 'codegen' | 'sgc') => void
    outputTab: OutputTab
    onOutputTabChange: (tab: OutputTab) => void
    currentResult: GenerationResult | null
    isMounted: boolean
    monacoTheme: string
}

export default function OutputPanel({
    generatorView,
    onGeneratorViewChange,
    outputTab,
    onOutputTabChange,
    currentResult,
    isMounted,
    monacoTheme,
}: OutputPanelProps) {
    const hasErrors = currentResult?.error
    const hasWarnings =
        currentResult?.warnings && currentResult.warnings.length > 0
    const hasDiagnostics = hasErrors || hasWarnings

    return (
        <div className="w-1/2 flex flex-col">
            <div className="px-4 py-2 border-b border-border-default flex items-center justify-between">
                <div className="flex items-center gap-4">
                    <div className="flex items-center gap-2">
                        <button
                            type="button"
                            onClick={() => onGeneratorViewChange('sgc')}
                            className={`px-3 py-1 text-xs font-medium rounded transition-colors ${
                                generatorView === 'sgc'
                                    ? 'bg-green-600 text-white'
                                    : 'bg-surface-inset text-text-muted hover:text-text-heading'
                            }`}
                        >
                            SGC
                        </button>
                        <button
                            type="button"
                            onClick={() => onGeneratorViewChange('codegen')}
                            className={`px-3 py-1 text-xs font-medium rounded transition-colors ${
                                generatorView === 'codegen'
                                    ? 'bg-blue-600 text-white'
                                    : 'bg-surface-inset text-text-muted hover:text-text-heading'
                            }`}
                        >
                            graphql-codegen
                        </button>
                    </div>

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
                            Output
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
                        currentResult &&
                        navigator.clipboard.writeText(currentResult.output)
                    }
                    className="text-xs text-text-faint hover:text-text-heading transition-colors"
                >
                    Copy
                </button>
            </div>

            <div className="flex-1 overflow-hidden">
                {outputTab === 'diagnostics' ? (
                    <DiagnosticsView result={currentResult} />
                ) : currentResult?.error ? (
                    <div className="p-4 text-red-400 font-mono text-sm overflow-auto h-full">
                        <div className="font-semibold mb-2">Error:</div>
                        <pre className="whitespace-pre-wrap">
                            {currentResult.error}
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
                            currentResult?.output ||
                            '// Click Generate to see output...'
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
