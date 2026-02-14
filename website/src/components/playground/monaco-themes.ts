import type { Monaco } from '@monaco-editor/react'

let themeDefined = false

export function defineCustomTheme(monaco: Monaco) {
    if (themeDefined) return
    themeDefined = true
    monaco.editor.defineTheme('sgc-dark', {
        base: 'vs-dark',
        inherit: true,
        rules: [
            { token: 'keyword', foreground: 'fbbf24' },           // amber-400
            { token: 'type', foreground: 'fbbf24' },              // amber-400
            { token: 'string', foreground: 'fbbf24' },            // amber-400
            { token: 'number', foreground: 'a3a3a3' },            // neutral-400
            { token: 'comment', foreground: '525252' },            // neutral-600
            { token: 'string.key.json', foreground: 'd4d4d4' },   // neutral-300
            { token: 'string.value.json', foreground: 'fbbf24' }, // amber-400
            { token: 'number.json', foreground: 'a3a3a3' },       // neutral-400
            { token: 'keyword.json', foreground: 'fbbf24' },      // amber-400
        ],
        colors: {
            'editor.background': '#171717',
            'editor.lineHighlightBackground': '#ffffff08',
            'editor.selectionBackground': '#fbbf2420',
            'editorLineNumber.foreground': '#404040',
            'editorLineNumber.activeForeground': '#737373',
            'editorCursor.foreground': '#fbbf24',
            'editor.selectionHighlightBackground': '#fbbf2410',
            'editorIndentGuide.background': '#ffffff10',
            'editorIndentGuide.activeBackground': '#ffffff20',
            'scrollbarSlider.background': '#ffffff15',
            'scrollbarSlider.hoverBackground': '#ffffff25',
            'scrollbarSlider.activeBackground': '#ffffff30',
            'editorBracketHighlight.foreground1': '#737373',
            'editorBracketHighlight.foreground2': '#737373',
            'editorBracketHighlight.foreground3': '#737373',
            'editorBracketHighlight.foreground4': '#737373',
            'editorBracketHighlight.foreground5': '#737373',
            'editorBracketHighlight.foreground6': '#737373',
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
            { token: 'keyword', foreground: 'b45309' },           // amber-700
            { token: 'type', foreground: 'b45309' },              // amber-700
            { token: 'string', foreground: '525252' },            // neutral-600
            { token: 'number', foreground: '737373' },            // neutral-500
            { token: 'comment', foreground: 'a3a3a3' },           // neutral-400
            { token: 'string.key.json', foreground: '404040' },   // neutral-700
            { token: 'string.value.json', foreground: '525252' }, // neutral-600
            { token: 'number.json', foreground: '737373' },       // neutral-500
            { token: 'keyword.json', foreground: 'b45309' },      // amber-700
        ],
        colors: {
            'editor.background': '#f5f5f5',
            'editor.lineHighlightBackground': '#00000006',
            'editor.selectionBackground': '#b4530920',
            'editorLineNumber.foreground': '#a3a3a3',
            'editorLineNumber.activeForeground': '#737373',
            'editorCursor.foreground': '#b45309',
            'editor.selectionHighlightBackground': '#b4530910',
            'editorIndentGuide.background': '#00000010',
            'editorIndentGuide.activeBackground': '#00000020',
            'scrollbarSlider.background': '#00000015',
            'scrollbarSlider.hoverBackground': '#00000025',
            'scrollbarSlider.activeBackground': '#00000030',
            'editorBracketHighlight.foreground1': '#737373',
            'editorBracketHighlight.foreground2': '#737373',
            'editorBracketHighlight.foreground3': '#737373',
            'editorBracketHighlight.foreground4': '#737373',
            'editorBracketHighlight.foreground5': '#737373',
            'editorBracketHighlight.foreground6': '#737373',
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
