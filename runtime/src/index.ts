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

const scopes: Record<
  string,
  Scope & {
    hookResults: Record<string, unknown>;
    renderCount: number;
    scopeId: string;
    hookChanges: HookCall[];
  }
> = {};

const hookStack = new Map<string, unknown>();

const scopeCounter: Record<string, number> = {};

declare global {
  interface Window {
    reactJitter?: {
      enabled?: boolean;
      onHookChange?: (change: HookCall) => void;
      onRender?: (
        scope: Scope & {
          hookResults: Record<string, unknown>;
          renderCount: number;
        },
      ) => void;
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
      scopeId,
      renderCount: 0,
      ...scope,
      hookResults: {},
      hookChanges: [],
    };
  }

  scopes[scopeId].renderCount++;

  const hooks = React.useRef<{
    s: (id: string) => void;
    e: (hookResult: unknown, hookEndEvent: HookEndEvent) => unknown;
    re: <T>(renderResult: T) => T;
  } | null>(null);

  if (!hooks.current) {
    hooks.current = {
      s: (id: string) => {
        const hookId = `${scopeId}-${id}`;
        hookStack.set(hookId, null);
      },
      e: (hookResult: unknown, hookEndEvent: HookEndEvent) => {
        const currentScope = scopes[scopeId];
        if (!currentScope) {
          return hookResult;
        }

        const hookId = `${scopeId}-${hookEndEvent.id}`;

        if (shouldReportChanges()) {
          const prevResult = currentScope.hookResults[hookId];
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
            scopes[scopeId].hookChanges.push(hookCall);
            callOnHookChange(hookCall);
          }
        }

        currentScope.hookResults[hookId] = hookResult;
        hookStack.delete(hookId);

        return hookResult;
      },
      re: <T>(renderResult: T): T => {
        // Render end - call onRender callback with scope data
        callOnRender(scopes[scopeId]);
        return renderResult;
      },
    };
  }

  // TODO: Think about cleanup strategy

  return hooks.current;
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
    onRender: windowGlobal.reactJitter?.onRender ?? options.onRender,
    clear: () => {
      Object.keys(scopes).forEach((key) => {
        scopes[key].renderCount = 0;
        scopes[key].hookChanges = [];
        scopes[key].hookResults = {};
      });
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

function shouldReportRender() {
  return (
    typeof window?.reactJitter?.onRender === 'function' &&
    window.reactJitter.enabled
  );
}

function callOnRender(
  scope: Scope & {
    hookResults: Record<string, unknown>;
    renderCount: number;
  },
) {
  if (shouldReportRender() && window.reactJitter?.onRender) {
    window.reactJitter.onRender(scope);
  }
}

function getScopeCount(scope: Scope) {
  if (!scopeCounter[scope.id]) {
    scopeCounter[scope.id] = 0;
  }

  return scopeCounter[scope.id]++;
}

function compareChanges(prev: unknown, current: unknown) {
  if (prev !== 'undefined' && prev !== current) {
    return getChanges(prev, current);
  }

  return null;
}
