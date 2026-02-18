import type { CodegenConfig } from './types'

export const DEFAULT_SCHEMA = `interface Node {
  id: ID!
}

type Query {
  node(id: ID!): Node
  nodes(id: [ID!]!): [Node]
}

type Review {
  id: ID!
}

type Subscription {
  reviewCreated: Review
}
`

export const DEFAULT_OPERATIONS = `subscription NewReviewCreated {
  reviewCreated {
    rating
    commentary
  }
}`

export const defaultConfig: CodegenConfig = {
    outputs: {
        'types.ts': {
            generators: ['schema-types', 'operation-types'],
        },
    },
}
