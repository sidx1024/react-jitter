# react-jitter

This repository contains an experimental SWC plugin implementing the build-time transformation described in the React-Jitter spec. The runtime implementation lives in `runtime.js` and tracks hook values at runtime.

Run the tests with `cargo test`.

## Using with Vite

Compile the transformer as a WASM plugin:

```bash
cargo build --release -p transformer --target wasm32-wasip1
```

Then register the generated plugin with `@vitejs/plugin-react-swc` in
`vite.config.js`:

```js
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react-swc'
import path from 'path'

export default defineConfig({
  plugins: [
    react({
      plugins: [
        [path.resolve(__dirname, 'target/wasm32-wasip1/release/transformer.wasm')]
      ]
    })
  ]
})
```

## Example App

Check out the `app` folder for a small Vite project using this plugin. To run
the demo and its tests:

```bash
cd app
npm install
npm test
```
