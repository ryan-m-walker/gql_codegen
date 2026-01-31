// Test file for extracting GraphQL from TypeScript
import { gql } from '@apollo/client';

const GET_USER = gql`
  query GetUserFromTsx($id: ID!) {
    user(id: $id) {
      id
      name
    }
  }
`;

// Magic comment style
const GET_POSTS = /* GraphQL */ `
  query GetPostsFromTsx {
    posts {
      id
      title
    }
  }
`;

// This should NOT be extracted (not tagged)
const NOT_GRAPHQL = `
  query FakeQuery {
    notReal
  }
`;

export function UserProfile({ id }: { id: string }) {
  // Component code...
  return null;
}
