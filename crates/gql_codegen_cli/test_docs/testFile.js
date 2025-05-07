const query = graphql`
    query TestQuery($id: ID!) {
        person(id: $id) {
            id
        }
    }
`

const query2 = graphql`
    {
        me {
            id
        }
    }

`
