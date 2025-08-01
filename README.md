# React Jitter

[![npm version](https://badge.fury.io/js/react-jitter.svg)](https://badge.fury.io/js/react-jitter)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

React Jitter is a developer tool to help you understand why your React components re-render. It pinpoints the exact hooks and values that have changed, so you can optimize your application''s performance with precision.

## Table of Contents

- [Motivation](#motivation)
- [Getting Started](#getting-started)
  - [Installation](#installation)
  - [Usage](#usage)
- [API and Configuration](#api-and-configuration)
- [How It Works](#how-it-works)
- [Limitations](#limitations)
- [Contributing](#contributing)
- [License](#license)

## Motivation

Unnecessary re-renders are a common source of performance issues in React applications. When a component re-renders, it can be difficult to trace the root cause. Was it a state change? A new prop value? A context update? React''s developer tools can tell you that a component re-rendered, but not always why.

Existing tools like `why-did-you-render` require you to manually whitelist the hooks and components you want to track. This is impractical in large codebases where you want to monitor everything without tedious configuration. Others, like `react-scan`, are excellent for a high-level overview but don''t pinpoint the exact cause of a re-render. Even the official React Profiler can be frustrating when it reports anonymous hooks or contexts.

React Jitter solves these problems by instrumenting your code at build time with a lightweight SWC plugin. This approach has minimal performance overhead and requires no changes to your component code. It tells you the exact hook that caused a re-render and shows you the previous and current values so you can see exactly what changed.

- **Precise**: Know the exact hook and value that triggered a re-render.
- **Easy to Use**: No changes to your code are required.

## Getting Started

### Installation

```bash
npm install --save-dev react-jitter
```

### Usage

React Jitter is designed to be used **during development only**. You must ensure that both the SWC plugin and the runtime are disabled in your production builds to prevent instrumenting your production code.

#### Enabling the SWC Plugin (Development Only)

##### Next.js

In `next.config.js`, conditionally add the plugin and its configuration based on the environment:

```js
const isDevelopment = process.env.NODE_ENV === "development";

const nextConfig = {
  swcMinify: true,
  experimental: {
    swcPlugins: isDevelopment
      ? [
          [
            "react-jitter/plugin-swc",
            {
              // An array of hooks to ignore. For example, `["useSelector"]`.
              ignoreHooks: [],
              // An array of glob patterns for files to exclude from instrumentation.
              // By default, `**/node_modules/**` is excluded.
              exclude: [],
              // When true, the arguments of a hook will be captured as strings
              // and included in the `onHookChange` callback. This is disabled
              // by default because it can significantly increase the bundle size,
              // especially if hooks receive large objects or functions as arguments.
              includeArguments: false,
            },
          ],
        ]
      : [],
  },
};

module.exports = nextConfig;
```

##### `.swcrc`

Since `.swcrc` is a static JSON file, you cannot use logic within it. You should use separate build scripts or a JavaScript-based configuration file (e.g., `swc.config.js`) to apply different SWC configurations for your development and production environments.

#### Initializing the Runtime (Development Only)

Once the plugin is configured, you need to initialize the runtime in your application. In your main application file (e.g., `_app.tsx` in Next.js), ensure the call is only made in development:

```tsx
// In your app's entry file (e.g., _app.tsx)

import { reactJitter } from "react-jitter/runtime";

if (process.env.NODE_ENV === "development") {
  reactJitter({
    // This function will be called whenever a hook's value changes.
    onHookChange: (change) => {
      console.log("Hook changed:", change);
    },
  });
}
```

Modern bundlers will tree-shake the `import` and the function call from your production build, so it will have zero performance impact.

## API and Configuration

The `reactJitter` function accepts a single `onHookChange` callback that receives a `change` object whenever a hook''s value changes.

Here is an example of the `change` object when an unstable object is detected:

```json
{
  "hook": "useUser",
  "file": "/src/components/UserProfile.tsx",
  "line": 12,
  "offset": 18,
  "scope": {
    "name": "UserProfile",
    "file": "/src/components/UserProfile.tsx",
    "line": 8,
    "offset": 1
  },
  "unstable": true,
  "unstableKeys": ["address"],
  "changedKeys": [],
  "previousResult": {
    "id": "user-123",
    "address": { "street": "123 Main St" }
  },
  "currentResult": {
    "id": "user-123",
    "address": { "street": "123 Main St" }
  }
}
```

In this case, the `address` object was re-created, causing an unstable reference even though its contents are the same.

## How It Works

React Jitter is composed of two parts:

1.  **SWC Plugin**: A plugin for the [SWC compiler](httpss://swc.rs) that transforms your code to add instrumentation. It finds your React components and wraps your hook calls with a light monitoring function.
2.  **Runtime**: A small runtime library that is injected into your components. It keeps track of the values returned by your hooks and, when they change, it reports the differences.

One of the most common causes of unnecessary re-renders is "unstable" objects and functions that are re-created on every render. `react-jitter` helps you identify these issues by tracking when a value''s reference changes and reporting it to you.

## Limitations

`react-jitter` detects React components using a set of heuristics. It looks for functions that either use hooks or return JSX, and whose names start with a capital letter. While this covers most common cases, it may not detect all components reliably, especially if you use less common patterns for defining components.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request if you have any ideas for improvement.

## License

This project is licensed under the MIT License.
