const assert = require("node:assert/strict");
const { cpSync, mkdtempSync, readFileSync, rmSync, writeFileSync } = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const test = require("node:test");

test("release validation rejects a stale Cargo.lock version", async () => {
  const root = path.resolve(__dirname, "../..");
  const temporaryRoot = mkdtempSync(path.join(os.tmpdir(), "storymesh-version-test-"));
  try {
    cpSync(path.join(root, "Cargo.toml"), path.join(temporaryRoot, "Cargo.toml"));
    const lock = readFileSync(path.join(root, "Cargo.lock"), "utf8");
    writeFileSync(
      path.join(temporaryRoot, "Cargo.lock"),
      lock.replace(
        /(\[\[package\]\]\nname = "storymesh"\nversion = ")[^"]+/,
        (_match, prefix) => `${prefix}0.0.0`,
      ),
    );

    const { validatePackageVersions } = await import("../../scripts/npm-packages.mjs");
    await assert.rejects(validatePackageVersions(temporaryRoot), /Cargo\.lock storymesh version/);
  } finally {
    rmSync(temporaryRoot, { recursive: true, force: true });
  }
});
