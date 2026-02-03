import { platform, arch } from 'node:process';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);

// Types for the native binding
export interface GenerateOptions {
  /** JSON string of the config */
  configJson: string;
  /** Whether to skip caching */
  noCache?: boolean;
  /** Whether to enable timing output */
  timing?: boolean;
}

export interface GeneratedFile {
  path: string;
  content: string;
}

export interface GenerateResult {
  /** Whether generation was skipped (cache hit) */
  fresh: boolean;
  /** Generated files (only populated if not fresh) */
  files: GeneratedFile[];
  /** Warnings encountered during generation */
  warnings: string[];
}

interface NativeBinding {
  generate(options: GenerateOptions): GenerateResult;
  clearCache(baseDir: string): boolean;
}

// Platform-specific package mapping
const PLATFORMS: Record<string, { pkg: string; file: string }> = {
  'darwin-arm64': { pkg: '@sgc/core-darwin-arm64', file: 'sgc-core.darwin-arm64.node' },
  'darwin-x64': { pkg: '@sgc/core-darwin-x64', file: 'sgc-core.darwin-x64.node' },
  'linux-x64': { pkg: '@sgc/core-linux-x64-gnu', file: 'sgc-core.linux-x64-gnu.node' },
  'linux-arm64': { pkg: '@sgc/core-linux-arm64-gnu', file: 'sgc-core.linux-arm64-gnu.node' },
  'win32-x64': { pkg: '@sgc/core-win32-x64-msvc', file: 'sgc-core.win32-x64-msvc.node' },
};

let nativeBinding: NativeBinding | null = null;
let loadError: Error | null = null;

function loadNativeBinding(): NativeBinding | null {
  if (nativeBinding) return nativeBinding;
  if (loadError) return null;

  const platformKey = `${platform}-${arch}`;
  const platformInfo = PLATFORMS[platformKey];

  if (!platformInfo) {
    loadError = new Error(`Unsupported platform: ${platformKey}`);
    return null;
  }

  // Try loading from npm package first (production)
  try {
    nativeBinding = require(platformInfo.pkg) as NativeBinding;
    return nativeBinding;
  } catch {
    // Fall through to local file
  }

  // Try loading local .node file (development)
  try {
    nativeBinding = require(`../${platformInfo.file}`) as NativeBinding;
    return nativeBinding;
  } catch (e) {
    loadError = e instanceof Error ? e : new Error(String(e));
  }

  return nativeBinding;
}

/**
 * Generate TypeScript types from GraphQL schema and operations
 */
export function generate(options: GenerateOptions): GenerateResult {
  const binding = loadNativeBinding();
  if (!binding) {
    throw loadError ?? new Error('Failed to load native binding');
  }
  return binding.generate(options);
}

/**
 * Clear the cache directory
 * @param baseDir - Base directory containing .sgc cache
 * @returns Whether cache was cleared
 */
export function clearCache(baseDir: string): boolean {
  const binding = loadNativeBinding();
  if (!binding) {
    throw loadError ?? new Error('Failed to load native binding');
  }
  return binding.clearCache(baseDir);
}

/**
 * Check if native binding is available
 */
export function isNativeAvailable(): boolean {
  try {
    return loadNativeBinding() !== null;
  } catch {
    return false;
  }
}

/**
 * Get the load error if native binding failed
 */
export function getLoadError(): Error | null {
  loadNativeBinding();
  return loadError;
}
