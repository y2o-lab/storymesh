const assert = require("node:assert/strict");
const test = require("node:test");

async function runRelease({ existing = [], draft = true, version = "0.1.0", status = 200 } = {}) {
  const { ensureGithubRelease } = await import("../../scripts/github-release.mjs");
  const calls = [];
  const promise = ensureGithubRelease({ repository: "y2o-lab/storymesh", version, sha: "release-sha", draft, token: "test-token" }, async (url, options) => {
    const body = options.body && JSON.parse(options.body);
    calls.push({ url, method: options.method, body });
    return {
      ok: status === 200,
      status,
      json: async () => options.method === "GET" ? existing : body,
    };
  });
  return { promise, calls };
}

test("staging creates a draft with generated notes at the staging commit", async () => {
  const { promise, calls } = await runRelease();
  const release = await promise;
  assert.equal(release.tag_name, "v0.1.0");
  assert.equal(release.target_commitish, "release-sha");
  assert.equal(release.draft, true);
  assert.equal(release.generate_release_notes, true);
  assert.equal(release.prerelease, false);
  assert.equal(release.make_latest, "false");
  assert.equal(calls[1].method, "POST");
});

test("tag publication creates a public release when no draft exists", async () => {
  const { promise } = await runRelease({ draft: false });
  const release = await promise;
  assert.equal(release.draft, false);
  assert.equal(release.generate_release_notes, true);
  assert.equal(release.make_latest, "legacy");
});

test("tag publication promotes the existing draft without overwriting edited notes", async () => {
  const { promise, calls } = await runRelease({ draft: false, existing: [
    { id: 42, tag_name: "v0.1.0", target_commitish: "release-sha", draft: true, body: "Reviewed notes" },
  ] });
  await promise;
  assert.equal(calls[1].method, "PATCH");
  assert.match(calls[1].url, /\/releases\/42$/);
  assert.deepEqual(calls[1].body, { draft: false, prerelease: false, make_latest: "legacy" });
});

for (const draft of [true, false]) {
  test(`rerunning ${draft ? "staging" : "publication"} preserves an existing release`, async () => {
    const existing = { tag_name: "v0.1.0", target_commitish: "release-sha", draft };
    const { promise, calls } = await runRelease({ draft, existing: [existing] });
    assert.deepEqual(await promise, existing);
    assert.equal(calls.length, 1);
  });
}

test("staging never turns a published release back into a draft", async () => {
  const { promise, calls } = await runRelease({ existing: [{ tag_name: "v0.1.0", draft: false }] });
  assert.equal((await promise).draft, false);
  assert.equal(calls.length, 1);
});

test("prerelease versions are marked as prereleases and never latest", async () => {
  const { promise } = await runRelease({ draft: false, version: "0.2.0-beta.1" });
  const release = await promise;
  assert.equal(release.prerelease, true);
  assert.equal(release.make_latest, "false");
});

test("a draft for a different commit requires review before staging or publishing", async () => {
  for (const draft of [true, false]) {
    const { promise, calls } = await runRelease({ draft, existing: [
      { tag_name: "v0.1.0", draft: true, target_commitish: "other-sha" },
    ] });
    await assert.rejects(promise, /review the release commit/);
    assert.equal(calls.length, 1);
  }
});

test("lookup failures stop without creating a release", async () => {
  const { promise, calls } = await runRelease({ status: 403 });
  await assert.rejects(promise, /HTTP 403/);
  assert.equal(calls.length, 1);
});

test("release lookup finds drafts beyond the first page", async () => {
  const { ensureGithubRelease } = await import("../../scripts/github-release.mjs");
  const draft = { tag_name: "v0.1.0", target_commitish: "release-sha", draft: true };
  const urls = [];
  const result = await ensureGithubRelease({ repository: "y2o-lab/storymesh", version: "0.1.0", sha: "release-sha", draft: true, token: "test-token" }, async (url) => {
    urls.push(url);
    return { ok: true, json: async () => urls.length === 1 ? Array.from({ length: 100 }, (_, id) => ({ tag_name: `other-${id}` })) : [draft] };
  });
  assert.deepEqual(result, draft);
  assert.equal(urls.length, 2);
  assert.match(urls[1], /page=2$/);
});
