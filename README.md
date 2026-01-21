# React Jitter

[![npm version](https://badge.fury.io/js/react-jitter.svg)](https://badge.fury.io/js/react-jitter)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

React Jitter is a developer tool to help you understand why your React components re-render. It pinpoints the exact hooks and values that have changed, so you can optimize your application's performance with precision.

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

Unnecessary re-renders are a common source of performance issues in React applications. When a component re-renders, it can be difficult to trace the root cause. React Profiler reports "Hook 7 changed" but not the name of hook or what exactly changed. It also does not provide anything useful for context changes and it just says "Context changed".

Existing tools like `why-did-you-render` require you to manually whitelist the hooks and components you want to track. This is impractical in large codebases where you want to monitor everything without tedious configuration. Others, like `react-scan`, are excellent for a high-level overview but does not report hook changes.

React Jitter solves these problems by instrumenting your code at build time with an SWC plugin. It tells you the exact hook that caused a re-render and shows you the previous and current values so you can see exactly what changed.

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
    enabled: true,
    // This function is called whenever a hook's value changes.
    onHookChange: (change) => {
      console.log("Hook changed:", change);
    },
    // This function is called after a component finishes rendering.
    onRender: (render) => {
      console.log("Component rendered:", render);
    }
  });
}
```

You can also change `enabled`, `onHookChange`, and `onRender` at runtime from your browser's developer console. This is useful for temporarily disabling logging or changing the callback behavior without a full page reload.

```js
// Disable React Jitter
window.reactJitter.enabled = false;

// Re-enable React Jitter
window.reactJitter.enabled = true;

// Change the onHookChange callback
window.reactJitter.onHookChange = (change) => {
  if (change.unstable) {
    console.warn("Unstable hook value:", change);
  }
};

// Change the onRender callback
window.reactJitter.onRender = (render) => {
  if (render.renderCount > 10) {
    console.warn("High render count:", render);
  }
}
```

Modern bundlers will tree-shake the `import` and the function call from your production build, so it will have zero performance impact.

### Advanced: Custom Comparator Selection

By default, React Jitter uses the `deepEqual` comparator to detect changes in hook values. However, you can customize which comparator is used on a per-hook basis using the `selectComparator` function. This is useful when dealing with circular data structures or when you need different comparison strategies for different hooks.

```js
// Set a custom comparator selector
window.reactJitter.selectComparator = (hookAddress) => {
  // Use circularDeepEqual for hooks that might return circular structures
  if (hookAddress.hook === 'useSelector' || hookAddress.hook === 'useReduxState') {
    return 'circularDeepEqual';
  }
  
  // Use deepEqual for everything else (default)
  return 'deepEqual';
};
```

The `hookAddress` parameter contains information about the hook:

```typescript
{
  hook: string;        // Hook name, e.g., "useState", "useContext"
  file: string;        // File path where the hook is called
  line: number;        // Line number
  offset: number;      // Column offset
  arguments?: string[]; // Hook arguments (if includeArguments is enabled)
}
```

**Available Comparators:**

- `deepEqual` (default): Fast deep equality check that handles most cases. Will throw an error if it encounters deeply nested or circular structures.
- `circularDeepEqual`: Slower but handles circular references safely. Use this when your hooks return data with circular dependencies or extremely deep nesting.

**When to Use `circularDeepEqual`:**

If you see an error like "Maximum call stack size exceeded. Please use the 'circularDeepEqual' comparator", you should configure `selectComparator` to return `'circularDeepEqual'` for the specific hook mentioned in the error message.

## API and Configuration

The `reactJitter` function accepts a configuration object with two callbacks: `onHookChange` and `onRender`.

- `onHookChange`: Called whenever a hook's value changes. It receives a `change` object with details about the hook, its location, and the value that changed.
- `onRender`: Called after a component finishes rendering. It receives a `render` object with metadata about the component's render cycle.

Here is an example of the `change` object from `onHookChange` when an unstable object is detected:

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

Here is an example of the `render` object from `onRender`:

```json
{
  "scopeId": "UserProfile-0",
  "renderCount": 5,
  "name": "UserProfile",
  "id": "UserProfile",
  "file": "/src/components/UserProfile.tsx",
  "line": 8,
  "offset": 1,
  "hookResults": {
    "4f23ef0": {
      "id": "user-123",
      "address": { "street": "123 Main St" }
    }
  }
}
```

This object provides the component's unique instance ID, its render count, location metadata, and a map of all hook results for the current render.

You can use the `includeArguments` option to identify which context has changed. When `includeArguments` is set to `true` in the SWC plugin configuration, the `onHookChange` callback will include the arguments passed to the hook. This is especially useful for `useContext`, as it allows you to see which context was used.

Here is an example of the `change` object when `includeArguments` is enabled:

```json
{
  "hook": "useContext",
  "arguments": ["UserContext"],
  "file": "/src/components/UserProfile.tsx",
  "line": 12,
  "offset": 18,
  "scope": {
    "name": "UserProfile",
    "file": "/src/components/UserProfile.tsx",
    "line": 8,
    "offset": 1
  },
  "unstable": false,
  "unstableKeys": [],
  "changedKeys": ["user"],
  "previousResult": {
    "user": { "name": "John" }
  },
  "currentResult": {
    "user": { "name": "Jane" }
  }
}
```

In this example, the `arguments` field shows that the `UserContext` was used, and the `changedKeys` field shows that the `user` property has changed.

### Detecting Unstable Hooks in Unit Tests

React Jitter can also be a powerful tool for improving code quality within your unit tests.

You can leverage this to write tests that fail if a hook becomes unstable, catching performance regressions early in the a testing setup where you might initialize React Jitter in a global setup file, you can easily override the `onHookChange` handler on a per-test basis.

```javascript
// Example of a Vitest/Jest test
it('should not have unstable hooks', () => {
  const unstableChanges = [];
  // Initialize React Jitter in a global setup file (e.g., setupTests.js)
  // Then, override the onHookChange handler for specific tests.
  window.reactJitter.onHookChange = (change) => {
    // You can ignore mocked hooks or handle them specifically
    if (change.unstable && !change.isMocked) {
      unstableChanges.push(change);
    }
  };

  render(<MyComponent />);

  // Assert that no unstable values were detected during the render
  expect(unstableChanges).toHaveLength(0);
});
```

The `onHookChange` callback's `change` object includes an `isMocked` boolean property. This is automatically set to `true` if React Jitter detects that the hook has been mocked (e.g., using `jest.fn()` or `vi.fn()`). This allows you to reliably identify and assert against unstable values in your test environment.

## How It Works

React Jitter is composed of two parts:

1.  **SWC Plugin**: A plugin for the [SWC compiler](httpss://swc.rs) that transforms your code to add instrumentation. It finds your React components and wraps your hook calls with a light monitoring function.
2.  **Runtime**: A small runtime library that is injected into your components. It keeps track of the values returned by your hooks and, when they change, it reports the differences.

One of the most common causes of unnecessary re-renders is "unstable" objects and functions that are re-created on every render. `react-jitter` helps you identify these issues by tracking when a value's reference changes and reporting it to you.

## Limitations

`react-jitter` detects React components using a set of heuristics. It looks for functions that either use hooks or return JSX, and whose names start with a capital letter. While this covers most common cases, it may not detect all components reliably, especially if you use less common patterns for defining components.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request if you have any ideas for improvement.

## License

This project is licensed under the MIT License.
