import { defineConfig } from './src/types'

const config = defineConfig({
    // schema: '../../crates/gql_codegen_cli/fixtures/schemas/schema.graphql',
    // documents: ['../../crates/gql_codegen_cli/fixtures/documents/*.graphql'],
    // schema: '../../../../../lindy/apps/web/src/schema.graphql',
    schema: '../../../../../lindy/apps/web/src/schema.graphql',
    documents: ['../../../../../../ryan/lindy/apps/web/src/**/*.tsx'],
    preset: 'graphql-codegen',
    generates: {
        './__generated__/types.ts': {
            config: {
                graphqlTag: 'graphql',
                futureProofEnums: true,
            },
            plugins: ['typescript', 'typescript-operations'],
            hooks: {
                afterAllFileWrite: ['echo "done"'],
            },
        },
    },
})

export default config
