import { useCallback } from 'react'
import Editor from '@monaco-editor/react'
import type { InputTab } from './types'
import CopyButton from './CopyButton'
import {
    defineCustomTheme,
    configureMonacoSchema,
    editorOptions,
    configEditorOptions,
} from './monaco-themes'
import { loadWasm } from './generators'

interface InputPanelProps {
    inputTab: InputTab
    onTabChange: (tab: InputTab) => void
    schema: string
    operations: string
    configJson: string
    configError: string | null
    onSchemaChange: (value: string) => void
    onOperationsChange: (value: string) => void
    onConfigChange: (value: string | undefined) => void
    isMounted: boolean
    monacoTheme: string
}

export default function InputPanel({
    inputTab,
    onTabChange,
    schema,
    operations,
    configJson,
    configError,
    onSchemaChange,
    onOperationsChange,
    onConfigChange,
    isMounted,
    monacoTheme,
}: InputPanelProps) {
    const getCopyText = useCallback(() => {
        return inputTab === 'schema' ? schema : inputTab === 'operations' ? operations : configJson
    }, [inputTab, schema, operations, configJson])

    return (
        <div className="flex flex-col min-w-0">
            <div className="flex items-center justify-between h-10 px-3 my-1">
                <div className="flex items-center gap-1">
                    <span className="text-sm font-bold text-text-heading mr-2 pr-3 border-r border-border-muted">Input</span>
                    {(['schema', 'operations', 'config'] as const).map((tab) => (
                        <button
                            key={tab}
                            onClick={() => onTabChange(tab)}
                            className={`px-3 py-1 text-xs font-medium rounded ${
                                inputTab === tab
                                    ? 'bg-amber-400 text-neutral-900'
                                    : 'bg-transparent text-text-muted hover:text-text-heading hover:bg-surface-raised'
                            }`}
                        >
                            {tab.charAt(0).toUpperCase() + tab.slice(1)}
                        </button>
                    ))}
                </div>
                <CopyButton getText={getCopyText} />
            </div>
            <div className="flex-1 overflow-hidden mx-3 mb-3 rounded-lg bg-surface-raised border border-border-default">
                {!isMounted ? (
                    <div className="p-4 text-text-faint font-mono text-sm">
                        Loading editor...
                    </div>
                ) : inputTab === 'config' ? (
                    <div className="flex flex-col h-full">
                        {configError && (
                            <div className="px-3 py-2 bg-red-950/50 border-b border-red-900/50 text-red-400 text-xs font-mono">
                                {configError}
                            </div>
                        )}
                        <div className="flex-1">
                            <Editor
                                height="100%"
                                defaultLanguage="json"
                                value={configJson}
                                onChange={onConfigChange}
                                theme={monacoTheme}
                                options={configEditorOptions}
                                beforeMount={(monaco) => {
                                    defineCustomTheme(monaco)
                                    configureMonacoSchema(monaco, loadWasm)
                                }}
                                loading={
                                    <div className="p-4 text-text-faint">
                                        Loading editor...
                                    </div>
                                }
                            />
                        </div>
                    </div>
                ) : inputTab === 'schema' ? (
                    <Editor
                        height="100%"
                        defaultLanguage="graphql"
                        value={schema}
                        onChange={(value) => onSchemaChange(value || '')}
                        theme={monacoTheme}
                        options={editorOptions}
                        beforeMount={defineCustomTheme}
                        loading={
                            <div className="p-4 text-text-faint">
                                Loading editor...
                            </div>
                        }
                    />
                ) : (
                    <Editor
                        height="100%"
                        defaultLanguage="graphql"
                        value={operations}
                        onChange={(value) => onOperationsChange(value || '')}
                        theme={monacoTheme}
                        options={editorOptions}
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
