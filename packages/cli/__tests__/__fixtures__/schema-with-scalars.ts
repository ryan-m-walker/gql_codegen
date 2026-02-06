import {
    GraphQLObjectType,
    GraphQLScalarType,
    GraphQLSchema,
    GraphQLString,
} from 'graphql'

const DateTimeScalar = new GraphQLScalarType({
    name: 'DateTime',
    extensions: {
        codegenScalarType: 'Date | string',
    },
})

const JSONScalar = new GraphQLScalarType({
    name: 'JSON',
    extensions: {
        codegenScalarType: {
            input: 'Record<string, unknown>',
            output: 'unknown',
        },
    },
})

const QueryType = new GraphQLObjectType({
    name: 'Query',
    fields: {
        now: { type: DateTimeScalar },
        data: { type: JSONScalar },
        name: { type: GraphQLString },
    },
})

export default new GraphQLSchema({ query: QueryType })
