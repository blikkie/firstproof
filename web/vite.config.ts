/// <reference types="vitest" />
import { defineConfig } from 'vitest/config';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
import { resolve } from 'path';
import { writeFileSync } from 'fs';

export default defineConfig({
  test: {
    environment: 'happy-dom',
    globals: true,
  },
  plugins: [
    wasm(),
    topLevelAwait(),
    {
      name: 'parity-logger',
      configureServer(server) {
        server.middlewares.use('/api/log-parity', (req, res) => {
          let body = '';
          req.on('data', chunk => body += chunk);
          req.on('end', () => {
            writeFileSync(resolve(__dirname, 'parity_report.json'), body);
            res.writeHead(200, { 'Content-Type': 'application/json' });
            res.end(JSON.stringify({ status: 'ok' }));
          });
        });
      }
    }
  ],
  resolve: {
    alias: {
      'firstproof-wasm': resolve(__dirname, '../firstproof-wasm/pkg')
    }
  },
  server: {
    port: 3000,
    open: true,
    fs: {
      allow: ['..']
    }
  }
});
