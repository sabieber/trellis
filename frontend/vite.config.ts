import { fileURLToPath, URL } from 'node:url'
import { createRequire } from 'node:module'

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import vueDevTools from 'vite-plugin-vue-devtools'
import tailwindcss from "@tailwindcss/vite";
import { VitePWA } from 'vite-plugin-pwa'

// Absolute path to the zxing-wasm reader binary. Aliased below so the barcode scanner
// can `import 'zxing-reader-wasm?url'` and have Vite emit it as a local hashed asset
// instead of zxing-wasm fetching it from a CDN at runtime. rolldown-vite can't resolve
// the package's `…/zxing_reader.wasm?url` exports subpath directly, hence the alias.
const zxingReaderWasm = createRequire(import.meta.url).resolve('zxing-wasm/reader/zxing_reader.wasm')

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    vueDevTools(),
    tailwindcss(),
    VitePWA({
      registerType: 'autoUpdate',
      injectRegister: 'auto',
      includeAssets: [
        'favicon.ico',
        'favicon-16x16.png',
        'favicon-32x32.png',
        'apple-touch-icon.png',
        'logo.svg',
      ],
      manifest: {
        name: 'trellis',
        short_name: 'trellis',
        id: '/',
        start_url: '/',
        scope: '/',
        display: 'standalone',
        theme_color: '#17150f',
        background_color: '#17150f',
        icons: [
          {
            src: '/android-chrome-192x192.png',
            sizes: '192x192',
            type: 'image/png',
          },
          {
            src: '/android-chrome-512x512.png',
            sizes: '512x512',
            type: 'image/png',
          },
          {
            src: '/android-chrome-512x512.png',
            sizes: '512x512',
            type: 'image/png',
            purpose: 'maskable',
          },
        ],
      },
    }),
  ],
  resolve: {
    alias: [
      { find: '@', replacement: fileURLToPath(new URL('./src', import.meta.url)) },
      { find: /^zxing-reader-wasm/, replacement: zxingReaderWasm },
    ],
  },
  // In dev, the frontend and backend run on separate ports. Proxy API calls to
  // the backend so they stay same-origin (matching production, where the
  // backend serves the built frontend) and no CORS is needed.
  server: {
    proxy: {
      '/api': 'http://localhost:5174',
    },
  },
})
