// Test with caching enabled
import { createRequire } from 'module';
const require = createRequire(import.meta.url);
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

console.log('=== NAPI with cache ===');
for (let i = 1; i <= 3; i++) {
  console.log(`\nRun ${i}:`);
  console.time('generate');
  const result = native.generate({
    configJson: JSON.stringify(config),
    noCache: false,  // Enable caching
    timing: true,
  });
  console.timeEnd('generate');
  console.log('Fresh:', result.fresh);
}
