import type { CodegenConfig } from './types'

export const DEFAULT_SCHEMA = `scalar DateTime

type Post {
  id: ID!
  title: String!
  content: String
  author: User!
  published: Boolean!
}

type Query {
  user(id: ID!): User
  users: [User!]!
}

enum Role {
    ADMIN
    USER
}

type User {
  id: ID!
  name: String!
  email: String!
  posts: [Post!]!
  role: Role!
  createdAt: DateTime!
}`

export const DEFAULT_OPERATIONS = `query GetUser($id: ID!) {
  user(id: $id) {
    id
    name
    email
    posts {
      id
      title
      published
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
  id
  name
  email
}`

// Default config - preset handles defaults internally in core
export const defaultConfig: CodegenConfig = {
    preset: 'graphql-codegen',
    generates: {
        'types.ts': {
            plugins: ['typescript', 'typescript-operations'],
        },
    },
}
