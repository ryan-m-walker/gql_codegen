import { GraphQLObjectType, GraphQLSchema, GraphQLString } from 'graphql'

const QueryType = new GraphQLObjectType({
    name: 'Query',
    fields: {
        world: { type: GraphQLString },
    },
})

export const schema = new GraphQLSchema({ query: QueryType })
