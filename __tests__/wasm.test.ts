import { expect, test } from "vitest";
import { transform } from "@swc/core";
import fs from "node:fs";
import path from "node:path";
import url from "node:url";

const pluginName = "swc_plugin_react_jitter.wasm";
const pluginPath = path.join(
  path.dirname(url.fileURLToPath(import.meta.url)),
  "..",
  pluginName
);

const transformCode = async (
  code: string,
  options = {},
  filename = "test.jsx"
) => {
  return transform(code, {
    jsc: {
      parser: { syntax: "ecmascript", jsx: true },
      target: "es2018",
      experimental: { plugins: [[pluginPath, options]] },
    },
    filename,
  });
};

const fixtureRoot = path.join(
  path.dirname(url.fileURLToPath(import.meta.url)),
  "../transform/tests/fixture"
);
const fixtureDirs = fs.readdirSync(fixtureRoot);

for (const dir of fixtureDirs) {
  test(`fixture: ${dir}`, async () => {
    const inputPath = path.join(fixtureRoot, dir, "input.js");
    const outputPath = path.join(fixtureRoot, dir, "output.js");
    const input = fs.readFileSync(inputPath, "utf-8");
    const expected = fs.readFileSync(outputPath, "utf-8");
    const { code } = await transformCode(input, {}, inputPath);
    expect(code.trim()).toBe(expected.trim());
  });
}
