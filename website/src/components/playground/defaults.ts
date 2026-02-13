import type { CodegenConfig } from './types'

export const DEFAULT_SCHEMA = `scalar DateTime

interface Node {
  id: ID!
}

type Post implements Node {
  id: ID!
  title: String!
  body: String!
  published: Boolean!
  author: User!
  createdAt: DateTime!
}

type Query {
  node(id: ID!): Node
  nodes(ids: [ID!]!): [Node]!
  user(id: ID!): User
  users: [User!]!
}

enum Role {
    ADMIN
    USER
}

type User implements Node {
  id: ID!
  name: String!
  email: String!
  role: Role!
  posts: [Post!]!
  createdAt: DateTime!
}`

export const DEFAULT_OPERATIONS = `query GetUser($id: ID!) {
  node(id: $id) {
    __typename
    ... on User {
      id
      ... UserFields
    }
  }
}

query GetUsers {
  users {
    id
    name
  }
}

fragment UserFields on User {
  name
  email
  posts {
    id
    title
    published
  }
}

query Nodes($ids: [ID!]!) {
  nodes(ids: $ids) {
    ... on User {
      name
    }
    ... on Post {
      title
    }
  }
}`

export const defaultConfig: CodegenConfig = {
    generates: {
        'types.ts': {
            plugins: ['typescript', 'typescript-operations'],
        },
    },
}
