// Polyfill for Node.js 'process' built-in
// Only applies in browser environment

export function cwd(): string {
  return '/';
}

export function hrtime(time?: [number, number]): [number, number] {
  const now = typeof performance !== 'undefined' ? performance.now() : Date.now();
  const sec = Math.floor(now / 1000);
  const nano = Math.floor((now % 1000) * 1e6);
  if (time) {
    return [sec - time[0], nano - time[1]];
  }
  return [sec, nano];
}

export const env: Record<string, string | undefined> = {};
export const platform = 'browser';
export const version = '';
export const versions = {};
export const argv: string[] = [];
export const pid = 1;
export const nextTick = (fn: () => void) => setTimeout(fn, 0);

export const process = {
  cwd,
  hrtime,
  env,
  platform,
  version,
  versions,
  argv,
  pid,
  nextTick,
};

// Only set on globalThis in browser (not Node.js SSR)
if (typeof window !== 'undefined') {
  (window as any).process = (window as any).process || {};
  // Only set properties that don't exist
  const wp = (window as any).process;
  if (!wp.hrtime) wp.hrtime = hrtime;
  if (!wp.cwd) wp.cwd = cwd;
  if (!wp.env) wp.env = env;
  if (!wp.nextTick) wp.nextTick = nextTick;
}

export default process;
