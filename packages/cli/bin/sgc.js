#!/usr/bin/env node

/**
 * SGC CLI Entry Point
 *
 * This script:
 * 1. Loads the config (JSON or TypeScript)
 * 2. Uses @sgc/core native module (fast path)
 * 3. Falls back to binary spawn if native unavailable
 */

import { existsSync, writeFileSync, unlinkSync, mkdtempSync, mkdirSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { tmpdir } from 'node:os';
import { parseArgs } from 'node:util';

import { loadConfig, configToJson, resolveConfigPaths } from '../dist/config.js';
import { runBinary } from '../dist/binary.js';

// Try to load native module
let nativeCore = null;
try {
  nativeCore = await import('@sgc/core');
  if (!nativeCore.isNativeAvailable()) {
    nativeCore = null;
  }
} catch {
  nativeCore = null;
}

const knownOptions = {
  config: { type: 'string', short: 'c' },
  watch: { type: 'boolean', short: 'w' },
  check: { type: 'boolean' },
  stdout: { type: 'boolean' },
  'clean-cache': { type: 'boolean' },
  'no-cache': { type: 'boolean' },
  verbose: { type: 'boolean', short: 'v' },
  quiet: { type: 'boolean', short: 'q' },
  timing: { type: 'boolean', short: 't' },
  help: { type: 'boolean', short: 'h' },
  version: { type: 'boolean', short: 'V' },
};

const { values: args, positionals } = parseArgs({
  options: knownOptions,
  allowPositionals: true,
  strict: false,
});

// Warn on unknown flags
const knownLongFlags = new Set(Object.keys(knownOptions));
const knownShortFlags = new Set(
  Object.values(knownOptions).map(opt => opt.short).filter(Boolean)
);

for (const arg of process.argv.slice(2)) {
  if (arg.startsWith('--')) {
    const flag = arg.split('=')[0].slice(2);
    if (!knownLongFlags.has(flag)) {
      console.warn(`Warning: Unknown option '--${flag}'`);
    }
  } else if (arg.startsWith('-') && arg.length > 1 && arg[1] !== '-') {
    const flag = arg[1];
    if (!knownShortFlags.has(flag)) {
      console.warn(`Warning: Unknown option '-${flag}'`);
    }
  }
}

async function main() {
  // Handle --help
  if (args.help) {
    console.log(`
SGC - Speedy GraphQL Codegen

Usage: sgc [options]

Options:
  -c, --config <path>  Path to config file (default: auto-detect)
  -w, --watch          Watch mode - regenerate on file changes
      --check          Check mode - validate without writing files
      --stdout         Print generated output to stdout instead of writing
      --clean-cache    Clear the cache directory and exit
      --no-cache       Disable caching (always regenerate)
  -v, --verbose        Verbose output
  -q, --quiet          Suppress output (only show errors)
  -t, --timing         Show timing information for performance debugging
  -h, --help           Show this help message
  -V, --version        Show version

Config files (in order of preference):
  codegen.config.ts    TypeScript config with full type safety
  codegen.config.js    JavaScript config
  codegen.config.mjs   ES Module config
  codegen.json         JSON config

Examples:
  sgc                          Run with auto-detected config
  sgc -c custom.config.ts      Run with specific config
  sgc --watch                  Watch mode
  sgc --check                  Validate without writing
`);
    process.exit(0);
  }

  // Handle --version
  if (args.version) {
    const mode = nativeCore ? 'native' : 'binary';
    console.log(`sgc 0.1.0 (${mode})`);
    process.exit(0);
  }

  // Warn on unexpected positional arguments
  if (positionals.length > 0) {
    console.warn(`Warning: Unexpected argument(s): ${positionals.join(', ')}`);
  }

  try {
    const totalStart = performance.now();
    const timing = (label, ms, extra = '') => {
      if (args.timing) {
        const extraStr = extra ? ` (${extra})` : '';
        console.error(`[timing] ${label}: ${ms.toFixed(1)}ms${extraStr}`);
      }
    };

    // Load config
    let t0 = performance.now();
    const { config, configPath } = await loadConfig(args.config);
    timing('Config load', performance.now() - t0);

    // Resolve paths relative to config file
    t0 = performance.now();
    const resolvedConfig = resolveConfigPaths(config, dirname(configPath));
    timing('Config resolve', performance.now() - t0);

    // Handle --clean-cache
    if (args['clean-cache']) {
      if (nativeCore) {
        const cleared = nativeCore.clearCache(resolvedConfig.baseDir || dirname(configPath));
        console.log(cleared ? '✓ Cache cleared' : '✓ Cache already clean');
      } else {
        // Fall back to binary
        await runBinaryMode(resolvedConfig, configPath, ['--clean-cache']);
      }
      process.exit(0);
    }

    // Use native module if available
    if (nativeCore) {
      await runNativeMode(resolvedConfig, timing);
    } else {
      await runBinaryMode(resolvedConfig, configPath);
    }

    timing('Total', performance.now() - totalStart);
    process.exit(0);
  } catch (err) {
    console.error('Error:', err instanceof Error ? err.message : err);
    if (args.verbose && err instanceof Error && err.stack) {
      console.error(err.stack);
    }
    process.exit(1);
  }
}

/**
 * Run using native NAPI module (fast path)
 */
async function runNativeMode(config, timing) {
  const t0 = performance.now();

  const result = nativeCore.generate({
    configJson: JSON.stringify(config),
    noCache: args['no-cache'],
    timing: args.timing,
  });

  timing('Native generate', performance.now() - t0);

  // Handle warnings
  for (const warning of result.warnings) {
    console.warn(`Warning: ${warning}`);
  }

  if (result.fresh) {
    if (!args.quiet) {
      console.log('✓ Nothing changed');
    }
    return; // Let main() print total timing
  }

  // Handle --stdout
  if (args.stdout) {
    for (const file of result.files) {
      console.log(`// ${file.path}`);
      console.log(file.content);
    }
    return;
  }

  // Handle --check
  if (args.check) {
    const count = result.files.length;
    const plural = count === 1 ? '' : 's';
    console.log(`Would generate ${count} file${plural}`);
    return;
  }

  // Write files
  const t1 = performance.now();
  for (const file of result.files) {
    const dir = dirname(file.path);
    if (!existsSync(dir)) {
      mkdirSync(dir, { recursive: true });
    }
    writeFileSync(file.path, file.content);
    if (!args.quiet) {
      console.log(`  ${file.path}`);
    }
  }
  timing('File write', performance.now() - t1);

  if (!args.quiet) {
    const count = result.files.length;
    const plural = count === 1 ? '' : 's';
    console.log(`✓ Generated ${count} file${plural}`);
  }
}

/**
 * Fallback: Run using binary spawn
 */
async function runBinaryMode(resolvedConfig, configPath, extraArgs = []) {
  // Write temp JSON config
  const tempDir = mkdtempSync(join(tmpdir(), 'sgc-'));
  const tempConfigPath = join(tempDir, 'config.json');
  writeFileSync(tempConfigPath, JSON.stringify(resolvedConfig));

  try {
    const binaryArgs = [...extraArgs];
    if (args.check) binaryArgs.push('--check');
    if (args.stdout) binaryArgs.push('--stdout');
    if (args['no-cache']) binaryArgs.push('--no-cache');
    if (args.verbose) binaryArgs.push('--verbose');
    if (args.quiet) binaryArgs.push('--quiet');
    if (args.timing) binaryArgs.push('--timing');

    const result = await runBinary({
      configPath: tempConfigPath,
      args: binaryArgs,
      stdio: 'inherit',
    });

    process.exit(result.exitCode);
  } finally {
    try {
      unlinkSync(tempConfigPath);
    } catch {
      // Ignore cleanup errors
    }
  }
}

main();
