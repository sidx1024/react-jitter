import { z } from 'zod';

declare const ScopeSchema: z.ZodObject<{
    name: z.ZodString;
    id: z.ZodString;
    file: z.ZodString;
    line: z.ZodNumber;
    offset: z.ZodNumber;
}, "strip", z.ZodTypeAny, {
    name: string;
    id: string;
    file: string;
    line: number;
    offset: number;
}, {
    name: string;
    id: string;
    file: string;
    line: number;
    offset: number;
}>;
declare const HookChangeSchema: z.ZodObject<{
    unstable: z.ZodBoolean;
    unstableKeys: z.ZodArray<z.ZodString, "many">;
    changedKeys: z.ZodArray<z.ZodString, "many">;
}, "strip", z.ZodTypeAny, {
    unstable: boolean;
    unstableKeys: string[];
    changedKeys: string[];
}, {
    unstable: boolean;
    unstableKeys: string[];
    changedKeys: string[];
}>;
type HookChange = z.infer<typeof HookChangeSchema>;
type HookEndEvent = {
    id: string;
    hook: string;
    file: string;
    line: number;
    offset: number;
    arguments?: string[];
};
type ReactJitterOptions = {
    enabled?: boolean;
    onHookChange?: (change: HookChange) => void;
};
type Scope = z.infer<typeof ScopeSchema>;

type HookCall = HookChange & HookEndEvent & {
    scopeId: string;
    scope: Scope;
    previousResult: unknown;
    currentResult: unknown;
};
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
declare function useJitterScope(scope: Scope): {
    s: (id: string) => void;
    e: (hookResult: unknown, hookEndEvent: HookEndEvent) => unknown;
};
declare function reactJitter(options: ReactJitterOptions): void;

export { reactJitter, useJitterScope };
