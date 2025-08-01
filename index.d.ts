type Prettify<T> = {
  [K in keyof T]: T[K];
} & {};

declare module "react-jitter" {
  /**
   * Configuration options for react-jitter
   */
  export interface ReactJitterOptions {
    /**
     * List of hook names to ignore (not transform)
     * By default, all default react hooks are ignored except useContext and useReducer.
     */
    ignoreHooks?: string[];

    /**
     * List of glob patterns to exclude from transformation
     * By default, all node_modules are excluded.
     */
    exclude?: string[];
  }

  /**
   * Configuration for react-jitter plugin
   * Can be either a boolean to enable/disable or an options object
   */
  export type ReactJitterConfig = Prettify<ReactJitterOptions>;
}
