import type {
  HookChange,
  HookEndEvent,
  ReactJitterOptions,
  Scope,
} from './types';

import React from 'react';
import { getChanges } from './utils/getChanges';

type HookCall = HookChange &
  HookEndEvent & {
    scope: Scope;
    previousResult: unknown;
    currentResult: unknown;
  };

const scopes: Record<string, Scope & { hookResults: Record<string, unknown> }> =
  {};

const hookStack = new Map<string, unknown>();

const scopeCounter: Record<string, number> = {};

declare global {
  interface Window {
    reactJitter?: {
      enabled?: boolean;
      onHookChange?: (change: HookCall) => void;
      clear: () => void;
    };
  }
}

/**
 * A React hook that creates a jitter scope for measuring component performance.
 * @param options Configuration options for the jitter scope
 * @returns void
 */
export function useJitterScope(scope: Scope) {
  const scopeCount = React.useRef(getScopeCount(scope)).current;
  const scopeId = `${scope.id}-${scopeCount}`;

  if (!scopes[scopeId]) {
    scopes[scopeId] = {
      ...scope,
      hookResults: {},
    };
  }

  // TODO: Think about cleanup strategy

  return {
    s: (id: string) => {
      hookStack.set(id, null);
    },
    e: (hookResult: unknown, hookEndEvent: HookEndEvent) => {
      const currentScope = scopes[scopeId];
      if (!currentScope) {
        return hookResult;
      }

      if (shouldReportChanges()) {
        const prevResult = currentScope.hookResults[hookEndEvent.id];
        const changes = compareChanges(prevResult, hookResult);
        if (changes) {
          const hookCall: HookCall = {
            hook: hookEndEvent.hook,
            file: hookEndEvent.file,
            line: hookEndEvent.line,
            offset: hookEndEvent.offset,
            id: hookEndEvent.id,
            scope,
            ...changes,
            previousResult: prevResult,
            currentResult: hookResult,
          };
          if (hookEndEvent.arguments) {
            hookCall.arguments = hookEndEvent.arguments;
          }
          callOnHookChange(hookCall);
        }
      }

      currentScope.hookResults[hookEndEvent.id] = hookResult;
      hookStack.delete(hookEndEvent.id);

      return hookResult;
    },
  };
}

export function reactJitter(options: ReactJitterOptions) {
  if (typeof window === 'undefined') {
    return;
  }

  const windowGlobal = window as any;
  windowGlobal.reactJitter = {
    enabled: windowGlobal.reactJitter?.enabled ?? options.enabled,
    onHookChange:
      windowGlobal.reactJitter?.onHookChange ?? options.onHookChange,
    clear: () => {
      Object.keys(scopes).forEach((key) => delete scopes[key]);
    },
  };
}

function shouldReportChanges() {
  return (
    typeof window?.reactJitter?.onHookChange === 'function' &&
    window.reactJitter.enabled
  );
}

function callOnHookChange(hookResult: HookCall) {
  if (shouldReportChanges() && window.reactJitter?.onHookChange) {
    window.reactJitter.onHookChange(hookResult);
  }
}

function getScopeCount(scope: Scope) {
  if (!scopeCounter[scope.id]) {
    scopeCounter[scope.id] = 0;
  }

  return scopeCounter[scope.id];
}

function compareChanges(prev: unknown, current: unknown) {
  if (prev !== 'undefined' && prev !== current) {
    return getChanges(prev, current);
  }

  return null;
}
