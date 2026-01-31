import { defineConfig } from './src/types';

const config = defineConfig({
  schema: '../../crates/gql_codegen_cli/fixtures/schemas/schema.graphql',
  documents: ['../../crates/gql_codegen_cli/fixtures/documents/*.graphql'],
  generates: {
    './__generated__/types.ts': {
      plugins: ['typescript', 'typescript-operations'],
    },
  },
});

export default config;
