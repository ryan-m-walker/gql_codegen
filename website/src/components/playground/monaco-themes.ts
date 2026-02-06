import type { Monaco } from '@monaco-editor/react'

export function defineCustomTheme(monaco: Monaco) {
    monaco.editor.defineTheme('sgc-dark', {
        base: 'vs-dark',
        inherit: true,
        rules: [
            { token: 'keyword', foreground: 'c792ea' },
            { token: 'type', foreground: 'ffcb6b' },
            { token: 'string', foreground: 'c3e88d' },
            { token: 'number', foreground: 'f78c6c' },
            { token: 'comment', foreground: '546e7a' },
            { token: 'string.key.json', foreground: '82aaff' },
            { token: 'string.value.json', foreground: 'c3e88d' },
            { token: 'number.json', foreground: 'f78c6c' },
            { token: 'keyword.json', foreground: 'c792ea' },
        ],
        colors: {
            'editor.background': '#00000000',
            'editor.lineHighlightBackground': '#ffffff08',
            'editor.selectionBackground': '#ffffff20',
            'editorLineNumber.foreground': '#4a5568',
            'editorLineNumber.activeForeground': '#718096',
            'editorCursor.foreground': '#ffcb6b',
            'editor.selectionHighlightBackground': '#ffffff10',
            'editorIndentGuide.background': '#ffffff10',
            'editorIndentGuide.activeBackground': '#ffffff20',
            'scrollbarSlider.background': '#ffffff15',
            'scrollbarSlider.hoverBackground': '#ffffff25',
            'scrollbarSlider.activeBackground': '#ffffff30',
            'editorBracketHighlight.foreground1': '#9ca3af',
            'editorBracketHighlight.foreground2': '#9ca3af',
            'editorBracketHighlight.foreground3': '#9ca3af',
            'editorBracketHighlight.foreground4': '#9ca3af',
            'editorBracketHighlight.foreground5': '#9ca3af',
            'editorBracketHighlight.foreground6': '#9ca3af',
            'editorBracketPairGuide.activeBackground1': '#00000000',
            'editorBracketPairGuide.activeBackground2': '#00000000',
            'editorBracketPairGuide.activeBackground3': '#00000000',
            'editorBracketPairGuide.activeBackground4': '#00000000',
            'editorBracketPairGuide.activeBackground5': '#00000000',
            'editorBracketPairGuide.activeBackground6': '#00000000',
        },
    })

    monaco.editor.defineTheme('sgc-light', {
        base: 'vs',
        inherit: true,
        rules: [
            { token: 'keyword', foreground: '7c3aed' },     // violet-600
            { token: 'type', foreground: 'b45309' },         // amber-700
            { token: 'string', foreground: '15803d' },       // green-700
            { token: 'number', foreground: 'c2410c' },       // orange-700
            { token: 'comment', foreground: '94a3b8' },      // slate-400
            { token: 'string.key.json', foreground: '2563eb' },  // blue-600
            { token: 'string.value.json', foreground: '15803d' },// green-700
            { token: 'number.json', foreground: 'c2410c' },     // orange-700
            { token: 'keyword.json', foreground: '7c3aed' },    // violet-600
        ],
        colors: {
            'editor.background': '#00000000',
            'editor.lineHighlightBackground': '#00000006',
            'editor.selectionBackground': '#7c3aed20',
            'editorLineNumber.foreground': '#94a3b8',
            'editorLineNumber.activeForeground': '#64748b',
            'editorCursor.foreground': '#7c3aed',
            'editor.selectionHighlightBackground': '#7c3aed10',
            'editorIndentGuide.background': '#00000010',
            'editorIndentGuide.activeBackground': '#00000020',
            'scrollbarSlider.background': '#00000015',
            'scrollbarSlider.hoverBackground': '#00000025',
            'scrollbarSlider.activeBackground': '#00000030',
            'editorBracketHighlight.foreground1': '#6b7280',
            'editorBracketHighlight.foreground2': '#6b7280',
            'editorBracketHighlight.foreground3': '#6b7280',
            'editorBracketHighlight.foreground4': '#6b7280',
            'editorBracketHighlight.foreground5': '#6b7280',
            'editorBracketHighlight.foreground6': '#6b7280',
            'editorBracketPairGuide.activeBackground1': '#00000000',
            'editorBracketPairGuide.activeBackground2': '#00000000',
            'editorBracketPairGuide.activeBackground3': '#00000000',
            'editorBracketPairGuide.activeBackground4': '#00000000',
            'editorBracketPairGuide.activeBackground5': '#00000000',
            'editorBracketPairGuide.activeBackground6': '#00000000',
        },
    })
}

// Configure Monaco JSON with the config schema for intellisense
export async function configureMonacoSchema(
    monaco: Monaco,
    loadWasm: () => Promise<{ getConfigSchema: () => string }>,
) {
    try {
        const wasm = await loadWasm()
        const schemaJson = wasm.getConfigSchema()
        const pluginOptionsSchema = JSON.parse(schemaJson) as Record<
            string,
            unknown
        >

        const fullSchema = {
            type: 'object',
            properties: {
                preset: {
                    type: 'string',
                    enum: ['sgc', 'graphql-codegen'],
                    description:
                        'Preset for default configuration values. "sgc" is optimized for TypeScript performance, "graphql-codegen" is compatible with graphql-codegen output.',
                },
                generates: {
                    type: 'object',
                    additionalProperties: {
                        type: 'object',
                        properties: {
                            plugins: {
                                type: 'array',
                                items: {
                                    type: 'string',
                                    enum: [
                                        'typescript',
                                        'typescript-operations',
                                    ],
                                },
                                description:
                                    'Plugins to run for this output file',
                            },
                            config: pluginOptionsSchema,
                        },
                        required: ['plugins'],
                    },
                    description: 'Output file configurations',
                },
            },
        }

        monaco.languages.json.jsonDefaults.setDiagnosticsOptions({
            validate: true,
            schemas: [
                {
                    uri: 'https://sgc.dev/config-schema.json',
                    fileMatch: ['*'],
                    schema: fullSchema,
                },
            ],
        })
    } catch (e) {
        console.warn('Failed to configure config schema:', e)
    }
}

export const editorOptions = {
    minimap: { enabled: false },
    fontSize: 13,
    lineNumbers: 'on' as const,
    scrollBeyondLastLine: false,
    automaticLayout: true,
    padding: { top: 12, bottom: 12 },
    fontFamily: 'JetBrains Mono, Menlo, Monaco, Consolas, monospace',
    fontLigatures: true,
    tabSize: 2,
    folding: false,
    foldingHighlight: false,
    showFoldingControls: 'never' as const,
    glyphMargin: false,
    lineDecorationsWidth: 16,
    lineNumbersMinChars: 3,
    renderLineHighlight: 'line' as const,
    scrollbar: {
        vertical: 'auto' as const,
        horizontal: 'auto' as const,
        verticalScrollbarSize: 8,
        horizontalScrollbarSize: 8,
    },
    overviewRulerBorder: false,
    overviewRulerLanes: 0,
    hideCursorInOverviewRuler: true,
    contextmenu: false,
    quickSuggestions: false,
    suggestOnTriggerCharacters: false,
    wordBasedSuggestions: 'off' as const,
    parameterHints: { enabled: false },
    lightbulb: { enabled: 'off' as const },
    hover: { enabled: false },
    links: false,
    occurrencesHighlight: 'off' as const,
    selectionHighlight: false,
    renderWhitespace: 'none' as const,
    bracketPairColorization: { enabled: false },
    guides: { bracketPairs: false },
    matchBrackets: 'always' as const,
    stickyScroll: { enabled: false },
}

export const configEditorOptions = {
    ...editorOptions,
    quickSuggestions: true,
    suggestOnTriggerCharacters: true,
    hover: { enabled: true },
}

export const readonlyEditorOptions = {
    ...editorOptions,
    readOnly: true,
    domReadOnly: true,
    cursorStyle: 'line' as const,
    cursorBlinking: 'solid' as const,
}
