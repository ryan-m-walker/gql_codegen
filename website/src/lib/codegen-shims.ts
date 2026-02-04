// Shim for @graphql-codegen/plugin-helpers resolve-external-module-and-fn
// We don't need dynamic plugin resolution because we import plugins directly

export function resolveExternalModuleAndFn(_module: string): never {
  throw new Error(
    'Dynamic module resolution is not supported in browser. Import plugins directly.'
  );
}

export function pickExportFromModule(
  _module: unknown,
  _identifier: string
): unknown {
  throw new Error(
    'Dynamic module resolution is not supported in browser. Import plugins directly.'
  );
}

export function pickExportFromModuleSync(
  _module: unknown,
  _identifier: string
): unknown {
  throw new Error(
    'Dynamic module resolution is not supported in browser. Import plugins directly.'
  );
}
