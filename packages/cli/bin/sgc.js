#!/usr/bin/env node

/**
 * SGC CLI Entry Point
 *
 * This script:
 * 1. Loads the config (JSON or TypeScript)
 * 2. Writes a temp JSON config if needed
 * 3. Invokes the native Rust binary
 */

import { existsSync, writeFileSync, unlinkSync, mkdtempSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { tmpdir } from 'node:os';
import { parseArgs } from 'node:util';

import { loadConfig, configToJson, resolveConfigPaths } from '../dist/config.js';
import { runBinary } from '../dist/binary.js';

const knownOptions = {
  config: { type: 'string', short: 'c' },
  watch: { type: 'boolean', short: 'w' },
  check: { type: 'boolean' },
  stdout: { type: 'boolean' },
  'clean-cache': { type: 'boolean' },
  'no-cache': { type: 'boolean' },
  verbose: { type: 'boolean', short: 'v' },
  quiet: { type: 'boolean', short: 'q' },
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
    // TODO: Read from package.json
    console.log('sgc 0.1.0');
    process.exit(0);
  }

  // Warn on unexpected positional arguments
  if (positionals.length > 0) {
    console.warn(`Warning: Unexpected argument(s): ${positionals.join(', ')}`);
  }

  try {
    // Load config
    const { config, configPath } = await loadConfig(args.config);

    // Check if config is already JSON - we can pass it directly
    const isJsonConfig = configPath.endsWith('.json');

    let tempConfigPath = null;
    let finalConfigPath = configPath;

    if (!isJsonConfig) {
      // Write temp JSON config for the Rust binary
      // Resolve all relative paths to absolute so they work from any directory
      const resolvedConfig = resolveConfigPaths(config, dirname(configPath));
      const tempDir = mkdtempSync(join(tmpdir(), 'sgc-'));
      tempConfigPath = join(tempDir, 'config.json');
      writeFileSync(tempConfigPath, configToJson(resolvedConfig));
      finalConfigPath = tempConfigPath;
    }

    // Build args for the binary
    const binaryArgs = [];
    if (args.check) binaryArgs.push('--check');
    if (args.stdout) binaryArgs.push('--stdout');
    if (args.clean) binaryArgs.push('--clean-cache');
    if (args['no-cache']) binaryArgs.push('--no-cache');
    if (args.verbose) binaryArgs.push('--verbose');
    if (args.quiet) binaryArgs.push('--quiet');

    // Run the binary
    const result = await runBinary({
      configPath: finalConfigPath,
      args: binaryArgs,
      stdio: 'inherit',
    });

    // Cleanup temp file
    if (tempConfigPath) {
      try {
        unlinkSync(tempConfigPath);
      } catch {
        // Ignore cleanup errors
      }
    }

    process.exit(result.exitCode);
  } catch (err) {
    console.error('Error:', err instanceof Error ? err.message : err);
    process.exit(1);
  }
}

main();
