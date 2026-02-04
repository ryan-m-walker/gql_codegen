// Polyfill for Node.js 'module' built-in
// Provides stub createRequire that throws at runtime (we import plugins directly)

export function createRequire(_url: string | URL): NodeRequire {
  const require = function (_id: string): never {
    throw new Error(
      'Dynamic require is not supported in browser. Import modules directly.'
    );
  } as NodeRequire;

  require.resolve = function (_id: string): never {
    throw new Error('require.resolve is not supported in browser.');
  } as any;

  require.cache = {};
  require.extensions = {};
  require.main = undefined;

  return require;
}

export const builtinModules: string[] = [];
export const Module = { createRequire, builtinModules };
export default Module;
