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
      const deepEqItem = deepEqual(prev[i], next[i]);
      const refDiffItem =
        isObject(prev[i]) && isObject(next[i]) && prev[i] !== next[i];
      if (!deepEqItem || refDiffItem) {
        const key = String(i);
        changedKeys.push(key);
        if (refDiffItem && deepEqItem) {
          unstableKeys.push(key);
        }
      }
    }

    // both plain objects
  } else if (isObject(prev) && isObject(next)) {
    const allKeys = new Set([...Object.keys(prev), ...Object.keys(next)]);
    for (const key of allKeys) {
      const deepEqProp = deepEqual(prev[key], next[key]);
      const refDiffProp =
        isObject(prev[key]) && isObject(next[key]) && prev[key] !== next[key];
      if (!deepEqProp || refDiffProp) {
        changedKeys.push(key);
        if (refDiffProp && deepEqProp) {
          unstableKeys.push(key);
        }
      }
    }

    // primitives (or mismatched types other than array↔object)
  } else {
    const deepEqRoot = deepEqual(prev, next);
    const refDiffRoot = isObject(prev) && isObject(next) && prev !== next;
    const unstable = refDiffRoot && deepEqRoot;
    const changed = !deepEqRoot || refDiffRoot;
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
