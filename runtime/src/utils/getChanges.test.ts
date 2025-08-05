import { expect, test, describe } from 'vitest';
import { getChanges } from './getChanges';

describe('getChanges', () => {
  test('handles primitive values', () => {
    // No change
    expect(getChanges(1, 1)).toEqual({
      unstable: false,
      unstableKeys: [],
      changedKeys: [],
    });

    // Changed primitives
    expect(getChanges(1, 2)).toEqual({
      unstable: false,
      unstableKeys: [],
      changedKeys: [''],
    });

    expect(getChanges('a', 'b')).toEqual({
      unstable: false,
      unstableKeys: [],
      changedKeys: [''],
    });
  });

  test('handles arrays', () => {
    // No change
    expect(getChanges([1, 2, 3], [1, 2, 3])).toEqual({
      unstable: false,
      unstableKeys: [],
      changedKeys: [],
    });

    // Length change
    expect(getChanges([1, 2], [1, 2, 3])).toEqual({
      unstable: false,
      unstableKeys: [],
      changedKeys: ['length', '2'],
    });

    // Value change
    expect(getChanges([1, 2, 3], [1, 5, 3])).toEqual({
      unstable: false,
      unstableKeys: [],
      changedKeys: ['1'],
    });

    // Object in array value change (deep value differs) – unstable should be false
    expect(getChanges([{ a: 1 }], [{ a: 2 }])).toEqual({
      unstable: false,
      unstableKeys: [],
      changedKeys: ['0'],
    });

    // Object in array reference change only – unstable should be true with key 0
    expect(getChanges([{ a: 1 }], [{ a: 1 }])).toEqual({
      unstable: true,
      unstableKeys: ['0'],
      changedKeys: ['0'],
    });
  });

  test('handles objects', () => {
    // Same value but different reference should be unstable
    expect(getChanges({ a: 1, b: 2 }, { a: 1, b: 2 })).toEqual({
      unstable: true,
      unstableKeys: [''],
      changedKeys: [''],
    });

    // Value change
    expect(getChanges({ a: 1, b: 2 }, { a: 1, b: 3 })).toEqual({
      unstable: false,
      unstableKeys: [],
      changedKeys: ['b'],
    });

    // Added key
    expect(getChanges({ a: 1 }, { a: 1, b: 2 })).toEqual({
      unstable: false,
      unstableKeys: [],
      changedKeys: ['b'],
    });

    // Removed key
    expect(getChanges({ a: 1, b: 2 }, { a: 1 })).toEqual({
      unstable: false,
      unstableKeys: [],
      changedKeys: ['b'],
    });

    // Nested object value change (b differs) – should NOT be unstable
    expect(getChanges({ a: { b: 1 } }, { a: { b: 2 } })).toEqual({
      unstable: false,
      unstableKeys: [],
      changedKeys: ['a'],
    });

    // Nested object reference-only change – unstable true, key 'a'
    expect(getChanges({ a: { b: 1 } }, { a: { b: 1 } })).toEqual({
      unstable: true,
      unstableKeys: ['a'],
      changedKeys: ['a'],
    });
  });

  test('handles object reference equality', () => {
    // Different references but same value
    expect(getChanges({ a: 3 }, { a: 3 })).toEqual({
      unstable: true,
      unstableKeys: [''],
      changedKeys: [''],
    });

    // Same reference objects
    const obj = { a: 3 };
    const obj2 = obj;
    expect(getChanges(obj, obj2)).toEqual({
      unstable: false,
      unstableKeys: [],
      changedKeys: [],
    });
  });

  test('handles type mismatches', () => {
    // Array vs Object
    expect(getChanges([1, 2], { 0: 1, 1: 2 })).toEqual({
      unstable: false,
      unstableKeys: [],
      changedKeys: ['*'],
    });

    // Array vs Primitive
    expect(getChanges([1], 1)).toEqual({
      unstable: false,
      unstableKeys: [],
      changedKeys: ['*'],
    });

    // Object vs Primitive should be stable (not unstable)
    expect(getChanges({ a: 1 }, 1)).toEqual({
      unstable: false,
      unstableKeys: [],
      changedKeys: [''],
    });

    // Primitive vs Object should be stable (not unstable)
    expect(getChanges(undefined, { a: 3 })).toEqual({
      unstable: false,
      unstableKeys: [],
      changedKeys: [''],
    });
  });

  test('practical props comparison', () => {
    const prev = {
      user: { id: 1, name: 'Alice' },
      todos: [
        { id: 1, done: false },
        { id: 2, done: true },
      ],
    };

    const next = {
      user: { id: 1, name: 'Alice' }, // new reference but same value
      todos: [
        { id: 1, done: false },
        { id: 2, done: true },
      ],
    };

    expect(getChanges(prev, next)).toEqual({
      unstable: true,
      unstableKeys: ['user', 'todos'],
      changedKeys: ['user', 'todos'],
    });
  });
});
