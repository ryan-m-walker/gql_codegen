#!/usr/bin/env node

/**
 * Generate reference output from @graphql-codegen for comparison testing.
 *
 * Reads JSON test case configs from cases/, runs them through @graphql-codegen/core,
 * and writes the output to references/.
 *
 * Usage: node generate-references.mjs
 */

import { readFileSync, writeFileSync, mkdirSync, readdirSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { buildSchema, parse } from 'graphql';
import { codegen } from '@graphql-codegen/core';
import * as typescriptPlugin from '@graphql-codegen/typescript';
import * as typescriptOperationsPlugin from '@graphql-codegen/typescript-operations';

const __dirname = dirname(fileURLToPath(import.meta.url));
const casesDir = join(__dirname, 'cases');
const referencesDir = join(__dirname, 'references');
const fixturesDir = join(__dirname, '..', 'fixtures');

const pluginMap = {
  'typescript': typescriptPlugin,
  'typescript-operations': typescriptOperationsPlugin,
};

async function generateReference(testCase) {
  // Load and merge schema files
  const schemaSdl = testCase.schema
    .map(f => readFileSync(join(fixturesDir, f), 'utf-8'))
    .join('\n');
  const schema = buildSchema(schemaSdl);

  // Load documents if any
  const documents = testCase.documents.map(f => ({
    document: parse(readFileSync(join(fixturesDir, f), 'utf-8')),
  }));

  // Build plugins array and pluginMap for codegen
  const plugins = testCase.plugins.map(name => ({ [name]: {} }));
  const resolvedPluginMap = {};
  for (const name of testCase.plugins) {
    if (!pluginMap[name]) throw new Error(`Unknown plugin: ${name}`);
    resolvedPluginMap[name] = pluginMap[name];
  }

  return codegen({
    schema: parse(readFileSync(join(fixturesDir, testCase.schema[0]), 'utf-8')),
    schemaAst: schema,
    documents,
    plugins,
    pluginMap: resolvedPluginMap,
    config: testCase.config || {},
    filename: 'output.ts',
  });
}

// ── Discovery and orchestration (this part is done) ─────────────────────────

function discoverCases() {
  const cases = [];
  for (const pluginDir of readdirSync(casesDir)) {
    const pluginPath = join(casesDir, pluginDir);
    const files = readdirSync(pluginPath).filter(f => f.endsWith('.json'));
    for (const file of files) {
      cases.push({ pluginDir, file, path: join(pluginPath, file) });
    }
  }
  return cases;
}

async function main() {
  const cases = discoverCases();
  console.log(`Found ${cases.length} test cases\n`);

  let passed = 0;
  let failed = 0;

  for (const { pluginDir, file, path } of cases) {
    const testCase = JSON.parse(readFileSync(path, 'utf-8'));
    const outDir = join(referencesDir, pluginDir);
    mkdirSync(outDir, { recursive: true });

    const outFile = join(outDir, file.replace('.json', '.ts'));
    const label = `${pluginDir}/${testCase.name}`;

    try {
      const output = await generateReference(testCase);
      writeFileSync(outFile, output, 'utf-8');
      console.log(`  ✓ ${label}`);
      passed++;
    } catch (err) {
      console.error(`  ✗ ${label}: ${err.message}`);
      failed++;
    }
  }

  console.log(`\nDone: ${passed} generated, ${failed} failed`);
  if (failed > 0) process.exit(1);
}

main();
