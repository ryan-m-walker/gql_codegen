
export const foo = observer(function RailsTesting() {
    const foo = useFragment(
        graphql`
            fragment Test on Identity {
                id
            }
        `,
    )
})
