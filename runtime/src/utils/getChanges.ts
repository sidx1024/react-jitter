import { deepEqual } from 'fast-equals';

export function getChanges(prev: unknown, next: unknown) {
  const changedKeys = [];
  const unstableKeys = [];
  const isObject = (v: unknown): v is Record<string, unknown> =>
    v !== null && typeof v === 'object';

  const prevIsArr = Array.isArray(prev);
  const nextIsArr = Array.isArray(next);

  // if one’s an array and the other isn’t, it's a value change – unstable should be false
  if (prevIsArr !== nextIsArr) {
    return {
      unstable: false,
      unstableKeys: [],
      changedKeys: ['*'],
    };
  }

  // both arrays
  if (prevIsArr && nextIsArr) {
    if (prev.length !== next.length) {
      changedKeys.push('length');
    }

    const max = Math.max(prev.length, next.length);
    for (let i = 0; i < max; i++) {
      if (
        !deepEqual(prev[i], next[i]) ||
        (isObject(prev[i]) && isObject(next[i]) && prev[i] !== next[i])
      ) {
        const key = String(i);
        changedKeys.push(key);
        if (isObject(prev[i]) || isObject(next[i])) {
          unstableKeys.push(key);
        }
      }
    }

    // both plain objects
  } else if (isObject(prev) && isObject(next)) {
    const allKeys = new Set([...Object.keys(prev), ...Object.keys(next)]);
    for (const key of allKeys) {
      if (
        !deepEqual(prev[key], next[key]) ||
        (isObject(prev[key]) && isObject(next[key]) && prev[key] !== next[key])
      ) {
        changedKeys.push(key);
        if (isObject(prev[key]) || isObject(next[key])) {
          unstableKeys.push(key);
        }
      }
    }

    // primitives (or mismatched types other than array↔object)
  } else {
    const unstable = isObject(prev) && isObject(next) && !deepEqual(prev, next);
    const changed = !deepEqual(prev, next);
    return {
      unstable,
      unstableKeys: [],
      changedKeys: changed ? [''] : [],
    };
  }

  const isPlainObject = (v: unknown): v is Record<string, unknown> =>
    v !== null && typeof v === 'object' && !Array.isArray(v);

  const unstableRoot =
    isPlainObject(prev) &&
    isPlainObject(next) &&
    prev !== next &&
    deepEqual(prev, next);

  if (unstableRoot && changedKeys.length === 0) {
    // For plain object reference change, ensure root sentinel marks both changed and unstable
    changedKeys.push('');
    unstableKeys.push('');
  }

  return {
    unstable: unstableKeys.length > 0,
    unstableKeys,
    changedKeys,
  };
}
