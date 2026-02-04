Main design goals of the project:

- As fast as possible: as zero-copy as possible, caching from the start, parallelism
- Great DX: useful logs, flexible and configurable
- Fault tolerant: still works when other files fail, or fails gracefully

Best practices:

- Prefer smaller focused files
- Prefer streaming to `write!` and `writeln!` when possible vs allocating temporary Strings or Vecs

Tips:

- Prefer `pnpm` over `npm` for package management
