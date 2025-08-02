"use strict";
var __create = Object.create;
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __getProtoOf = Object.getPrototypeOf;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, { get: all[name], enumerable: true });
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toESM = (mod, isNodeMode, target) => (target = mod != null ? __create(__getProtoOf(mod)) : {}, __copyProps(
  // If the importer is in node compatibility mode or this is not an ESM
  // file that has been converted to a CommonJS file using a Babel-
  // compatible transform (i.e. "__esModule" has not been set), then set
  // "default" to the CommonJS "module.exports" for node compatibility.
  isNodeMode || !mod || !mod.__esModule ? __defProp(target, "default", { value: mod, enumerable: true }) : target,
  mod
));
var __toCommonJS = (mod) => __copyProps(__defProp({}, "__esModule", { value: true }), mod);

// src/index.ts
var index_exports = {};
__export(index_exports, {
  reactJitter: () => reactJitter,
  useJitterScope: () => useJitterScope
});
module.exports = __toCommonJS(index_exports);
var import_react = __toESM(require("react"));

// src/utils/getChanges.ts
var import_fast_equals = require("fast-equals");
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
      const deepEqItem = (0, import_fast_equals.deepEqual)(prev[i], next[i]);
      const refDiffItem = isObject(prev[i]) && isObject(next[i]) && prev[i] !== next[i];
      if (!deepEqItem || refDiffItem) {
        const key = String(i);
        changedKeys.push(key);
        if (refDiffItem && deepEqItem) {
          unstableKeys.push(key);
        }
      }
    }
  } else if (isObject(prev) && isObject(next)) {
    const allKeys = /* @__PURE__ */ new Set([...Object.keys(prev), ...Object.keys(next)]);
    for (const key of allKeys) {
      const deepEqProp = (0, import_fast_equals.deepEqual)(prev[key], next[key]);
      const refDiffProp = isObject(prev[key]) && isObject(next[key]) && prev[key] !== next[key];
      if (!deepEqProp || refDiffProp) {
        changedKeys.push(key);
        if (refDiffProp && deepEqProp) {
          unstableKeys.push(key);
        }
      }
    }
  } else {
    const deepEqRoot = (0, import_fast_equals.deepEqual)(prev, next);
    const refDiffRoot = isObject(prev) && isObject(next) && prev !== next;
    const unstable = refDiffRoot && deepEqRoot;
    const changed = !deepEqRoot || refDiffRoot;
    return {
      unstable,
      unstableKeys: [],
      changedKeys: changed ? [""] : []
    };
  }
  const isPlainObject = (v) => v !== null && typeof v === "object" && !Array.isArray(v);
  const unstableRoot = isPlainObject(prev) && isPlainObject(next) && prev !== next && (0, import_fast_equals.deepEqual)(prev, next);
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
  const scopeCount = import_react.default.useRef(getScopeCount(scope)).current;
  const scopeId = `${scope.id}-${scopeCount}`;
  if (!scopes[scopeId]) {
    scopes[scopeId] = {
      renderCount: 0,
      ...scope,
      hookResults: {}
    };
  }
  scopes[scopeId].renderCount++;
  const hooks = import_react.default.useRef(null);
  if (!hooks.current) {
    hooks.current = {
      s: (id) => {
        const hookId = `${scopeId}-${id}`;
        hookStack.set(hookId, null);
      },
      e: (hookResult, hookEndEvent) => {
        const currentScope = scopes[scopeId];
        if (!currentScope) {
          return hookResult;
        }
        const hookId = `${scopeId}-${hookEndEvent.id}`;
        if (shouldReportChanges()) {
          const prevResult = currentScope.hookResults[hookId];
          const changes = compareChanges(prevResult, hookResult);
          if (changes) {
            const hookCall = {
              hook: hookEndEvent.hook,
              file: hookEndEvent.file,
              line: hookEndEvent.line,
              offset: hookEndEvent.offset,
              id: hookEndEvent.id,
              scopeId,
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
        currentScope.hookResults[hookId] = hookResult;
        hookStack.delete(hookId);
        return hookResult;
      },
      re: (renderResult) => {
        return renderResult;
      }
    };
  }
  return hooks.current;
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
// Annotate the CommonJS export names for ESM import in node:
0 && (module.exports = {
  reactJitter,
  useJitterScope
});
