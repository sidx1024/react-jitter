import { getChanges } from './getChanges';
import type { HookAddress } from '../types';

export function compareChanges(
  hookAddress: HookAddress,
  prev: unknown,
  current: unknown,
) {
  if (prev !== 'undefined' && prev !== current) {
    const comparator =
      window?.reactJitter?.selectComparator?.(hookAddress) ?? 'deepEqual';

    try {
      return getChanges(prev, current, comparator);
    } catch (error: unknown) {
      const isRecursionError =
        error instanceof Error &&
        /(?:maximum call stack(?: size)? exceeded|too much recursion|stack overflow)/i.test(
          String(error?.message),
        );

      if (isRecursionError && comparator !== 'circularDeepEqual') {
        throw new Error(
          `Maximum call stack size exceeded. Please use the "circularDeepEqual" comparator with selectComparator option. \nHook address: ${JSON.stringify(hookAddress, null, 2)}.`,
          { cause: error },
        );
      }

      throw error;
    }
  }

  return null;
}
