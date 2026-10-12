#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { distTagForVersion } from "./npm-release-version.mjs";

const root = path.resolve(process.argv[2] ?? "");
const options = process.argv.slice(3);
const stage = options.includes("--stage");
const skipWindows = options.includes("--skip-windows");
if (!process.argv[2] || options.some((option) => !["--stage", "--skip-windows"].includes(option))) {
  console.error("usage: scripts/publish-npm-packages.mjs PREPARED_PACKAGES_DIRECTORY [--stage] [--skip-windows]");
  process.exit(2);
}

const { version, directories } = JSON.parse(await readFile(path.join(root, "packages.json"), "utf8"));
const distTag = distTagForVersion(version);
let staged = [];
if (stage) {
  const result = spawnSync("npm", ["stage", "list", "--json"], { encoding: "utf8" });
  if (result.status !== 0) throw new Error(`could not list staged packages: ${result.stderr}`);
  staged = JSON.parse(result.stdout);
  if (!Array.isArray(staged)) throw new Error("npm stage list returned an unexpected response");
}

for (const directory of directories) {
  const manifest = JSON.parse(await readFile(path.join(root, directory, "package.json"), "utf8"));
  if (manifest.version !== version) throw new Error(`${manifest.name} has an unexpected version`);

  const spec = `${manifest.name}@${version}`;
  if (skipWindows && manifest.name === "storymesh-windows-x64") {
    console.log(`${spec} is temporarily excluded from npm ${stage ? "staging" : "publishing"}; skipping.`);
    continue;
  }
  const existing = spawnSync("npm", ["view", spec, "version", "--json"], { encoding: "utf8" });
  if (existing.status === 0) {
    console.log(`${spec} is already published; skipping.`);
    continue;
  }
  if (!`${existing.stdout}\n${existing.stderr}`.includes("E404")) {
    throw new Error(`could not check ${spec}: ${existing.stderr || existing.stdout}`);
  }

  if (stage && staged.some((item) => item.packageName === manifest.name && item.version === version)) {
    console.log(`${spec} is already staged; review and approve it on npmjs.com.`);
    continue;
  }

  console.log(`${stage ? "Staging" : "Publishing"} ${spec} with npm dist-tag ${distTag}...`);
  const command = stage ? ["stage", "publish"] : ["publish"];
  const published = spawnSync("npm", [...command, path.join(root, directory), "--tag", distTag], {
    stdio: "inherit",
  });
  if (published.status !== 0) throw new Error(`npm ${command.join(" ")} failed for ${spec}`);
}

if (stage) console.log("Review Staged Packages on npmjs.com with 2FA: approve the staged platform packages before storymesh.");
