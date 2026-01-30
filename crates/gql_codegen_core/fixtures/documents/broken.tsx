// Test file with broken TypeScript syntax
// The extractor should still find the GraphQL

import { missing from  // syntax error!

const QUERY = gql`
  query StillExtractable {
    user {
      id
    }
  }
`;

const x = {{{ // more broken syntax

export const OTHER = graphql`
  query AlsoExtractable {
    posts {
      title
    }
  }
`;
