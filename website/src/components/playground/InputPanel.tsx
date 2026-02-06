import Editor from '@monaco-editor/react'
import type { InputTab } from './types'
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
    return (
        <div className="w-1/2 flex flex-col border-r border-border-default">
            <div className="flex border-b border-border-default">
                {(['schema', 'operations', 'config'] as const).map((tab) => (
                    <button
                        key={tab}
                        onClick={() => onTabChange(tab)}
                        className={`px-4 py-2 text-sm font-medium transition-colors ${
                            inputTab === tab
                                ? 'text-text-heading border-b-2 border-green-500 bg-surface-raised/50'
                                : 'text-text-muted hover:text-text-heading'
                        }`}
                    >
                        {tab.charAt(0).toUpperCase() + tab.slice(1)}
                    </button>
                ))}
            </div>

            <div className="flex-1 overflow-hidden">
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
