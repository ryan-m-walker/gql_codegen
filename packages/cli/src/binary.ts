/**
 * Rust binary invocation
 *
 * Handles locating and executing the native sgc binary.
 */

import { spawn, type SpawnOptions } from 'node:child_process';
import { existsSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));

/**
 * Platform-specific binary names
 */
function getBinaryName(): string {
  const platform = process.platform;
  const arch = process.arch;

  // Map Node.js platform/arch to Rust target names
  const platformMap: Record<string, string> = {
    darwin: 'apple-darwin',
    linux: 'unknown-linux-gnu',
    win32: 'pc-windows-msvc',
  };

  const archMap: Record<string, string> = {
    x64: 'x86_64',
    arm64: 'aarch64',
  };

  const rustPlatform = platformMap[platform];
  const rustArch = archMap[arch];

  if (!rustPlatform || !rustArch) {
    throw new Error(`Unsupported platform: ${platform}-${arch}`);
  }

  const ext = platform === 'win32' ? '.exe' : '';
  return `sgc-${rustArch}-${rustPlatform}${ext}`;
}

/**
 * Find the native binary path.
 * Looks in several locations:
 * 1. ./native/ directory (npm package)
 * 2. Local development build
 */
export function findBinary(): string {
  const binaryName = getBinaryName();

  // Check npm package location
  const npmPath = join(__dirname, '..', 'native', binaryName);
  if (existsSync(npmPath)) {
    return npmPath;
  }

  // Check for platform-agnostic binary name (development)
  const devPath = join(__dirname, '..', 'native', 'sgc');
  if (existsSync(devPath)) {
    return devPath;
  }

  // Check local cargo build (development)
  const cargoPath = join(__dirname, '..', '..', '..', 'target', 'release', 'gql-codegen');
  if (existsSync(cargoPath)) {
    return cargoPath;
  }

  const cargoDebugPath = join(__dirname, '..', '..', '..', 'target', 'debug', 'gql-codegen');
  if (existsSync(cargoDebugPath)) {
    return cargoDebugPath;
  }

  throw new Error(
    `Could not find sgc binary. Looked for:\n` +
    `  - ${npmPath}\n` +
    `  - ${devPath}\n` +
    `  - ${cargoPath}\n` +
    `  - ${cargoDebugPath}`
  );
}

export interface CliFlags {
  /** Check mode - validate without writing files */
  check?: boolean;
  /** Print generated output to stdout instead of writing files */
  stdout?: boolean;
  /** Disable caching (always regenerate) */
  noCache?: boolean;
  /** Clear the cache directory and exit */
  clean?: boolean;
  /** Verbose output */
  verbose?: boolean;
  /** Suppress output (only show errors) */
  quiet?: boolean;
}

export interface RunOptions {
  /** Working directory */
  cwd?: string;
  /** Path to JSON config file */
  configPath: string;
  /** CLI flags */
  flags?: CliFlags;
  /** Additional CLI arguments (raw) */
  args?: string[];
  /** Inherit stdio (show output) */
  stdio?: 'inherit' | 'pipe';
}

export interface RunResult {
  exitCode: number;
  stdout?: string;
  stderr?: string;
}

/**
 * Convert CliFlags to CLI arguments array.
 */
function flagsToArgs(flags: CliFlags): string[] {
  const args: string[] = [];
  if (flags.check) args.push('--check');
  if (flags.stdout) args.push('--stdout');
  if (flags.noCache) args.push('--no-cache');
  if (flags.clean) args.push('--clean');
  if (flags.verbose) args.push('--verbose');
  if (flags.quiet) args.push('--quiet');
  return args;
}

/**
 * Run the native sgc binary with the given arguments.
 */
export async function runBinary(options: RunOptions): Promise<RunResult> {
  const binary = findBinary();

  const flagArgs = options.flags ? flagsToArgs(options.flags) : [];
  const args = ['-c', options.configPath, ...flagArgs, ...(options.args ?? [])];

  const spawnOptions: SpawnOptions = {
    cwd: options.cwd ?? process.cwd(),
    stdio: options.stdio ?? 'inherit',
    env: process.env,
  };

  return new Promise((resolve, reject) => {
    const child = spawn(binary, args, spawnOptions);

    let stdout = '';
    let stderr = '';

    if (options.stdio === 'pipe') {
      child.stdout?.on('data', (data) => {
        stdout += data.toString();
      });
      child.stderr?.on('data', (data) => {
        stderr += data.toString();
      });
    }

    child.on('error', (err: Error) => {
      reject(new Error(`Failed to run sgc binary: ${err.message}`));
    });

    child.on('close', (code: number) => {
      resolve({
        exitCode: code ?? 1,
        stdout: options.stdio === 'pipe' ? stdout : undefined,
        stderr: options.stdio === 'pipe' ? stderr : undefined,
      });
    });
  });
}
