import deepEqual from 'fast-deep-equal';

export function getChanges(prev: unknown, next: unknown) {
  const changedKeys = [];
  const unstableKeys = [];
  const isObject = (v: unknown): v is Record<string, unknown> =>
    v !== null && typeof v === 'object';

  const prevIsArr = Array.isArray(prev);
  const nextIsArr = Array.isArray(next);

  // if one’s an array and the other isn’t, bail out immediately
  if (prevIsArr !== nextIsArr) {
    return {
      unstable: true,
      unstableKeys: ['*'],
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
      if (!deepEqual(prev[i], next[i])) {
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
      if (!deepEqual(prev[key], next[key])) {
        changedKeys.push(key);
        if (isObject(prev[key]) || isObject(next[key])) {
          unstableKeys.push(key);
        }
      }
    }

    // primitives (or mismatched types other than array↔object)
  } else {
    const unstable = !deepEqual(prev, next);
    return {
      unstable,
      unstableKeys: [],
      changedKeys: unstable ? [''] : [],
    };
  }

  return {
    unstable: changedKeys.length > 0,
    unstableKeys,
    changedKeys,
  };
}
