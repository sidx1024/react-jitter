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

// ../node_modules/fast-equals/dist/esm/index.mjs
var getOwnPropertyNames = Object.getOwnPropertyNames;
var getOwnPropertySymbols = Object.getOwnPropertySymbols;
var hasOwnProperty = Object.prototype.hasOwnProperty;
function combineComparators(comparatorA, comparatorB) {
  return function isEqual(a, b, state) {
    return comparatorA(a, b, state) && comparatorB(a, b, state);
  };
}
function createIsCircular(areItemsEqual) {
  return function isCircular(a, b, state) {
    if (!a || !b || typeof a !== "object" || typeof b !== "object") {
      return areItemsEqual(a, b, state);
    }
    var cache = state.cache;
    var cachedA = cache.get(a);
    var cachedB = cache.get(b);
    if (cachedA && cachedB) {
      return cachedA === b && cachedB === a;
    }
    cache.set(a, b);
    cache.set(b, a);
    var result = areItemsEqual(a, b, state);
    cache.delete(a);
    cache.delete(b);
    return result;
  };
}
function getStrictProperties(object) {
  return getOwnPropertyNames(object).concat(getOwnPropertySymbols(object));
}
var hasOwn = Object.hasOwn || function(object, property) {
  return hasOwnProperty.call(object, property);
};
function sameValueZeroEqual(a, b) {
  return a === b || !a && !b && a !== a && b !== b;
}
var PREACT_VNODE = "__v";
var PREACT_OWNER = "__o";
var REACT_OWNER = "_owner";
var getOwnPropertyDescriptor = Object.getOwnPropertyDescriptor;
var keys = Object.keys;
function areArraysEqual(a, b, state) {
  var index = a.length;
  if (b.length !== index) {
    return false;
  }
  while (index-- > 0) {
    if (!state.equals(a[index], b[index], index, index, a, b, state)) {
      return false;
    }
  }
  return true;
}
function areDatesEqual(a, b) {
  return sameValueZeroEqual(a.getTime(), b.getTime());
}
function areErrorsEqual(a, b) {
  return a.name === b.name && a.message === b.message && a.cause === b.cause && a.stack === b.stack;
}
function areFunctionsEqual(a, b) {
  return a === b;
}
function areMapsEqual(a, b, state) {
  var size = a.size;
  if (size !== b.size) {
    return false;
  }
  if (!size) {
    return true;
  }
  var matchedIndices = new Array(size);
  var aIterable = a.entries();
  var aResult;
  var bResult;
  var index = 0;
  while (aResult = aIterable.next()) {
    if (aResult.done) {
      break;
    }
    var bIterable = b.entries();
    var hasMatch = false;
    var matchIndex = 0;
    while (bResult = bIterable.next()) {
      if (bResult.done) {
        break;
      }
      if (matchedIndices[matchIndex]) {
        matchIndex++;
        continue;
      }
      var aEntry = aResult.value;
      var bEntry = bResult.value;
      if (state.equals(aEntry[0], bEntry[0], index, matchIndex, a, b, state) && state.equals(aEntry[1], bEntry[1], aEntry[0], bEntry[0], a, b, state)) {
        hasMatch = matchedIndices[matchIndex] = true;
        break;
      }
      matchIndex++;
    }
    if (!hasMatch) {
      return false;
    }
    index++;
  }
  return true;
}
var areNumbersEqual = sameValueZeroEqual;
function areObjectsEqual(a, b, state) {
  var properties = keys(a);
  var index = properties.length;
  if (keys(b).length !== index) {
    return false;
  }
  while (index-- > 0) {
    if (!isPropertyEqual(a, b, state, properties[index])) {
      return false;
    }
  }
  return true;
}
function areObjectsEqualStrict(a, b, state) {
  var properties = getStrictProperties(a);
  var index = properties.length;
  if (getStrictProperties(b).length !== index) {
    return false;
  }
  var property;
  var descriptorA;
  var descriptorB;
  while (index-- > 0) {
    property = properties[index];
    if (!isPropertyEqual(a, b, state, property)) {
      return false;
    }
    descriptorA = getOwnPropertyDescriptor(a, property);
    descriptorB = getOwnPropertyDescriptor(b, property);
    if ((descriptorA || descriptorB) && (!descriptorA || !descriptorB || descriptorA.configurable !== descriptorB.configurable || descriptorA.enumerable !== descriptorB.enumerable || descriptorA.writable !== descriptorB.writable)) {
      return false;
    }
  }
  return true;
}
function arePrimitiveWrappersEqual(a, b) {
  return sameValueZeroEqual(a.valueOf(), b.valueOf());
}
function areRegExpsEqual(a, b) {
  return a.source === b.source && a.flags === b.flags;
}
function areSetsEqual(a, b, state) {
  var size = a.size;
  if (size !== b.size) {
    return false;
  }
  if (!size) {
    return true;
  }
  var matchedIndices = new Array(size);
  var aIterable = a.values();
  var aResult;
  var bResult;
  while (aResult = aIterable.next()) {
    if (aResult.done) {
      break;
    }
    var bIterable = b.values();
    var hasMatch = false;
    var matchIndex = 0;
    while (bResult = bIterable.next()) {
      if (bResult.done) {
        break;
      }
      if (!matchedIndices[matchIndex] && state.equals(aResult.value, bResult.value, aResult.value, bResult.value, a, b, state)) {
        hasMatch = matchedIndices[matchIndex] = true;
        break;
      }
      matchIndex++;
    }
    if (!hasMatch) {
      return false;
    }
  }
  return true;
}
function areTypedArraysEqual(a, b) {
  var index = a.length;
  if (b.length !== index) {
    return false;
  }
  while (index-- > 0) {
    if (a[index] !== b[index]) {
      return false;
    }
  }
  return true;
}
function areUrlsEqual(a, b) {
  return a.hostname === b.hostname && a.pathname === b.pathname && a.protocol === b.protocol && a.port === b.port && a.hash === b.hash && a.username === b.username && a.password === b.password;
}
function isPropertyEqual(a, b, state, property) {
  if ((property === REACT_OWNER || property === PREACT_OWNER || property === PREACT_VNODE) && (a.$$typeof || b.$$typeof)) {
    return true;
  }
  return hasOwn(b, property) && state.equals(a[property], b[property], property, property, a, b, state);
}
var ARGUMENTS_TAG = "[object Arguments]";
var BOOLEAN_TAG = "[object Boolean]";
var DATE_TAG = "[object Date]";
var ERROR_TAG = "[object Error]";
var MAP_TAG = "[object Map]";
var NUMBER_TAG = "[object Number]";
var OBJECT_TAG = "[object Object]";
var REG_EXP_TAG = "[object RegExp]";
var SET_TAG = "[object Set]";
var STRING_TAG = "[object String]";
var URL_TAG = "[object URL]";
var isArray = Array.isArray;
var isTypedArray = typeof ArrayBuffer === "function" && ArrayBuffer.isView ? ArrayBuffer.isView : null;
var assign = Object.assign;
var getTag = Object.prototype.toString.call.bind(Object.prototype.toString);
function createEqualityComparator(_a) {
  var areArraysEqual2 = _a.areArraysEqual, areDatesEqual2 = _a.areDatesEqual, areErrorsEqual2 = _a.areErrorsEqual, areFunctionsEqual2 = _a.areFunctionsEqual, areMapsEqual2 = _a.areMapsEqual, areNumbersEqual2 = _a.areNumbersEqual, areObjectsEqual2 = _a.areObjectsEqual, arePrimitiveWrappersEqual2 = _a.arePrimitiveWrappersEqual, areRegExpsEqual2 = _a.areRegExpsEqual, areSetsEqual2 = _a.areSetsEqual, areTypedArraysEqual2 = _a.areTypedArraysEqual, areUrlsEqual2 = _a.areUrlsEqual;
  return function comparator(a, b, state) {
    if (a === b) {
      return true;
    }
    if (a == null || b == null) {
      return false;
    }
    var type = typeof a;
    if (type !== typeof b) {
      return false;
    }
    if (type !== "object") {
      if (type === "number") {
        return areNumbersEqual2(a, b, state);
      }
      if (type === "function") {
        return areFunctionsEqual2(a, b, state);
      }
      return false;
    }
    var constructor = a.constructor;
    if (constructor !== b.constructor) {
      return false;
    }
    if (constructor === Object) {
      return areObjectsEqual2(a, b, state);
    }
    if (isArray(a)) {
      return areArraysEqual2(a, b, state);
    }
    if (isTypedArray != null && isTypedArray(a)) {
      return areTypedArraysEqual2(a, b, state);
    }
    if (constructor === Date) {
      return areDatesEqual2(a, b, state);
    }
    if (constructor === RegExp) {
      return areRegExpsEqual2(a, b, state);
    }
    if (constructor === Map) {
      return areMapsEqual2(a, b, state);
    }
    if (constructor === Set) {
      return areSetsEqual2(a, b, state);
    }
    var tag = getTag(a);
    if (tag === DATE_TAG) {
      return areDatesEqual2(a, b, state);
    }
    if (tag === REG_EXP_TAG) {
      return areRegExpsEqual2(a, b, state);
    }
    if (tag === MAP_TAG) {
      return areMapsEqual2(a, b, state);
    }
    if (tag === SET_TAG) {
      return areSetsEqual2(a, b, state);
    }
    if (tag === OBJECT_TAG) {
      return typeof a.then !== "function" && typeof b.then !== "function" && areObjectsEqual2(a, b, state);
    }
    if (tag === URL_TAG) {
      return areUrlsEqual2(a, b, state);
    }
    if (tag === ERROR_TAG) {
      return areErrorsEqual2(a, b, state);
    }
    if (tag === ARGUMENTS_TAG) {
      return areObjectsEqual2(a, b, state);
    }
    if (tag === BOOLEAN_TAG || tag === NUMBER_TAG || tag === STRING_TAG) {
      return arePrimitiveWrappersEqual2(a, b, state);
    }
    return false;
  };
}
function createEqualityComparatorConfig(_a) {
  var circular = _a.circular, createCustomConfig = _a.createCustomConfig, strict = _a.strict;
  var config = {
    areArraysEqual: strict ? areObjectsEqualStrict : areArraysEqual,
    areDatesEqual,
    areErrorsEqual,
    areFunctionsEqual,
    areMapsEqual: strict ? combineComparators(areMapsEqual, areObjectsEqualStrict) : areMapsEqual,
    areNumbersEqual,
    areObjectsEqual: strict ? areObjectsEqualStrict : areObjectsEqual,
    arePrimitiveWrappersEqual,
    areRegExpsEqual,
    areSetsEqual: strict ? combineComparators(areSetsEqual, areObjectsEqualStrict) : areSetsEqual,
    areTypedArraysEqual: strict ? areObjectsEqualStrict : areTypedArraysEqual,
    areUrlsEqual
  };
  if (createCustomConfig) {
    config = assign({}, config, createCustomConfig(config));
  }
  if (circular) {
    var areArraysEqual$1 = createIsCircular(config.areArraysEqual);
    var areMapsEqual$1 = createIsCircular(config.areMapsEqual);
    var areObjectsEqual$1 = createIsCircular(config.areObjectsEqual);
    var areSetsEqual$1 = createIsCircular(config.areSetsEqual);
    config = assign({}, config, {
      areArraysEqual: areArraysEqual$1,
      areMapsEqual: areMapsEqual$1,
      areObjectsEqual: areObjectsEqual$1,
      areSetsEqual: areSetsEqual$1
    });
  }
  return config;
}
function createInternalEqualityComparator(compare) {
  return function(a, b, _indexOrKeyA, _indexOrKeyB, _parentA, _parentB, state) {
    return compare(a, b, state);
  };
}
function createIsEqual(_a) {
  var circular = _a.circular, comparator = _a.comparator, createState = _a.createState, equals = _a.equals, strict = _a.strict;
  if (createState) {
    return function isEqual(a, b) {
      var _a2 = createState(), _b = _a2.cache, cache = _b === void 0 ? circular ? /* @__PURE__ */ new WeakMap() : void 0 : _b, meta = _a2.meta;
      return comparator(a, b, {
        cache,
        equals,
        meta,
        strict
      });
    };
  }
  if (circular) {
    return function isEqual(a, b) {
      return comparator(a, b, {
        cache: /* @__PURE__ */ new WeakMap(),
        equals,
        meta: void 0,
        strict
      });
    };
  }
  var state = {
    cache: void 0,
    equals,
    meta: void 0,
    strict
  };
  return function isEqual(a, b) {
    return comparator(a, b, state);
  };
}
var deepEqual = createCustomEqual();
var strictDeepEqual = createCustomEqual({ strict: true });
var circularDeepEqual = createCustomEqual({ circular: true });
var strictCircularDeepEqual = createCustomEqual({
  circular: true,
  strict: true
});
var shallowEqual = createCustomEqual({
  createInternalComparator: function() {
    return sameValueZeroEqual;
  }
});
var strictShallowEqual = createCustomEqual({
  strict: true,
  createInternalComparator: function() {
    return sameValueZeroEqual;
  }
});
var circularShallowEqual = createCustomEqual({
  circular: true,
  createInternalComparator: function() {
    return sameValueZeroEqual;
  }
});
var strictCircularShallowEqual = createCustomEqual({
  circular: true,
  createInternalComparator: function() {
    return sameValueZeroEqual;
  },
  strict: true
});
function createCustomEqual(options) {
  if (options === void 0) {
    options = {};
  }
  var _a = options.circular, circular = _a === void 0 ? false : _a, createCustomInternalComparator = options.createInternalComparator, createState = options.createState, _b = options.strict, strict = _b === void 0 ? false : _b;
  var config = createEqualityComparatorConfig(options);
  var comparator = createEqualityComparator(config);
  var equals = createCustomInternalComparator ? createCustomInternalComparator(comparator) : createInternalEqualityComparator(comparator);
  return createIsEqual({ circular, comparator, createState, equals, strict });
}

// src/utils/getChanges.ts
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
      const deepEqItem = deepEqual(prev[i], next[i]);
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
      const deepEqProp = deepEqual(prev[key], next[key]);
      const refDiffProp = isObject(prev[key]) && isObject(next[key]) && prev[key] !== next[key];
      if (!deepEqProp || refDiffProp) {
        changedKeys.push(key);
        if (refDiffProp && deepEqProp) {
          unstableKeys.push(key);
        }
      }
    }
  } else {
    const deepEqRoot = deepEqual(prev, next);
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
  const scopeCount = import_react.default.useRef(getScopeCount(scope)).current;
  const scopeId = `${scope.id}-${scopeCount}`;
  if (!scopes[scopeId]) {
    scopes[scopeId] = {
      scopeId,
      renderCount: 0,
      ...scope,
      hookResults: {},
      hookChanges: []
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
              scope,
              ...changes,
              previousResult: prevResult,
              currentResult: hookResult
            };
            if (hookEndEvent.arguments) {
              hookCall.arguments = hookEndEvent.arguments;
            }
            scopes[scopeId].hookChanges.push(hookCall);
            callOnHookChange(hookCall);
          }
        }
        currentScope.hookResults[hookId] = hookResult;
        hookStack.delete(hookId);
        return hookResult;
      },
      re: (renderResult) => {
        callOnRender(scopes[scopeId]);
        return renderResult;
      }
    };
  }
  return hooks.current;
}
function reactJitter(options) {
  var _a, _b, _c, _d, _e, _f;
  if (typeof window === "undefined") {
    return;
  }
  const windowGlobal = window;
  windowGlobal.reactJitter = {
    enabled: (_b = (_a = windowGlobal.reactJitter) == null ? void 0 : _a.enabled) != null ? _b : options.enabled,
    onHookChange: (_d = (_c = windowGlobal.reactJitter) == null ? void 0 : _c.onHookChange) != null ? _d : options.onHookChange,
    onRender: (_f = (_e = windowGlobal.reactJitter) == null ? void 0 : _e.onRender) != null ? _f : options.onRender,
    clear: () => {
      Object.keys(scopes).forEach((key) => {
        scopes[key].renderCount = 0;
        scopes[key].hookChanges = [];
        scopes[key].hookResults = {};
      });
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
function shouldReportRender() {
  var _a;
  return typeof ((_a = window == null ? void 0 : window.reactJitter) == null ? void 0 : _a.onRender) === "function" && window.reactJitter.enabled;
}
function callOnRender(scope) {
  var _a;
  if (shouldReportRender() && ((_a = window.reactJitter) == null ? void 0 : _a.onRender)) {
    window.reactJitter.onRender(scope);
  }
}
function getScopeCount(scope) {
  if (!scopeCounter[scope.id]) {
    scopeCounter[scope.id] = 0;
  }
  return scopeCounter[scope.id]++;
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
