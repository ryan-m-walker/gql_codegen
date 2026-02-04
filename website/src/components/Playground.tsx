// Browser polyfills - MUST be before any graphql-codegen imports
import '../lib/process-polyfill';

import { useState, useCallback, useEffect, useRef } from 'react';
import { parse } from 'graphql';
import { codegen } from '@graphql-codegen/core';
import * as typescriptPlugin from '@graphql-codegen/typescript';
import * as typescriptOperationsPlugin from '@graphql-codegen/typescript-operations';
import Editor, { loader, type Monaco } from '@monaco-editor/react';
import LZString from 'lz-string';

// Configure Monaco to use CDN
loader.config({
  paths: {
    vs: 'https://cdn.jsdelivr.net/npm/monaco-editor@0.55.1/min/vs',
  },
});

// Custom theme matching our site's dark background
function defineCustomTheme(monaco: Monaco) {
  monaco.editor.defineTheme('sgc-dark', {
    base: 'vs-dark',
    inherit: true,
    rules: [
      { token: 'keyword', foreground: 'c792ea' },
      { token: 'type', foreground: 'ffcb6b' },
      { token: 'string', foreground: 'c3e88d' },
      { token: 'number', foreground: 'f78c6c' },
      { token: 'comment', foreground: '546e7a' },
      // JSON-specific tokens
      { token: 'string.key.json', foreground: '82aaff' },  // Property keys - blue
      { token: 'string.value.json', foreground: 'c3e88d' }, // String values - green
      { token: 'number.json', foreground: 'f78c6c' },       // Numbers - orange
      { token: 'keyword.json', foreground: 'c792ea' },      // true/false/null - purple
    ],
    colors: {
      'editor.background': '#00000000', // Transparent
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
      // Disable bracket pair colorization by setting all to same color
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
  });
}

// WASM module types
interface WasmGenerateResult {
  output: string;
  error?: string;
  warnings: string[];
}

interface WasmModule {
  generate: (
    schema: string | string[],
    operations: string | string[],
    config: unknown
  ) => WasmGenerateResult;
  getConfigSchema: () => string;  // Returns JSON string
}

// Lazy-load WASM module
let wasmModule: WasmModule | null = null;
let wasmLoadPromise: Promise<WasmModule> | null = null;
let configSchemaConfigured = false;

async function loadWasm(): Promise<WasmModule> {
  if (wasmModule) return wasmModule;
  if (wasmLoadPromise) return wasmLoadPromise;

  wasmLoadPromise = (async () => {
    const wasm = await import('../lib/wasm/gql_codegen_wasm.js');
    await wasm.default();
    wasmModule = wasm as unknown as WasmModule;
    return wasmModule;
  })();

  return wasmLoadPromise;
}

// Configure Monaco JSON with the config schema for intellisense
async function configureMonacoSchema(monaco: Monaco) {
  if (configSchemaConfigured) return;

  try {
    const wasm = await loadWasm();
    const schemaJson = wasm.getConfigSchema();
    const pluginOptionsSchema = JSON.parse(schemaJson) as Record<string, unknown>;
    console.log('SGC Plugin Options Schema:', pluginOptionsSchema);

    // Wrap the PluginOptions schema into a full config schema
    const fullSchema = {
      type: 'object',
      properties: {
        preset: {
          type: 'string',
          enum: ['sgc', 'graphql-codegen'],
          description: 'Preset for default configuration values. "sgc" is optimized for TypeScript performance, "graphql-codegen" is compatible with graphql-codegen output.',
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
                  enum: ['typescript', 'typescript-operations'],
                },
                description: 'Plugins to run for this output file',
              },
              config: pluginOptionsSchema,
            },
            required: ['plugins'],
          },
          description: 'Output file configurations',
        },
      },
    };

    console.log('SGC Full Config Schema:', fullSchema);

    monaco.languages.json.jsonDefaults.setDiagnosticsOptions({
      validate: true,
      schemas: [
        {
          uri: 'https://sgc.dev/config-schema.json',
          fileMatch: ['*'],
          schema: fullSchema,
        },
      ],
    });

    console.log('SGC Monaco schema configured successfully');
    configSchemaConfigured = true;
  } catch (e) {
    console.warn('Failed to configure config schema:', e);
  }
}

// URL state management - compress each param individually
interface PlaygroundState {
  schema: string;
  operations: string;
  config: CodegenConfig;
}

function compress(str: string): string {
  return LZString.compressToEncodedURIComponent(str);
}

function decompress(str: string): string | null {
  return LZString.decompressFromEncodedURIComponent(str);
}

function encodeStateToParams(state: PlaygroundState): string {
  const params = new URLSearchParams();
  params.set('schema', compress(state.schema));
  params.set('operations', compress(state.operations));
  params.set('config', compress(JSON.stringify(state.config)));
  return params.toString();
}

function getInitialState(): PlaygroundState | null {
  if (typeof window === 'undefined') return null;

  const params = new URLSearchParams(window.location.search);
  const schemaParam = params.get('schema');
  const operationsParam = params.get('operations');
  const configParam = params.get('config');

  // If no params, return null to use defaults
  if (!schemaParam && !operationsParam && !configParam) return null;

  try {
    const schema = schemaParam ? decompress(schemaParam) : null;
    const operations = operationsParam ? decompress(operationsParam) : null;
    const config = configParam ? JSON.parse(decompress(configParam) || '{}') : null;

    return {
      schema: schema || DEFAULT_SCHEMA,
      operations: operations || DEFAULT_OPERATIONS,
      config: config || defaultConfig,
    };
  } catch {
    return null;
  }
}

const DEFAULT_SCHEMA = `scalar DateTime

type Post {
  id: ID!
  title: String!
  content: String
  author: User!
  published: Boolean!
}

type Query {
  user(id: ID!): User
  users: [User!]!
}

enum Role {
    ADMIN
    USER
}

type User {
  id: ID!
  name: String!
  email: String!
  posts: [Post!]!
  role: Role!
  createdAt: DateTime!
}`;


const DEFAULT_OPERATIONS = `query GetUser($id: ID!) {
  user(id: $id) {
    id
    name
    email
    posts {
      id
      title
      published
    }
  }
}

query GetUsers {
  users {
    id
    name
  }
}

fragment UserFields on User {
  id
  name
  email
}`;

type InputTab = 'schema' | 'operations' | 'config';
type OutputTab = 'output' | 'diagnostics';
type Preset = 'sgc' | 'graphql-codegen';

// Matches real SGC/graphql-codegen config structure
interface OutputConfig {
  plugins: string[];
  config?: Record<string, unknown>;
}

interface CodegenConfig {
  preset?: Preset;
  generates: {
    [outputPath: string]: OutputConfig;
  };
}

// Default config - preset handles defaults internally in core
const defaultConfig: CodegenConfig = {
  preset: 'graphql-codegen',
  generates: {
    'types.ts': {
      plugins: ['typescript', 'typescript-operations'],
      // No config needed - preset defaults applied by core
    },
  },
};

// Helper to extract first output config
function getOutputConfig(config: CodegenConfig): OutputConfig | null {
  const outputs = Object.values(config.generates || {});
  return outputs[0] || null;
}

interface GenerationResult {
  output: string;
  timeMs: number;
  error?: string;
  warnings: string[];
}

async function runGraphQLCodegen(
  schemaStr: string,
  operationsStr: string,
  config: CodegenConfig
): Promise<GenerationResult> {
  const start = performance.now();

  try {
    const outputConfig = getOutputConfig(config);
    if (!outputConfig || outputConfig.plugins.length === 0) {
      return {
        output: '// No plugins configured',
        timeMs: performance.now() - start,
        warnings: [],
      };
    }

    const schema = parse(schemaStr);
    const documents = operationsStr.trim()
      ? [{ document: parse(operationsStr) }]
      : [];

    // Build plugins list from config
    // See: https://the-guild.dev/graphql/codegen/plugins/typescript/typescript
    const plugins: Array<Record<string, unknown>> = [];
    const pluginMap: Record<string, unknown> = {};

    for (const pluginName of outputConfig.plugins) {
      if (pluginName === 'typescript') {
        plugins.push({ typescript: {} });
        pluginMap.typescript = typescriptPlugin;
      } else if (pluginName === 'typescript-operations') {
        plugins.push({ 'typescript-operations': {} });
        pluginMap['typescript-operations'] = typescriptOperationsPlugin;
      }
    }

    if (plugins.length === 0) {
      return {
        output: '// No supported plugins configured',
        timeMs: performance.now() - start,
        warnings: [],
      };
    }

    const output = await codegen({
      schema,
      documents,
      filename: 'types.ts',
      config: outputConfig.config || {},
      plugins,
      pluginMap,
    });

    const timeMs = performance.now() - start;
    return { output, timeMs, warnings: [] };
  } catch (e) {
    const timeMs = performance.now() - start;
    return {
      output: '',
      timeMs,
      error: e instanceof Error ? e.message : String(e),
      warnings: [],
    };
  }
}

// SGC via WASM
async function runSGC(
  schemaStr: string,
  operationsStr: string,
  config: CodegenConfig
): Promise<GenerationResult> {
  const start = performance.now();

  const outputConfig = getOutputConfig(config);
  if (!outputConfig || outputConfig.plugins.length === 0) {
    return {
      output: '// No plugins configured',
      timeMs: performance.now() - start,
      warnings: [],
    };
  }

  try {
    const wasm = await loadWasm();
    // Pass schema, operations, and config to WASM
    const result = wasm.generate(schemaStr, operationsStr, config);
    const timeMs = performance.now() - start;

    if (result.error) {
      return {
        output: '',
        timeMs,
        error: result.error,
        warnings: result.warnings || [],
      };
    }

    return {
      output: result.output,
      timeMs,
      warnings: result.warnings || [],
    };
  } catch (e) {
    const timeMs = performance.now() - start;
    return {
      output: '',
      timeMs,
      error: e instanceof Error ? e.message : String(e),
      warnings: [],
    };
  }
}

// Monaco editor options - simplified and clean
const editorOptions = {
  minimap: { enabled: false },
  fontSize: 13,
  lineNumbers: 'on' as const,
  scrollBeyondLastLine: false,
  automaticLayout: true,
  padding: { top: 12, bottom: 12 },
  fontFamily: 'JetBrains Mono, Menlo, Monaco, Consolas, monospace',
  fontLigatures: true,
  tabSize: 2,
  // Simplify UI
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
  // Disable rainbow brackets but keep bracket matching
  bracketPairColorization: { enabled: false },
  guides: { bracketPairs: false },
  matchBrackets: 'always' as const,
  stickyScroll: { enabled: false },
};

// Config editor needs suggestions enabled for schema autocomplete
const configEditorOptions = {
  ...editorOptions,
  quickSuggestions: true,
  suggestOnTriggerCharacters: true,
  hover: { enabled: true },
};

const readonlyEditorOptions = {
  ...editorOptions,
  readOnly: true,
  domReadOnly: true,
  cursorStyle: 'line' as const,
  cursorBlinking: 'solid' as const,
};

export default function Playground() {
  // Initialize state from URL or defaults
  const initialState = typeof window !== 'undefined' ? getInitialState() : null;

  const [schema, setSchema] = useState(initialState?.schema ?? DEFAULT_SCHEMA);
  const [operations, setOperations] = useState(initialState?.operations ?? DEFAULT_OPERATIONS);
  const [config, setConfig] = useState<CodegenConfig>(initialState?.config ?? defaultConfig);
  const [configJson, setConfigJson] = useState(() => JSON.stringify(initialState?.config ?? defaultConfig, null, 2));
  const [configError, setConfigError] = useState<string | null>(null);
  const [inputTab, setInputTab] = useState<InputTab>('schema');
  const [outputTab, setOutputTab] = useState<OutputTab>('output');
  const [isGenerating, setIsGenerating] = useState(false);
  const [isMounted, setIsMounted] = useState(false);
  const [shareMessage, setShareMessage] = useState<string | null>(null);

  // Results for each generator
  const [codegenResult, setCodegenResult] = useState<GenerationResult | null>(null);
  const [sgcResult, setSgcResult] = useState<GenerationResult | null>(null);

  // Which generator to show: 'codegen' or 'sgc'
  const [generatorView, setGeneratorView] = useState<'codegen' | 'sgc'>('sgc');

  // Debounce timer refs
  const debounceRef = useRef<NodeJS.Timeout | null>(null);
  const urlUpdateRef = useRef<NodeJS.Timeout | null>(null);

  // Track client-side mount (Monaco doesn't work in SSR)
  useEffect(() => {
    setIsMounted(true);
  }, []);

  // Update URL query params when state changes (debounced)
  useEffect(() => {
    if (!isMounted) return;

    // Clear previous timer
    if (urlUpdateRef.current) {
      clearTimeout(urlUpdateRef.current);
    }

    // Debounce URL updates to avoid thrashing history
    urlUpdateRef.current = setTimeout(() => {
      const params = encodeStateToParams({ schema, operations, config });
      window.history.replaceState(null, '', `?${params}`);
    }, 500);

    return () => {
      if (urlUpdateRef.current) {
        clearTimeout(urlUpdateRef.current);
      }
    };
  }, [isMounted, schema, operations, config]);

  // Auto-generate with debounce when inputs change
  useEffect(() => {
    if (!isMounted) return;

    // Clear previous timer
    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }

    // Set new debounced generation
    debounceRef.current = setTimeout(() => {
      handleGenerate();
    }, 400);

    // Cleanup on unmount or before next effect
    return () => {
      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
      }
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isMounted, schema, operations, config]);

  // Copy share link to clipboard
  const handleShare = useCallback(() => {
    const params = encodeStateToParams({ schema, operations, config });
    const url = `${window.location.origin}${window.location.pathname}?${params}`;
    navigator.clipboard.writeText(url).then(() => {
      setShareMessage('Link copied!');
      setTimeout(() => setShareMessage(null), 2000);
    });
  }, [schema, operations, config]);

  // Handle config JSON changes
  const handleConfigJsonChange = useCallback((value: string | undefined) => {
    const json = value || '';
    setConfigJson(json);
    try {
      const parsed = JSON.parse(json) as CodegenConfig;
      // Preserve preset from parsed JSON if present, otherwise keep current
      // This prevents Rust's #[default] (Sgc) from overriding when preset is omitted
      if (parsed.preset === undefined) {
        parsed.preset = config.preset;
      }
      setConfig(parsed);
      setConfigError(null);
    } catch (e) {
      setConfigError(e instanceof Error ? e.message : 'Invalid JSON');
    }
  }, [config.preset]);

  const handleGenerate = useCallback(async () => {
    setIsGenerating(true);

    // Run both generators in parallel
    const [codegenRes, sgcRes] = await Promise.all([
      runGraphQLCodegen(schema, operations, config),
      runSGC(schema, operations, config),
    ]);

    setCodegenResult(codegenRes);
    setSgcResult(sgcRes);
    setIsGenerating(false);
  }, [schema, operations, config]);

  const currentResult = generatorView === 'codegen' ? codegenResult : sgcResult;
  const hasErrors = currentResult?.error;
  const hasWarnings = currentResult?.warnings && currentResult.warnings.length > 0;
  const hasDiagnostics = hasErrors || hasWarnings;

  // Handle preset changes - applies preset defaults to config
  // Handle preset changes - just update the preset name, core applies defaults
  const handlePresetChange = useCallback((newPreset: Preset) => {
    const newConfig: CodegenConfig = {
      ...config,
      preset: newPreset,
    };
    setConfig(newConfig);
    setConfigJson(JSON.stringify(newConfig, null, 2));
  }, [config]);

  return (
    <div className="flex flex-col h-full">
      {/* Toolbar */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-gray-800 bg-gray-900">
        <div className="flex items-center gap-4">
          <h1 className="text-lg font-semibold">Playground</h1>
          {isGenerating ? (
            <span className="text-xs text-gray-400 flex items-center gap-2">
              <svg
                className="animate-spin h-3 w-3"
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
              >
                <circle
                  className="opacity-25"
                  cx="12"
                  cy="12"
                  r="10"
                  stroke="currentColor"
                  strokeWidth="4"
                />
                <path
                  className="opacity-75"
                  fill="currentColor"
                  d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                />
              </svg>
              Generating...
            </span>
          ) : codegenResult && sgcResult ? (
            <span className="text-xs text-gray-400">
              graphql-codegen: <span className={codegenResult.error ? 'text-red-400' : 'text-blue-400'}>{codegenResult.timeMs.toFixed(1)}ms</span>
              {' · '}SGC: <span className={sgcResult.error ? 'text-red-400' : 'text-green-400'}>{sgcResult.timeMs.toFixed(1)}ms</span>
              {!codegenResult.error && !sgcResult.error && sgcResult.timeMs < codegenResult.timeMs && (
                <span className="text-green-400 ml-1">
                  ({(codegenResult.timeMs / sgcResult.timeMs).toFixed(1)}x faster)
                </span>
              )}
            </span>
          ) : null}
        </div>
        <div className="flex items-center gap-3">
          {/* Preset dropdown */}
          <div className="flex items-center gap-2">
            <label htmlFor="preset" className="text-xs text-gray-400">Preset:</label>
            <select
              id="preset"
              value={config.preset || 'graphql-codegen'}
              onChange={(e) => handlePresetChange(e.target.value as Preset)}
              className="px-2 py-1 text-xs font-medium text-gray-300 bg-gray-800 hover:bg-gray-700 rounded border border-gray-700 focus:border-green-500 focus:outline-none transition-colors cursor-pointer"
            >
              <option value="sgc">SGC (Optimized)</option>
              <option value="graphql-codegen">graphql-codegen (Compatible)</option>
            </select>
          </div>
          <button
            onClick={handleShare}
          className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-gray-300 hover:text-white bg-gray-800 hover:bg-gray-700 rounded transition-colors"
        >
          <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.368 2.684 3 3 0 00-5.368-2.684z" />
          </svg>
          {shareMessage || 'Share'}
          </button>
        </div>
      </div>

      {/* Main content */}
      <div className="flex flex-1 overflow-hidden">
        {/* Left panel - Input */}
        <div className="w-1/2 flex flex-col border-r border-gray-800">
          {/* Tabs */}
          <div className="flex border-b border-gray-800">
            <button
              onClick={() => setInputTab('schema')}
              className={`px-4 py-2 text-sm font-medium transition-colors ${
                inputTab === 'schema'
                  ? 'text-white border-b-2 border-green-500 bg-gray-900/50'
                  : 'text-gray-400 hover:text-white'
              }`}
            >
              Schema
            </button>
            <button
              onClick={() => setInputTab('operations')}
              className={`px-4 py-2 text-sm font-medium transition-colors ${
                inputTab === 'operations'
                  ? 'text-white border-b-2 border-green-500 bg-gray-900/50'
                  : 'text-gray-400 hover:text-white'
              }`}
            >
              Operations
            </button>
            <button
              onClick={() => setInputTab('config')}
              className={`px-4 py-2 text-sm font-medium transition-colors ${
                inputTab === 'config'
                  ? 'text-white border-b-2 border-green-500 bg-gray-900/50'
                  : 'text-gray-400 hover:text-white'
              }`}
            >
              Config
            </button>
          </div>

          {/* Editor / Config */}
          <div className="flex-1 overflow-hidden">
            {!isMounted ? (
              <div className="p-4 text-gray-500 font-mono text-sm">Loading editor...</div>
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
                    onChange={handleConfigJsonChange}
                    theme="sgc-dark"
                    options={configEditorOptions}
                    beforeMount={(monaco) => {
                      defineCustomTheme(monaco);
                      configureMonacoSchema(monaco);
                    }}
                    loading={<div className="p-4 text-gray-500">Loading editor...</div>}
                  />
                </div>
              </div>
            ) : inputTab === 'schema' ? (
              <Editor
                height="100%"
                defaultLanguage="graphql"
                value={schema}
                onChange={(value) => setSchema(value || '')}
                theme="sgc-dark"
                options={editorOptions}
                beforeMount={defineCustomTheme}
                loading={<div className="p-4 text-gray-500">Loading editor...</div>}
              />
            ) : (
              <Editor
                height="100%"
                defaultLanguage="graphql"
                value={operations}
                onChange={(value) => setOperations(value || '')}
                theme="sgc-dark"
                options={editorOptions}
                beforeMount={defineCustomTheme}
                loading={<div className="p-4 text-gray-500">Loading editor...</div>}
              />
            )}
          </div>
        </div>

        {/* Right panel - Output */}
        <div className="w-1/2 flex flex-col">
          {/* Generator selector and output tabs */}
          <div className="px-4 py-2 border-b border-gray-800 flex items-center justify-between">
            <div className="flex items-center gap-4">
              {/* Generator toggle */}
              <div className="flex items-center gap-2">
                <button
                  onClick={() => setGeneratorView('sgc')}
                  className={`px-3 py-1 text-xs font-medium rounded transition-colors ${
                    generatorView === 'sgc'
                      ? 'bg-green-600 text-white'
                      : 'bg-gray-800 text-gray-400 hover:text-white'
                  }`}
                >
                  SGC
                </button>
                <button
                  onClick={() => setGeneratorView('codegen')}
                  className={`px-3 py-1 text-xs font-medium rounded transition-colors ${
                    generatorView === 'codegen'
                      ? 'bg-blue-600 text-white'
                      : 'bg-gray-800 text-gray-400 hover:text-white'
                  }`}
                >
                  graphql-codegen
                </button>
              </div>

              {/* Output/Diagnostics tabs */}
              <div className="flex items-center gap-1 ml-4 border-l border-gray-700 pl-4">
                <button
                  onClick={() => setOutputTab('output')}
                  className={`px-2 py-1 text-xs transition-colors ${
                    outputTab === 'output'
                      ? 'text-white'
                      : 'text-gray-500 hover:text-white'
                  }`}
                >
                  Output
                </button>
                <button
                  onClick={() => setOutputTab('diagnostics')}
                  className={`px-2 py-1 text-xs transition-colors flex items-center gap-1 ${
                    outputTab === 'diagnostics'
                      ? 'text-white'
                      : 'text-gray-500 hover:text-white'
                  }`}
                >
                  Diagnostics
                  {hasDiagnostics && (
                    <span className={`w-2 h-2 rounded-full ${hasErrors ? 'bg-red-500' : 'bg-yellow-500'}`} />
                  )}
                </button>
              </div>
            </div>
            <button
              onClick={() => currentResult && navigator.clipboard.writeText(currentResult.output)}
              className="text-xs text-gray-500 hover:text-white transition-colors"
            >
              Copy
            </button>
          </div>

          {/* Content */}
          <div className="flex-1 overflow-hidden">
            {outputTab === 'diagnostics' ? (
              <div className="p-4 font-mono text-sm space-y-4 overflow-auto h-full">
                {hasErrors && (
                  <div>
                    <div className="flex items-center gap-2 text-red-400 font-semibold mb-2">
                      <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                        <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
                      </svg>
                      Error
                    </div>
                    <pre className="text-red-300 whitespace-pre-wrap bg-red-950/30 rounded p-3 border border-red-900/50">
                      {currentResult?.error}
                    </pre>
                  </div>
                )}

                {hasWarnings && (
                  <div>
                    <div className="flex items-center gap-2 text-yellow-400 font-semibold mb-2">
                      <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                        <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
                      </svg>
                      Warnings ({currentResult?.warnings.length})
                    </div>
                    <div className="space-y-2">
                      {currentResult?.warnings.map((warning, i) => (
                        <pre key={i} className="text-yellow-300 whitespace-pre-wrap bg-yellow-950/30 rounded p-3 border border-yellow-900/50">
                          {warning}
                        </pre>
                      ))}
                    </div>
                  </div>
                )}

                {!hasDiagnostics && (
                  <div className="text-green-400 flex items-center gap-2">
                    <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                      <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
                    </svg>
                    No errors or warnings
                  </div>
                )}
              </div>
            ) : currentResult?.error ? (
              <div className="p-4 text-red-400 font-mono text-sm overflow-auto h-full">
                <div className="font-semibold mb-2">Error:</div>
                <pre className="whitespace-pre-wrap">{currentResult.error}</pre>
              </div>
            ) : !isMounted ? (
              <div className="p-4 text-gray-500 font-mono text-sm">Loading editor...</div>
            ) : (
              <Editor
                height="100%"
                defaultLanguage="typescript"
                value={currentResult?.output || '// Click Generate to see output...'}
                theme="sgc-dark"
                options={readonlyEditorOptions}
                beforeMount={defineCustomTheme}
                loading={<div className="p-4 text-gray-500">Loading editor...</div>}
              />
            )}
          </div>
        </div>
      </div>

      {/* Footer info */}
      <div className="px-4 py-2 border-t border-gray-800 text-xs text-gray-500 flex items-center justify-between">
        <span>
          Compare graphql-codegen (JS) and SGC (Rust/WASM) output side-by-side.
        </span>
        <span>
          Both generators run entirely in your browser.
        </span>
      </div>
    </div>
  );
}
