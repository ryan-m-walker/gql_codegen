// i do like TS config files with jiti, they're more powerful than JSON and typesafe too, overhead is low
const config: CodegenConfig = {
  // can be array or string
  // paths support glob patterns
  // urls support adding headers somehow (needed for remote server auth) maybe method too
  // supports path to introspection json
  schema: ['./schema.graphql'],
  // same, glob to docs this is good
  documents: 'src/**/*.tsx',

  // idk but i like "outputs" better than "generates"
  outputs: {
    './src/generated/types.ts': {
      // i like generators vs plugins since we don't have a plugin system
      generators: [
        'typescript',
        'typescript-operations',
        {
          // don't really like the "type" name, maybe something else? "generator"?
          type: 'typed-document-node',
          config: {
             // plugin specific or scoped config seems like a good idea
          }
        }
      ],
      // i think shared global config is useful, probably only a small set that are actually sharable
      config: {
        scalars: {
          DateTime: 'string'
        }
      }
    }
  },
  // i like hooks, formatting and other is good
  hooks: {
    afterGenerate: ["biome format --write"]
  }
};

// config options that i like
const configOptions = {
    shared: {
        // arbitrary content appended to the top of the generated file
        prelude: 'import type { Something } from "somewhere";',
        // maybe like graphql codegen we can have these be broken down by type too?
        typeNamePrefix: 'I',
        // maybe like graphql codegen we can have these be broken down by type too?
        typeNameSuffix: 'Type',
        // i prefer interface but think it should maybe be ok to allow setting this? or maybe we should be more opinionated
        declarationKind: 'class',
        // default = true, but should be able to disable
        immutableTypes: true,
        // default = true, should be able to disable
        futureProofEnums: true,
        // default = true, should be able to disable
        futureProofUnions: true,
        // as-selected, ..., maybe this is only really relevant for typescript-operations? __typname should pretty much always be there for schema types and non null i imagine
        typeNamePolicy: 'always',
        // maybe we allow some kind of strict null, opitonal + null, undefined, etc
        nullType: 'null',
    }
}
