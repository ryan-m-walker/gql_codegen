# SGC - Speedy GraphQL Codegen

![Logo](images/graph_crab.png)

## Design Goals

- As fast as possible: as zero-copy as possible, caching from the start, parallelism
- Sensible defaults - default settings should follow modern best practices and generate performant generated types (avoiding slow TypeScript type checking patterns)
- Great DX: useful logs, flexible and configurable, warnings about conflicting configurations, etc...
- Great "AIX" (AI Experience): the tool should make AI agents faster and more effective in the same way it helps developers. clear actionable messages, fast quick feedback loop, reseliant and recoverable (collect all errors at once rather than only showing one at a time)
- Deterministic: reproducible builds, no randomness
- Drop in replacement for existing GraphQL codegen tools - migrating to SGC should be as easy as replacing `graphql-codegen` with `sgc --preset graphql-codegen`

