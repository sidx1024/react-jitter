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
      unstable: true,
      unstableKeys: [],
      changedKeys: [''],
    });

    expect(getChanges('a', 'b')).toEqual({
      unstable: true,
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
      unstable: true,
      unstableKeys: [],
      changedKeys: ['length', '2'],
    });

    // Value change
    expect(getChanges([1, 2, 3], [1, 5, 3])).toEqual({
      unstable: true,
      unstableKeys: [],
      changedKeys: ['1'],
    });

    // Object in array change
    expect(getChanges([{ a: 1 }], [{ a: 2 }])).toEqual({
      unstable: true,
      unstableKeys: ['0'],
      changedKeys: ['0'],
    });
  });

  test('handles objects', () => {
    // No change
    expect(getChanges({ a: 1, b: 2 }, { a: 1, b: 2 })).toEqual({
      unstable: false,
      unstableKeys: [],
      changedKeys: [],
    });

    // Value change
    expect(getChanges({ a: 1, b: 2 }, { a: 1, b: 3 })).toEqual({
      unstable: true,
      unstableKeys: [],
      changedKeys: ['b'],
    });

    // Added key
    expect(getChanges({ a: 1 }, { a: 1, b: 2 })).toEqual({
      unstable: true,
      unstableKeys: [],
      changedKeys: ['b'],
    });

    // Removed key
    expect(getChanges({ a: 1, b: 2 }, { a: 1 })).toEqual({
      unstable: true,
      unstableKeys: [],
      changedKeys: ['b'],
    });

    // Nested object change
    expect(getChanges({ a: { b: 1 } }, { a: { b: 2 } })).toEqual({
      unstable: true,
      unstableKeys: ['a'],
      changedKeys: ['a'],
    });
  });

  test('handles type mismatches', () => {
    // Array vs Object
    expect(getChanges([1, 2], { 0: 1, 1: 2 })).toEqual({
      unstable: true,
      unstableKeys: ['*'],
      changedKeys: ['*'],
    });

    // Array vs Primitive
    expect(getChanges([1], 1)).toEqual({
      unstable: true,
      unstableKeys: ['*'],
      changedKeys: ['*'],
    });

    // Object vs Primitive
    expect(getChanges({ a: 1 }, 1)).toEqual({
      unstable: true,
      unstableKeys: [],
      changedKeys: [''],
    });
  });
});
