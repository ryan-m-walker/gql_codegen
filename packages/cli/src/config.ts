/**
 * Config loading utilities
 *
 * Handles loading configuration from JSON or TypeScript files.
 * TypeScript configs are transpiled on-the-fly using esbuild.
 */

import { build } from 'esbuild';
import { existsSync, readFileSync, unlinkSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { tmpdir } from 'node:os';
import { randomBytes } from 'node:crypto';

import type { CodegenConfig } from './types.js';

const CONFIG_FILES = [
  'codegen.config.ts',
  'codegen.config.js',
  'codegen.config.mjs',
  'codegen.json',
];

export interface LoadConfigResult {
  config: CodegenConfig;
  configPath: string;
}

/**
 * Find and load the codegen configuration file.
 *
 * Searches for config files in order of preference:
 * 1. codegen.config.ts
 * 2. codegen.config.js
 * 3. codegen.config.mjs
 * 4. codegen.json
 */
export async function loadConfig(
  configPath?: string,
  cwd: string = process.cwd()
): Promise<LoadConfigResult> {
  const resolvedPath = configPath
    ? resolve(cwd, configPath)
    : findConfigFile(cwd);

  if (!resolvedPath) {
    throw new Error(
      `No config file found. Create one of: ${CONFIG_FILES.join(', ')}`
    );
  }

  if (!existsSync(resolvedPath)) {
    throw new Error(`Config file not found: ${resolvedPath}`);
  }

  const config = await loadConfigFile(resolvedPath);
  return { config, configPath: resolvedPath };
}

function findConfigFile(cwd: string): string | null {
  for (const filename of CONFIG_FILES) {
    const fullPath = join(cwd, filename);
    if (existsSync(fullPath)) {
      return fullPath;
    }
  }
  return null;
}

async function loadConfigFile(configPath: string): Promise<CodegenConfig> {
  const ext = configPath.split('.').pop()?.toLowerCase();

  if (ext === 'json') {
    return loadJsonConfig(configPath);
  }

  if (ext === 'ts') {
    return loadTypeScriptConfig(configPath);
  }

  if (ext === 'js' || ext === 'mjs') {
    return loadJavaScriptConfig(configPath);
  }

  throw new Error(`Unsupported config file extension: .${ext}`);
}

function loadJsonConfig(configPath: string): CodegenConfig {
  const content = readFileSync(configPath, 'utf-8');
  try {
    return JSON.parse(content) as CodegenConfig;
  } catch (err) {
    throw new Error(`Failed to parse JSON config: ${configPath}`);
  }
}

async function loadJavaScriptConfig(
  configPath: string
): Promise<CodegenConfig> {
  const url = pathToFileURL(configPath).href;
  const module = await import(url);
  return module.default ?? module;
}

async function loadTypeScriptConfig(
  configPath: string
): Promise<CodegenConfig> {
  // Transpile TypeScript to JavaScript using esbuild
  const result = await build({
    entryPoints: [configPath],
    bundle: true,
    platform: 'node',
    format: 'esm',
    write: false,
    packages: 'external',
  });

  const code = result.outputFiles?.[0]?.text;
  if (!code) {
    throw new Error(`Failed to transpile TypeScript config: ${configPath}`);
  }

  // Write to a temp file and import it
  const tempFile = join(
    tmpdir(),
    `sgc-config-${randomBytes(8).toString('hex')}.mjs`
  );

  try {
    writeFileSync(tempFile, code);
    const url = pathToFileURL(tempFile).href;
    const module = await import(url);
    return module.default ?? module;
  } finally {
    // Clean up temp file
    try {
      unlinkSync(tempFile);
    } catch {
      // Ignore cleanup errors
    }
  }
}

/**
 * Convert config to JSON for passing to the Rust binary.
 */
export function configToJson(config: CodegenConfig): string {
  return JSON.stringify(config, null, 2);
}

/**
 * Resolve all relative paths in the config to absolute paths.
 * This is needed when writing a temp JSON config from a TS config.
 */
export function resolveConfigPaths(
  config: CodegenConfig,
  baseDir: string
): CodegenConfig {
  const resolvePath = (p: string) => (p.startsWith('/') ? p : resolve(baseDir, p));
  const resolvePaths = (paths: string | string[]): string | string[] => {
    if (Array.isArray(paths)) {
      return paths.map(resolvePath);
    }
    return resolvePath(paths);
  };

  const resolved: CodegenConfig = {
    ...config,
    schema: resolvePaths(config.schema),
    documents: resolvePaths(config.documents),
    generates: {},
    // Set baseDir so the Rust CLI knows where the original config was
    baseDir: baseDir,
  };

  // Resolve output paths
  for (const [outputPath, outputConfig] of Object.entries(config.generates)) {
    const resolvedOutputPath = resolvePath(outputPath);
    resolved.generates[resolvedOutputPath] = outputConfig;
  }

  return resolved;
}
