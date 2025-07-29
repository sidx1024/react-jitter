import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react-swc'
import path from 'path'

// https://vite.dev/config/
const transformer = [path.resolve(__dirname, '../target/wasm32-wasip1/release/transformer.wasm')]

export default defineConfig(() => ({
  plugins: [
    react({
      plugins: process.env.VITEST ? [] : [transformer]
    })
  ],
  test: {
    environment: 'jsdom'
  }
}))
