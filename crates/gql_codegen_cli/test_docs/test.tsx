
export const frag = graphql`
    fragment Test_Person2 on Person {
        name
    }
`

export const frag = graphql`
    fragment Test_Person on Person {
        ...Test_Person2 
    }
`


export const frag = graphql`
    query TestQuery {
        ...Test_Person
    }
`
