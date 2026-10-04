#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { distTagForVersion } from "./npm-release-version.mjs";

const root = path.resolve(process.argv[2] ?? "");
const stage = process.argv[3] === "--stage";
if (!process.argv[2] || process.argv.length > 4 || (process.argv[3] && !stage)) {
  console.error("usage: scripts/publish-npm-packages.mjs PREPARED_PACKAGES_DIRECTORY [--stage]");
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

if (stage) console.log("Review Staged Packages on npmjs.com with 2FA: approve the five platform packages before storymesh.");
