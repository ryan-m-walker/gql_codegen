// Example: Using the NAPI module directly from Node.js
// Run with: node example.mjs

import { createRequire } from 'module';
const require = createRequire(import.meta.url);

// Load the native module
const native = require('./index.js');

const config = {
  schema: "/Users/ryan/lindy/apps/web/src/schema.graphql",
  documents: ["/Users/ryan/lindy/apps/web/src/**/*.tsx"],
  generates: {
    "./__generated__/types.ts": {
      config: { graphqlTag: "graphql" },
      plugins: ["typescript", "typescript-operations"]
    }
  },
  baseDir: "/Users/ryan/Documents/coding/gql_codegen/packages/cli"
};

console.log('=== NAPI Direct Call ===');
console.time('NAPI generate');

const result = native.generate({
  configJson: JSON.stringify(config),
  noCache: true,
  timing: true,
});

console.timeEnd('NAPI generate');

console.log('Fresh:', result.fresh);
console.log('Files:', result.files.length);

if (result.files.length > 0) {
  console.log('First file:', result.files[0].path, `(${result.files[0].content.length} bytes)`);
}
