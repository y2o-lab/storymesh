#!/usr/bin/env node

import path from "node:path";
import { fileURLToPath } from "node:url";
import { validatePackageVersions } from "./npm-packages.mjs";
import { distTagForVersion } from "./npm-release-version.mjs";

export async function ensureGithubRelease({ repository, version, sha, draft, token, apiUrl = "https://api.github.com" }, fetchImpl = fetch) {
  if (!repository || !sha || !token || typeof draft !== "boolean") {
    throw new Error("GitHub release requires repository, commit SHA, token, and draft mode");
  }
  const prerelease = distTagForVersion(version) !== "latest";
  const tag = `v${version}`;
  const endpoint = `/repos/${repository}/releases`;
  async function request(method, route, body) {
    const response = await fetchImpl(`${apiUrl}${route}`, {
      method,
      headers: {
        Accept: "application/vnd.github+json",
        Authorization: `Bearer ${token}`,
        "Content-Type": "application/json",
        "X-GitHub-Api-Version": "2022-11-28",
      },
      ...(body ? { body: JSON.stringify(body) } : {}),
    });
    if (!response.ok) {
      throw new Error(`GitHub ${method} ${route} failed (HTTP ${response.status})`);
    }
    return response.json();
  }

  // Listing includes drafts; the release-by-tag endpoint only finds published releases.
  let existing;
  for (let page = 1; ; page += 1) {
    const releases = await request("GET", `${endpoint}?per_page=100&page=${page}`);
    existing = releases.find((release) => release.tag_name === tag);
    if (existing || releases.length < 100) break;
  }
  if (existing) {
    if (!existing.draft) return existing;
    if (existing.target_commitish !== sha) {
      throw new Error(`Draft ${tag} targets ${existing.target_commitish}, not ${sha}; review the release commit before retrying`);
    }
    if (draft) return existing;
    // Preserve generated notes and any maintainer edits when publishing a staged draft.
    return request("PATCH", `${endpoint}/${existing.id}`, {
      draft: false,
      prerelease,
      make_latest: prerelease ? "false" : "legacy",
    });
  }
  return request("POST", endpoint, {
    tag_name: tag,
    target_commitish: sha,
    name: tag,
    draft,
    prerelease,
    generate_release_notes: true,
    make_latest: draft || prerelease ? "false" : "legacy",
  });
}

async function run() {
  if (!["true", "false"].includes(process.env.RELEASE_DRAFT)) {
    throw new Error("RELEASE_DRAFT must be true or false");
  }
  const version = await validatePackageVersions();
  if (process.env.RELEASE_DRAFT === "false" && process.env.GITHUB_REF_NAME !== `v${version}`) {
    throw new Error(`Release tag must match package version v${version}`);
  }
  const release = await ensureGithubRelease({
    repository: process.env.GITHUB_REPOSITORY,
    version,
    sha: process.env.GITHUB_SHA,
    draft: process.env.RELEASE_DRAFT === "true",
    token: process.env.GH_TOKEN,
    apiUrl: process.env.GITHUB_API_URL,
  });
  console.log(`GitHub release ${release.tag_name}: ${release.draft ? "draft" : "published"} (${release.html_url})`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  run().catch((error) => {
    console.error(`GitHub release failed: ${error.message}`);
    process.exitCode = 1;
  });
}
