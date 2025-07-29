import equal from "fast-deep-equal";
import { useRef, useEffect } from "react";

/** @typedef {{
 *    i: string,
 *    h: string,
 *    l: number,
 *    f: string,
 *    parent?: HookMeta,
 *    c?: number
 * }} HookMeta */

/** @typedef {{ onHookChange?: (data:any)=>void, enabled?: boolean }} ReactJitterCallbacks */

const previous = new Map();
const stack = [];
const metas = new Map();
const counts = new Map();
const activeScopes = new Set();
const scopeToComponent = new Map();

function getReactJitter() {
  return (
    (typeof globalThis !== "undefined" ? globalThis.reactJitter : undefined) ||
    undefined
  );
}

function getCallbacks() {
  return getReactJitter() ?? {};
}

function enabled() {
  const { enabled, onHookChange } = getCallbacks();
  if (enabled === false) return false;
  if (enabled === true) return true;
  return !!onHookChange;
}

function generateScopeId() {
  if (typeof crypto !== "undefined" && crypto.randomUUID) {
    return crypto.randomUUID();
  }
  return Math.random().toString(36).substring(2, 15);
}

function cleanupScope(scopeId) {
  if (!scopeId) return;

  activeScopes.delete(scopeId);
  scopeToComponent.delete(scopeId);

  const keysToDelete = [];
  for (const key of previous.keys()) {
    if (key.startsWith(`${scopeId}_`)) {
      keysToDelete.push(key);
    }
  }
  for (const key of keysToDelete) {
    previous.delete(key);
  }

  for (const key of metas.keys()) {
    if (key.startsWith(`${scopeId}_`)) {
      keysToDelete.push(key);
    }
  }
  for (const key of keysToDelete) {
    metas.delete(key);
  }

  for (const key of counts.keys()) {
    if (key.startsWith(`${scopeId}_`)) {
      keysToDelete.push(key);
    }
  }
  for (const key of keysToDelete) {
    counts.delete(key);
  }
}

function buildComponentPath(scopeId) {
  const componentName = scopeToComponent.get(scopeId);
  if (!componentName) return [];
  return [componentName];
}

function buildHookPath(hookMeta) {
  const path = [];
  let current = hookMeta;
  while (current) {
    path.unshift(current.h);
    current = current.parent;
  }
  return path;
}

function buildFullPath(scopeId, hookMeta) {
  const componentPath = buildComponentPath(scopeId);
  const hookPath = buildHookPath(hookMeta);
  return [...componentPath, ...hookPath].join(" -> ");
}

function diff(a, b) {
  if (Object.is(a, b)) return [];
  if (a === null || b === null || typeof a !== "object" || typeof b !== "object") {
    return [];
  }
  const keysA = Object.keys(a);
  const keysB = Object.keys(b);
  const allKeys = new Set([...keysA, ...keysB]);
  const changed = [];
  for (const key of allKeys) {
    if (!keysA.includes(key) || !keysB.includes(key) || !equal(a[key], b[key])) {
      changed.push(key);
    }
  }
  return changed;
}

function refDiff(a, b) {
  if (Object.is(a, b)) return [];
  if (a === null || b === null || typeof a !== "object" || typeof b !== "object") {
    return [];
  }
  const keysA = Object.keys(a);
  const keysB = Object.keys(b);
  const allKeys = new Set([...keysA, ...keysB]);
  const changed = [];
  for (const key of allKeys) {
    if (!keysA.includes(key) || !keysB.includes(key) || !Object.is(a[key], b[key])) {
      changed.push(key);
    }
  }
  return changed;
}

function startHook(scopeId, hookId, meta) {
  if (!enabled()) return;
  const scopedId = `${scopeId}_${hookId}`;
  stack.push(scopedId);
  const scopedMeta = {
    i: scopedId,
    h: meta.hook,
    l: meta.line,
    f: meta.file,
  };
  metas.set(scopedId, scopedMeta);
}

function endHook(scopeId, hookId, result, meta) {
  if (!enabled()) return result;
  const { onHookChange } = getCallbacks();
  const scopedId = `${scopeId}_${hookId}`;
  stack.pop();
  const parentId = stack[stack.length - 1];
  if (parentId) {
    meta.parent = metas.get(parentId);
  }
  const count = (counts.get(scopedId) ?? 0) + 1;
  counts.set(scopedId, count);
  const scopedMeta = {
    i: scopedId,
    h: meta.hook,
    l: meta.line,
    f: meta.file,
    parent: meta.parent,
    c: count,
  };
  metas.set(scopedId, scopedMeta);
  const prev = previous.get(scopedId);
  if (prev !== undefined) {
    const isUnstable = !equal(prev, result);
    if (onHookChange) {
      const unstableKeys = isUnstable ? diff(prev, result) : [];
      const changedKeys = refDiff(prev, result);
      function transformParent(hookMeta) {
        if (!hookMeta) return undefined;
        const currentMeta = metas.get(hookMeta.i) || hookMeta;
        return {
          id: currentMeta.i,
          hook: currentMeta.h,
          line: currentMeta.l,
          file: currentMeta.f,
          count: currentMeta.c,
          parent: transformParent(currentMeta.parent),
          getPath: () => buildFullPath(scopeId, currentMeta),
        };
      }
      onHookChange({
        id: scopedMeta.i,
        hook: scopedMeta.h,
        line: scopedMeta.l,
        file: scopedMeta.f,
        parent: transformParent(scopedMeta.parent),
        count: scopedMeta.c,
        unstable: isUnstable,
        unstableKeys,
        changedKeys,
        prev,
        current: result,
        getPath: () => buildFullPath(scopeId, scopedMeta),
      });
    }
  }
  previous.set(scopedId, result);
  return result;
}

export function useJitterScope(componentName) {
  const scopeIdRef = useRef(undefined);
  if (!scopeIdRef.current) {
    scopeIdRef.current = generateScopeId();
    activeScopes.add(scopeIdRef.current);
  }
  if (componentName && scopeIdRef.current) {
    scopeToComponent.set(scopeIdRef.current, componentName);
  }
  useEffect(() => {
    return () => {
      if (scopeIdRef.current) {
        cleanupScope(scopeIdRef.current);
      }
    };
  }, []);
  return {
    s: (hookId, meta) => startHook(scopeIdRef.current, hookId, meta),
    e: (hookId, result, meta) => endHook(scopeIdRef.current, hookId, result, meta),
  };
}

export default { useJitterScope };
