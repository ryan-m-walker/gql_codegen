import { generate, isNativeAvailable, getLoadError } from './dist/index.js';

console.log('Native available:', isNativeAvailable());

if (!isNativeAvailable()) {
  console.log('Load error:', getLoadError());
  process.exit(1);
}

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

console.log('\n=== Testing @sgc/core ===');
for (let i = 1; i <= 3; i++) {
  console.time(`Run ${i}`);
  const result = generate({
    configJson: JSON.stringify(config),
    timing: i === 1,
  });
  console.timeEnd(`Run ${i}`);
  console.log(`  Fresh: ${result.fresh}`);
}
