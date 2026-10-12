const assert = require("node:assert/strict");
const { chmodSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { spawnSync } = require("node:child_process");
const test = require("node:test");

function runPublisher({ checkStaged = false, staged = [], published = [], listError = false, viewError = false, publishError = false, windowsError = false, extraArgs } = {}) {
  const root = mkdtempSync(path.join(os.tmpdir(), "storymesh-stage-test-"));
  try {
    const packages = path.join(root, "packages");
    const bin = path.join(root, "bin");
    const directories = ["storymesh-linux-x64", "storymesh-windows-x64", "storymesh"];
    mkdirSync(bin);
    for (const name of directories) {
      mkdirSync(path.join(packages, name), { recursive: true });
      writeFileSync(path.join(packages, name, "package.json"), JSON.stringify({ name, version: "0.2.0-beta.1" }));
    }
    writeFileSync(path.join(packages, "packages.json"), JSON.stringify({ version: "0.2.0-beta.1", directories }));
    const log = path.join(root, "calls.jsonl");
    writeFileSync(log, "");
    const npm = path.join(bin, "npm");
    writeFileSync(npm, `#!${process.execPath}\n` + `
const fs = require('node:fs');
const args = process.argv.slice(2);
const scenario = JSON.parse(process.env.NPM_SCENARIO);
fs.appendFileSync(process.env.NPM_CALL_LOG, JSON.stringify(args) + '\\n');
if (scenario.windowsError && args.some((arg) => arg.includes('storymesh-windows-x64'))) {
  console.error('E403 spam classification'); process.exit(1);
}
if (args[0] === 'stage' && args[1] === 'list') {
  if (scenario.listError) { console.error('E401'); process.exit(1); }
  console.log(JSON.stringify(scenario.staged));
} else if (args[0] === 'view') {
  if (scenario.viewError) { console.error('E503'); process.exit(1); }
  if (scenario.published.includes(args[1])) console.log('"0.2.0-beta.1"');
  else { console.error('E404'); process.exit(1); }
} else if (scenario.publishError) { console.error('E403'); process.exit(1); }
`);
    chmodSync(npm, 0o755);
    const result = spawnSync(process.execPath, [
      path.resolve(__dirname, "../../scripts/publish-npm-packages.mjs"),
      packages,
      ...(extraArgs ?? (checkStaged ? ["--check-staged"] : [])),
    ], {
      encoding: "utf8",
      env: {
        ...process.env,
        PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}`,
        NPM_CALL_LOG: log,
        NPM_SCENARIO: JSON.stringify({ staged, published, listError, viewError, publishError, windowsError }),
      },
    });
    const calls = readFileSync(log, "utf8").trim().split("\n").filter(Boolean).map(JSON.parse);
    return { ...result, calls, packages };
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

test("OIDC stages platform packages before the CLI without querying the interactive staging API", () => {
  const result = runPublisher();
  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(result.calls.filter((args) => args.includes("publish")), [
    ["stage", "publish", path.join(result.packages, "storymesh-linux-x64"), "--tag", "beta"],
    ["stage", "publish", path.join(result.packages, "storymesh-windows-x64"), "--tag", "beta"],
    ["stage", "publish", path.join(result.packages, "storymesh"), "--tag", "beta"],
  ]);
  assert.equal(result.calls.some((args) => args[0] === "stage" && args[1] === "list"), false);
  assert.match(result.stdout, /approve the staged platform packages before storymesh/);
});

test("bootstrap skips both staged and already published versions on rerun", () => {
  const result = runPublisher({
    checkStaged: true,
    staged: [{ packageName: "storymesh-linux-x64", version: "0.2.0-beta.1" }],
    published: ["storymesh-windows-x64@0.2.0-beta.1", "storymesh@0.2.0-beta.1"],
  });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.calls.filter((args) => args.includes("publish")).length, 0);
});

test("other staged versions do not prevent staging the requested version", () => {
  const result = runPublisher({ checkStaged: true, staged: [{ packageName: "storymesh", version: "0.2.0-beta.2" }] });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.calls.filter((args) => args.includes("publish")).length, 3);
});

for (const scenario of [{ listError: true }, { staged: {} }, { viewError: true }]) {
  test(`bootstrap stops without staging on lookup failure ${JSON.stringify(scenario)}`, () => {
    const result = runPublisher({ ...scenario, checkStaged: true });
    assert.notEqual(result.status, 0);
    assert.equal(result.calls.filter((args) => args.includes("publish")).length, 0);
  });
}

test("failed platform staging prevents submission of the CLI", () => {
  const result = runPublisher({ publishError: true });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /npm stage publish failed for storymesh-linux-x64/);
  assert.equal(result.calls.filter((args) => args.includes("publish")).length, 1);
});

test("Windows exclusion allows staging the CLI despite Windows rejection", () => {
  const result = runPublisher({
    windowsError: true,
    extraArgs: ["--skip-windows"],
  });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.calls.some((args) => args.some((arg) => arg.includes("storymesh-windows-x64"))), false);
  assert.deepEqual(result.calls.filter((args) => args.includes("publish")), [
    ["stage", "publish", path.join(result.packages, "storymesh-linux-x64"), "--tag", "beta"],
    ["stage", "publish", path.join(result.packages, "storymesh"), "--tag", "beta"],
  ]);
  assert.match(result.stdout, /storymesh-windows-x64@0\.2\.0-beta\.1 is temporarily excluded/);
});

test("unknown options fail before contacting npm", () => {
  const result = runPublisher({ extraArgs: ["--stgae"] });
  assert.equal(result.status, 2);
  assert.deepEqual(result.calls, []);
});
