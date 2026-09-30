const assert = require("node:assert/strict");
const { chmodSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { spawnSync } = require("node:child_process");
const test = require("node:test");

test("release versions select the intended npm dist-tag", async () => {
  const { distTagForVersion } = await import("../../scripts/npm-release-version.mjs");
  assert.equal(distTagForVersion("0.2.0"), "latest");
  for (const stage of ["alpha", "beta", "rc", "canary"]) {
    assert.equal(distTagForVersion(`0.2.0-${stage}.1`), stage);
  }
  for (const version of ["0.2.0-preview.1", "0.2.0-beta", "0.2.0-beta.0", "01.2.0"]) {
    assert.throws(() => distTagForVersion(version), /unsupported release version/);
  }
});

for (const [version, expectedTag] of [
  ["0.2.0", "latest"],
  ["0.2.0-alpha.1", "alpha"],
  ["0.2.0-beta.1", "beta"],
  ["0.2.0-rc.1", "rc"],
  ["0.2.0-canary.1", "canary"],
]) {
  test(`publisher passes --tag ${expectedTag} for ${version}`, () => {
    const root = mkdtempSync(path.join(os.tmpdir(), "storymesh-publish-test-"));
    try {
      const packages = path.join(root, "packages");
      const packageDirectory = path.join(packages, "storymesh");
      const bin = path.join(root, "bin");
      mkdirSync(packageDirectory, { recursive: true });
      mkdirSync(bin);
      writeFileSync(path.join(packages, "packages.json"), JSON.stringify({ version, directories: ["storymesh"] }));
      writeFileSync(path.join(packageDirectory, "package.json"), JSON.stringify({ name: "storymesh", version }));

      const npm = path.join(bin, "npm");
      writeFileSync(
        npm,
        `#!${process.execPath}\n` +
          "const fs = require('node:fs');\n" +
          "if (process.argv[2] === 'view') { console.error('E404'); process.exit(1); }\n" +
          "fs.writeFileSync(process.env.NPM_CALL_LOG, JSON.stringify(process.argv.slice(2)));\n",
      );
      chmodSync(npm, 0o755);

      const log = path.join(root, "npm-call.json");
      const result = spawnSync(
        process.execPath,
        [path.resolve(__dirname, "../../scripts/publish-npm-packages.mjs"), packages],
        {
          encoding: "utf8",
          env: { ...process.env, PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}`, NPM_CALL_LOG: log },
        },
      );
      assert.equal(result.status, 0, result.stderr);
      assert.deepEqual(JSON.parse(readFileSync(log, "utf8")), [
        "publish",
        packageDirectory,
        "--tag",
        expectedTag,
      ]);
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
}
