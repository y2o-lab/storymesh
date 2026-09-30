---
name: storymesh-npm-release
description: Choose versions and prepare or perform npm releases of Storymesh through its GitHub tag workflow. Use for Storymesh npm versioning, release preparation, first publication, and release recovery.
---

# Storymesh npm release

Read `docs/npm-publishing.md` for the current commands and maintainer setup. Check the current manifests, scripts, workflow, Git remote, and npm registry before acting; the values in this skill do not replace live state. Creating this skill does not authorize a release. Only publish or push a release tag when the user's request authorizes that action.

## Version decision

- `storymesh` is the user-facing npm package and CLI. Five platform packages are optional dependencies. All six npm versions, the dependency versions, `Cargo.toml`, and the `storymesh` entry in `Cargo.lock` must match.
- Choose `MAJOR.MINOR.PATCH` from changes to the documented CLI behavior. Use a patch for compatible fixes, a minor for compatible features, and a major for incompatible changes after `1.0.0`. Before `1.0.0`, use a patch for fixes and a minor for features or intentional compatibility changes; do not claim a stability guarantee for `0.x`. Choose `1.0.0` when the maintainer is ready to define the stable interface. Honor an explicitly requested version.
- Each changed release needs a new version. Never reuse or move a published version's tag. The tag is `vX.Y.Z`; it does not rewrite package versions. A tag/version mismatch must fail validation.
- The current publisher runs `npm publish` without an explicit dist-tag, which makes `latest` the default. Do not use a prerelease version with this workflow until an appropriate npm dist-tag is implemented and checked for all six packages.

## Prepare and release

1. Determine whether this is the first npm publication. Check ownership and availability of all six package names. A registry lookup error other than a confirmed 404 is not proof of availability. For the first publication, follow the manual bootstrap in `docs/npm-publishing.md`; trusted publishing requires existing packages.
2. For a new version, run `node scripts/set-version.mjs X.Y.Z`, review the resulting manifest and lockfile diff, then run `mise run handoff`. Before tagging, run `node scripts/check-npm-version.mjs X.Y.Z`. Ensure the canonical GitHub repository matches every npm `repository.url` and each trusted publisher configuration.
3. Put the release commit on `main` and verify its CI result. When the request authorizes publication, create `vX.Y.Z` on that exact commit and push the tag. Only a tag push runs the npm publish job; a manual workflow run builds artifacts without publishing.
4. Inspect the Release npm workflow, all six registry versions and provenance, and a clean install and `storymesh --version` smoke test. Report the exact version, tag, commit, and any package that did not publish.

For a partial publish failure, rerun the same tag workflow only when the commit and package contents are unchanged; the script skips packages already at that version. If contents must change, prepare a new version and tag. Do not force-move a release tag.
