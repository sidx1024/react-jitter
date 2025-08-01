// src/index.ts
import React from "react";

// src/utils/getChanges.ts
import { deepEqual } from "fast-equals";
function getChanges(prev, next) {
  const changedKeys = [];
  const unstableKeys = [];
  const isObject = (v) => v !== null && typeof v === "object";
  const prevIsArr = Array.isArray(prev);
  const nextIsArr = Array.isArray(next);
  if (prevIsArr !== nextIsArr) {
    return {
      unstable: false,
      unstableKeys: [],
      changedKeys: ["*"]
    };
  }
  if (prevIsArr && nextIsArr) {
    if (prev.length !== next.length) {
      changedKeys.push("length");
    }
    const max = Math.max(prev.length, next.length);
    for (let i = 0; i < max; i++) {
      if (!deepEqual(prev[i], next[i]) || isObject(prev[i]) && isObject(next[i]) && prev[i] !== next[i]) {
        const key = String(i);
        changedKeys.push(key);
        if (isObject(prev[i]) || isObject(next[i])) {
          unstableKeys.push(key);
        }
      }
    }
  } else if (isObject(prev) && isObject(next)) {
    const allKeys = /* @__PURE__ */ new Set([...Object.keys(prev), ...Object.keys(next)]);
    for (const key of allKeys) {
      if (!deepEqual(prev[key], next[key]) || isObject(prev[key]) && isObject(next[key]) && prev[key] !== next[key]) {
        changedKeys.push(key);
        if (isObject(prev[key]) || isObject(next[key])) {
          unstableKeys.push(key);
        }
      }
    }
  } else {
    const unstable = isObject(prev) && isObject(next) && !deepEqual(prev, next);
    const changed = !deepEqual(prev, next);
    return {
      unstable,
      unstableKeys: [],
      changedKeys: changed ? [""] : []
    };
  }
  const isPlainObject = (v) => v !== null && typeof v === "object" && !Array.isArray(v);
  const unstableRoot = isPlainObject(prev) && isPlainObject(next) && prev !== next && deepEqual(prev, next);
  if (unstableRoot && changedKeys.length === 0) {
    changedKeys.push("");
    unstableKeys.push("");
  }
  return {
    unstable: unstableKeys.length > 0,
    unstableKeys,
    changedKeys
  };
}

// src/index.ts
var scopes = {};
var hookStack = /* @__PURE__ */ new Map();
var scopeCounter = {};
function useJitterScope(scope) {
  const scopeCount = React.useRef(getScopeCount(scope)).current;
  const scopeId = `${scope.id}-${scopeCount}`;
  if (!scopes[scopeId]) {
    scopes[scopeId] = {
      ...scope,
      hookResults: {}
    };
  }
  return {
    s: (id) => {
      hookStack.set(id, null);
    },
    e: (hookResult, hookEndEvent) => {
      const currentScope = scopes[scopeId];
      if (!currentScope) {
        return hookResult;
      }
      if (shouldReportChanges()) {
        const prevResult = currentScope.hookResults[hookEndEvent.id];
        const changes = compareChanges(prevResult, hookResult);
        if (changes) {
          const hookCall = {
            hook: hookEndEvent.hook,
            file: hookEndEvent.file,
            line: hookEndEvent.line,
            offset: hookEndEvent.offset,
            id: hookEndEvent.id,
            scope,
            ...changes,
            previousResult: prevResult,
            currentResult: hookResult
          };
          if (hookEndEvent.arguments) {
            hookCall.arguments = hookEndEvent.arguments;
          }
          callOnHookChange(hookCall);
        }
      }
      currentScope.hookResults[hookEndEvent.id] = hookResult;
      hookStack.delete(hookEndEvent.id);
      return hookResult;
    }
  };
}
function reactJitter(options) {
  var _a, _b, _c, _d;
  if (typeof window === "undefined") {
    return;
  }
  const windowGlobal = window;
  windowGlobal.reactJitter = {
    enabled: (_b = (_a = windowGlobal.reactJitter) == null ? void 0 : _a.enabled) != null ? _b : options.enabled,
    onHookChange: (_d = (_c = windowGlobal.reactJitter) == null ? void 0 : _c.onHookChange) != null ? _d : options.onHookChange,
    clear: () => {
      Object.keys(scopes).forEach((key) => delete scopes[key]);
    }
  };
}
function shouldReportChanges() {
  var _a;
  return typeof ((_a = window == null ? void 0 : window.reactJitter) == null ? void 0 : _a.onHookChange) === "function" && window.reactJitter.enabled;
}
function callOnHookChange(hookResult) {
  var _a;
  if (shouldReportChanges() && ((_a = window.reactJitter) == null ? void 0 : _a.onHookChange)) {
    window.reactJitter.onHookChange(hookResult);
  }
}
function getScopeCount(scope) {
  if (!scopeCounter[scope.id]) {
    scopeCounter[scope.id] = 0;
  }
  return scopeCounter[scope.id];
}
function compareChanges(prev, current) {
  if (prev !== "undefined" && prev !== current) {
    return getChanges(prev, current);
  }
  return null;
}
export {
  reactJitter,
  useJitterScope
};
