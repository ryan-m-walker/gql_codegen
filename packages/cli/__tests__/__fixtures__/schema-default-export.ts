import { GraphQLObjectType, GraphQLSchema, GraphQLString } from 'graphql'

const QueryType = new GraphQLObjectType({
    name: 'Query',
    fields: {
        hello: { type: GraphQLString },
    },
})

export default new GraphQLSchema({ query: QueryType })
