---
name: storymesh-npm-release
description: Choose versions and prepare or perform npm releases of Storymesh through its GitHub tag workflow. Use for Storymesh npm versioning, release preparation, first publication, and release recovery.
---

# Storymesh npm release

Check the current manifests, scripts, workflow, Git remote, and npm registry before acting; the values in this skill do not replace live state. Creating this skill does not authorize a release. Only publish or push a release tag when the user's request authorizes that action.

## Version decision

- `storymesh` is the user-facing npm package and CLI. Five platform packages are optional dependencies. All six npm versions, the dependency versions, `Cargo.toml`, and the `storymesh` entry in `Cargo.lock` must match.
- Choose `MAJOR.MINOR.PATCH` from changes to the documented CLI behavior. Use a patch for compatible fixes, a minor for compatible features, and a major for incompatible changes after `1.0.0`. Before `1.0.0`, use a patch for fixes and a minor for features or intentional compatibility changes; do not claim a stability guarantee for `0.x`. Choose `1.0.0` when the maintainer is ready to define the stable interface. Honor an explicitly requested version.
- Each changed release needs a new version. Never reuse or move a published version's tag. The tag is `vVERSION`; it does not rewrite package versions. A tag/version mismatch must fail validation.
- For prereleases use `X.Y.Z-{alpha|beta|rc|canary}.N`, with `N` starting at 1 and increasing for each changed release. Use canary for frequent development builds, alpha for early testing, beta for user trials, and rc for a final candidate. The publisher explicitly applies the matching npm dist-tag to all six packages; stable `X.Y.Z` uses `latest`. Reject other prerelease spellings until the release scripts support them.

## Maintainer setup and first publication

The canonical GitHub repository is `y2o-lab/storymesh`; the registry is `https://registry.npmjs.org`. Confirm the Git remote and all six `repository.url` fields match the canonical repository before releasing. The current package names are `storymesh`, `storymesh-darwin-arm64`, `storymesh-darwin-x64`, `storymesh-linux-arm64`, `storymesh-linux-x64`, and `storymesh-win32-x64`; verify these against the manifests.

1. Enable 2FA on the npm account with publication rights. Check ownership or availability of all six package names; only a confirmed E404 means a name is unpublished.
2. In GitHub Settings → Environments, create `npm`. This is an Environment name, not an environment variable. If restricting deployment refs, allow `main` for bootstrap and `v*` tags for subsequent releases. Required reviewers are optional.
3. On npmjs.com, open the profile → Access Tokens → Generate New Token. Create a short-lived granular token with **Read and write (stage only)**, **Bypass two-factor authentication disabled**, and **All Packages** to create the new unscoped packages. Store it only as the GitHub `npm` Environment secret **NPM_TOKEN**. Local `npm login` is not needed for this bootstrap.
4. Prepare the version using the commands below, merge the release commit to `main`, and verify CI. In Actions → **Release npm** → Run workflow, select `main` and enable **bootstrap_stage**. Default manual runs only build artifacts. The bootstrap job requires Node.js 22.14.0+ and npm CLI 11.15.0+ (the workflow uses Node.js 24), has no OIDC write permission, and passes the token only to the staging step.
5. Review version, dist-tag, and tarball contents in npmjs.com's **Staged Packages**. Confirm package contents include the intended manifests, README, LICENSE, launcher files or platform binary, without unwanted files. Approve the five platform packages first with 2FA, then approve `storymesh`. A successful staging workflow is not a completed publication. Staging a new package creates a publicly visible `0.0.0-stage` placeholder; the release contents remain private until approval.
6. In Settings → Trusted publishing on **every package**, add a GitHub Actions publisher with Organization or user **y2o-lab**, Repository **storymesh**, Workflow filename **release.yml**, Environment name **npm**, and permission for **npm publish**. The owner is the GitHub owner, not the npm login name. Verify all fields: saving a publisher does not validate them.
7. Set every package's Publishing access to **Require two-factor authentication and disallow tokens**; OIDC still works. Revoke the bootstrap token on npm and delete the GitHub **NPM_TOKEN** secret. Normal tag publishing uses OIDC without an npm token. It requires Node.js 22.14.0+ and npm CLI 11.5.1+ on GitHub-hosted runners. Public packages from a public repository receive automatic provenance.
8. Verify all six versions and dist-tags and run a clean-install smoke test. When authorized, create `vVERSION` on the exact bootstrap commit and push it; the workflow skips the already published versions.

After an authorized publication, run this in a new temporary directory and verify the reported version:

```sh
npm exec --yes --package=storymesh@VERSION -- storymesh --version
```

Configuration references: [npm tokens](https://docs.npmjs.com/creating-and-viewing-access-tokens/), [staged publishing](https://docs.npmjs.com/staged-publishing/), [Trusted Publishing](https://docs.npmjs.com/trusted-publishers/), and [GitHub Environments](https://docs.github.com/en/actions/reference/workflows-and-actions/deployments-and-environments).

## Prepare and release

1. Determine whether this is the first npm publication. Check ownership and availability of all six package names. A registry lookup error other than a confirmed 404 is not proof of availability. For the first publication, follow the maintainer setup and first publication section above; trusted publishing requires existing packages.
2. For a new version, run `node scripts/set-version.mjs VERSION`, review the resulting manifest and lockfile diff, then run `mise run handoff`. Before tagging, run `node scripts/check-npm-version.mjs VERSION`. Ensure the canonical GitHub repository matches every npm `repository.url` and each trusted publisher configuration.
3. Put the release commit on `main` and verify its CI result. When the request authorizes publication, create `vVERSION` on that exact commit and push the tag. Only a tag push runs the npm publish job; a default manual workflow run only builds artifacts. On main, explicitly enabling bootstrap_stage submits packages for npm web 2FA approval using the temporary NPM_TOKEN Environment secret; it never directly publishes. Revoke the bootstrap token and remove the secret after configuring all six trusted publishers.
4. Inspect the Release npm workflow, all six registry versions, their npm dist-tags and provenance, and a clean install and `storymesh --version` smoke test. Report the exact version, tag, commit, and any package that did not publish.

For a partial publish failure, rerun the same tag workflow only when the commit and package contents are unchanged; the script skips packages already at that version. For a partial bootstrap staging failure, rerun the manual bootstrap on the same unchanged main commit; already staged or published versions are skipped. If contents must change, prepare a new version and tag; reject obsolete staged versions on npmjs.com. A tag workflow build failure that needs code changes also requires a new commit and version. Do not force-move a release tag.
