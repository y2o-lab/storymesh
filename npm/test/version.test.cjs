const assert = require("node:assert/strict");
const { cpSync, mkdtempSync, readFileSync, rmSync, writeFileSync } = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const test = require("node:test");
const { spawnSync } = require("node:child_process");

function copyVersionFiles(t) {
  const root = path.resolve(__dirname, "../..");
  const temporaryRoot = mkdtempSync(path.join(os.tmpdir(), "storymesh-version-test-"));
  t.after(() => rmSync(temporaryRoot, { recursive: true, force: true }));
  for (const file of ["Cargo.toml", "Cargo.lock", "npm", "scripts"]) {
    cpSync(path.join(root, file), path.join(temporaryRoot, file), { recursive: true });
  }
  return temporaryRoot;
}

test("version updater synchronizes every release manifest and dependency", async (t) => {
  const root = copyVersionFiles(t);
  const { validatePackageVersions } = await import("../../scripts/npm-packages.mjs");
  for (const version of ["0.3.0-rc.1", "0.3.0"]) {
    const result = spawnSync(process.execPath, [path.join(root, "scripts/set-version.mjs"), version], { encoding: "utf8" });
    assert.equal(result.status, 0, result.stderr);
    assert.equal(await validatePackageVersions(root), version);
    const check = spawnSync(process.execPath, [path.join(root, "scripts/check-npm-version.mjs"), "9.9.9"], { encoding: "utf8" });
    assert.notEqual(check.status, 0);
    assert.match(check.stderr, /does not match package version/);
  }
});

test("invalid release input fails without changing version files", (t) => {
  const root = copyVersionFiles(t);
  const before = readFileSync(path.join(root, "Cargo.toml"), "utf8");
  const result = spawnSync(process.execPath, [path.join(root, "scripts/set-version.mjs"), "0.3.0; echo injected"], { encoding: "utf8" });
  assert.equal(result.status, 2);
  assert.equal(readFileSync(path.join(root, "Cargo.toml"), "utf8"), before);
});

test("validation detects each npm package or optional dependency left behind", async (t) => {
  const { platformPackages, validatePackageVersions } = await import("../../scripts/npm-packages.mjs");
  const cases = [
    ["npm/storymesh/package.json", (manifest) => { manifest.version = "0.0.0"; }],
    ...platformPackages.map(({ name }) => [
      `npm/platforms/${name}/package.json`,
      (manifest) => { manifest.version = "0.0.0"; },
    ]),
    ...platformPackages.map(({ name }) => [
      "npm/storymesh/package.json",
      (manifest) => { manifest.optionalDependencies[name] = "0.0.0"; },
    ]),
  ];
  const root = copyVersionFiles(t);
  for (const [file, mutate] of cases) {
    const target = path.join(root, file);
    const original = readFileSync(target, "utf8");
    const manifest = JSON.parse(original);
    mutate(manifest);
    writeFileSync(target, JSON.stringify(manifest));
    await assert.rejects(validatePackageVersions(root), /version|optionalDependencies/);
    writeFileSync(target, original);
  }
});

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
