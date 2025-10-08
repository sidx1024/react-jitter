import { expect, test, describe, beforeEach, afterEach, vi } from 'vitest';
import { compareChanges } from './compareChanges';
import type { HookAddress } from '../types';

describe('compareChanges', () => {
  const hookAddress: HookAddress = {
    hook: 'useState',
    file: 'src/utils/test.tsx',
    line: 10,
    offset: 5,
  };

  let originalWindow: typeof globalThis.window;

  beforeEach(() => {
    originalWindow = globalThis.window;
  });

  afterEach(() => {
    globalThis.window = originalWindow;
  });

  test('returns null when prev is "undefined" string', () => {
    expect(compareChanges(hookAddress, 'undefined', { a: 1 })).toBeNull();
  });

  test('returns null when prev === current', () => {
    const value = { a: 1 };
    expect(compareChanges(hookAddress, value, value)).toBeNull();
  });

  test('returns getChanges result when values differ', () => {
    const result = compareChanges(hookAddress, { a: 1 }, { a: 2 });
    expect(result).toMatchObject({
      unstable: expect.any(Boolean),
      unstableKeys: expect.any(Array),
      changedKeys: expect.any(Array),
    });
  });

  test('defaults to deepEqual comparator', () => {
    globalThis.window = {} as typeof globalThis.window;
    const result = compareChanges(hookAddress, { a: 1 }, { a: 2 });
    expect(result).not.toBeNull();
  });

  test('calls selectComparator with hookAddress', () => {
    const selectComparator = vi.fn(() => 'deepEqual' as const);
    globalThis.window = {
      reactJitter: { selectComparator },
    } as unknown as typeof globalThis.window;

    compareChanges(hookAddress, { a: 1 }, { a: 2 });

    expect(selectComparator).toHaveBeenCalledWith(hookAddress);
  });

  test('uses circularDeepEqual when selected', () => {
    const selectComparator = vi.fn(() => 'circularDeepEqual' as const);
    globalThis.window = {
      reactJitter: { selectComparator },
    } as unknown as typeof globalThis.window;

    const obj = { value: 1 } as Record<string, unknown>;
    obj.self = obj;

    const result = compareChanges(hookAddress, obj, obj);
    expect(result).toBeNull();
  });

  test('throws enhanced error for stack overflow with deepEqual', () => {
    const deep = (d: number): unknown =>
      d === 0 ? { v: 1 } : { nested: deep(d - 1) };

    try {
      compareChanges(hookAddress, deep(10000), deep(10000));
    } catch (error) {
      expect((error as Error).message).toContain('Maximum call stack');
      expect((error as Error).message).toContain('circularDeepEqual');
      expect((error as Error).message).toContain('useState');
      expect((error as Error & { cause?: unknown }).cause).toBeDefined();
    }
  });

  test('does not throw enhanced error with circularDeepEqual', () => {
    const selectComparator = vi.fn(() => 'circularDeepEqual' as const);
    globalThis.window = {
      reactJitter: { selectComparator },
    } as unknown as typeof globalThis.window;

    const deep = (d: number): unknown =>
      d === 0 ? { v: 1 } : { nested: deep(d - 1) };

    try {
      compareChanges(hookAddress, deep(10000), deep(10000));
    } catch (error) {
      expect((error as Error).message).not.toContain('circularDeepEqual');
    }
  });
});
