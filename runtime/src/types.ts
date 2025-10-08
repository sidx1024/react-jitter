import { z } from 'zod';

// Base schema for hook execution metadata
export const HookExecutionSchema = z.object({
  id: z.string(),
  hook: z.string(),
  line: z.number(),
  offset: z.number(),
});

// Schema for scope initialization options
export const ScopeSchema = z.object({
  name: z.string(),
  id: z.string(),
  file: z.string(),
  line: z.number(),
  offset: z.number(),
});

export const HookChangeSchema = z.object({
  unstable: z.boolean(),
  unstableKeys: z.array(z.string()),
  changedKeys: z.array(z.string()),
});

export const ReactJitterGlobalSchema = z.object({
  enabled: z.boolean().optional(),
  onHookChange: z.function().args(z.any()).returns(z.void()).optional(),
  onRender: z.function().args(z.any()).returns(z.void()).optional(),
  clear: z.function().args().returns(z.void()).optional(),
});

// Export TypeScript types derived from the schemas
export type HookExecution = z.infer<typeof HookExecutionSchema>;

export type HookChange = z.infer<typeof HookChangeSchema>;

export type HookEndEvent = {
  id: string;
  hook: string;
  file: string;
  line: number;
  offset: number;
  arguments?: string[];
};

export type HookAddress = Pick<
  HookEndEvent,
  'hook' | 'file' | 'line' | 'offset' | 'arguments'
>;

export type ReactJitterGlobal = z.infer<typeof ReactJitterGlobalSchema>;

export type ReactJitterOptions = {
  enabled?: boolean;
  onHookChange?: (change: HookChange) => void;
  onRender?: (
    scope: Scope & {
      scopeId: string;
      hookResults: Record<string, unknown>;
      renderCount: number;
    },
  ) => void;
};

export type Scope = z.infer<typeof ScopeSchema>;

export type Comparator = 'deepEqual' | 'circularDeepEqual';
