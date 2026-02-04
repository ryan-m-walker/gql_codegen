// @ts-check
import { defineConfig } from 'astro/config';
import { fileURLToPath } from 'url';
import path from 'path';

import react from '@astrojs/react';
import tailwindcss from '@tailwindcss/vite';
import { nodePolyfills } from 'vite-plugin-node-polyfills';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// https://astro.build/config
export default defineConfig({
  integrations: [react()],

  vite: {
    plugins: [
      tailwindcss(),
      nodePolyfills({
        // Don't include 'process' - we provide our own polyfill with cwd()
        include: ['path', 'crypto', 'stream', 'vm', 'buffer'],
        globals: {
          global: true,
          Buffer: true,
        },
      }),
    ],
    resolve: {
      alias: {
        // Provide our own polyfills for Node.js built-ins
        module: path.resolve(__dirname, 'src/lib/module-polyfill.ts'),
        process: path.resolve(__dirname, 'src/lib/process-polyfill.ts'),
      },
    },
  },
});