import { expect, test } from "vitest";
import { transform } from "@swc/core";
import type { ParserConfig } from "@swc/core";
import path from "node:path";
import url from "node:url";
import fs from "node:fs";

const pluginName =
  process.env.TEST_DEBUG === "true"
    ? "swc_plugin_react_jitter_debug.wasm"
    : "swc_plugin_react_jitter.wasm";
const pluginPath = path.join(
  path.dirname(url.fileURLToPath(import.meta.url)),
  "..",
  "plugin-swc",
  pluginName
);

const transformCode = async (
  code: string,
  options = {},
  filename = "test.jsx"
) => {
  const ext = path.extname(filename);
  const isTypescript = ext === ".ts" || ext === ".tsx";
  const parser: ParserConfig = isTypescript
    ? { syntax: "typescript", tsx: ext === ".tsx" }
    : { syntax: "ecmascript", jsx: true };
  return transform(code, {
    jsc: {
      parser,
      target: "es2018",
      experimental: { plugins: [[pluginPath, options]] },
    },
    filename,
  });
};

const fixturesDir = path.join(
  path.dirname(url.fileURLToPath(import.meta.url)),
  "fixtures"
);
const fixtureFiles = fs
  .readdirSync(fixturesDir)
  .filter((f) => f.endsWith(".jsx") || f.endsWith(".tsx"));

for (const file of fixtureFiles) {
  test(`fixture: ${file}`, async () => {
    const input = fs.readFileSync(path.join(fixturesDir, file), "utf-8");
    const options = file === "1_default.tsx" ? { includeArguments: true } : {};
    const { code } = await transformCode(input, options, file);
    expect(code.trim()).toMatchSnapshot();
  });
}
