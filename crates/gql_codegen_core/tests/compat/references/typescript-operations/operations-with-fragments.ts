export type UserFieldsFragment = { __typename?: 'User', id: string, name: string, email?: string | null };

export type PostFieldsFragment = { __typename?: 'Post', id: string, title: string, body: string };

export type GetUserWithFragmentsQueryVariables = Exact<{
  id: Scalars['ID']['input'];
}>;


export type GetUserWithFragmentsQuery = { __typename?: 'Query', user?: { __typename?: 'User', id: string, name: string, email?: string | null, posts: Array<{ __typename?: 'Post', id: string, title: string, body: string }> } | null };
