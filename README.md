# storymesh

English | [日本語ドキュメント](README.ja.md)

`storymesh` is a Rust CLI that checks whether components have corresponding Storybook story files and reports story coverage. It supports React, Preact, SolidJS, Qwik, Web Components, HTML, Ember, Vue, Svelte, and Angular.

Use it to:

- List components with missing stories.
- Generate minimal stories for components without them.
- View Storybook coverage as counts and percentages.
- Detect missing stories in CI.

## Supported frameworks

| Framework | Component files | `--framework` |
| --- | --- | --- |
| React | `.tsx`, `.jsx`, and PascalCase `.ts` / `.js` files | `react` |
| Preact | `.tsx`, `.jsx`, and PascalCase `.ts` / `.js` files | `preact` |
| SolidJS | `.tsx`, `.jsx`, and PascalCase `.ts` / `.js` files | `solid` |
| Qwik | `.tsx`, `.jsx`, and PascalCase `.ts` / `.js` files | `qwik` |
| Web Components / Lit | `.js` / `.ts` files with recognized custom-element registrations | `web-components` |
| HTML | explicitly named `*.component.html` fragments | `html` |
| Ember (experimental) | `.gjs`, `.gts`, and colocated `.hbs` under `app/components` | `ember` |
| Vue | `.vue` | `vue` |
| Svelte | `.svelte` | `svelte` |
| Angular | `*.component.ts` and `.ts` files with `@Component(...)` | `angular` |

## Installation

Install the npm package with npm, pnpm, Yarn, or Bun, then run the CLI in your project:

| Package manager | Install as a development dependency | Run |
| --- | --- | --- |
| [npm](https://docs.npmjs.com/cli/commands/npm-install) | `npm install --save-dev storymesh` | `npx storymesh check src/components --framework react` |
| [pnpm](https://pnpm.io/cli/add) | `pnpm add -D storymesh` | `pnpm exec storymesh check src/components --framework react` |
| [Yarn](https://yarnpkg.com/cli/add) | `yarn add -D storymesh` | `yarn run storymesh check src/components --framework react` |
| [Bun](https://bun.sh/docs/pm/cli/add) | `bun add -d storymesh` | `bunx storymesh check src/components --framework react` |

The Yarn example assumes a `node_modules` installation (Yarn Classic, or `nodeLinker: node-modules` in `.yarnrc.yml` for modern Yarn). Plug'n'Play compatibility has not been verified.

To run without adding a project dependency, choose one of these commands:

```sh
npx storymesh check src/components --framework react
pnpm dlx storymesh check src/components --framework react
bunx storymesh check src/components --framework react
```

See the [pnpm execution docs](https://pnpm.io/cli/exec), [pnpm dlx docs](https://pnpm.io/cli/dlx), [Yarn execution docs](https://yarnpkg.com/cli/run), and [bunx docs](https://bun.sh/docs/pm/bunx) for details. Bun also requires Node.js 18 or later for this CLI's Node.js launcher.

To install globally:

```sh
npm install --global storymesh
storymesh --help
```

## AI agent skill

Install the `storymesh` skill with `npx skills` to let an AI agent check story coverage, detect missing stories, and generate story skeletons when explicitly requested.

```sh
npx skills add y2o-lab/storymesh --skill storymesh
```

To install it in your project's Codex configuration without confirmation:

```sh
npx skills add y2o-lab/storymesh --skill storymesh --agent codex --yes
```

The skill lives in [skills/storymesh/SKILL.md](skills/storymesh/SKILL.md) and is distributed directly from the public GitHub repository; no npm bundle or separate registry upload is needed. Add `--global` for a global installation.

Before publishing a skill update, run `npx skills@latest add . --list` and `mise run handoff`, merge to `main`, then check discovery with `npx skills@latest add y2o-lab/storymesh --list`. The `agent-skills` GitHub topic can help discovery. If tagging a skill release, use `skills-vX.Y.Z`; `v*` tags trigger npm staging.

The npm distribution supports glibc Linux x64/ARM64, macOS x64/ARM64, and Windows x64. Node.js 18 or later is required.

Windows x64 npm releases use `storymesh-windows-x64`. All five platform packages must have the matching release version published before approving `storymesh`.

To build from source, install [mise](https://mise.jdx.dev/) and run these commands in your checkout:

```sh
mise install
mise exec -- cargo build --release
./target/release/storymesh --help
```

The examples below use the built binary at `./target/release/storymesh`. During development, you can use `mise exec -- cargo run --` instead.

## Quick start

Check `src/components` in a React project:

```sh
./target/release/storymesh check src/components --framework react
```

If any components lack stories, the command lists their files and exits with code `1`:

```text
Missing stories for 1 React component(s):
Card.tsx
```

If every component has a story, the exit code is `0`:

```text
All 3 React components have stories.
```

## Commands

### `check`

List components with missing stories. Use this command to detect missing stories in CI.

```sh
./target/release/storymesh check [PATH] [--framework react|preact|solid|qwik|web-components|html|ember|vue|svelte|angular] [--ignore PATTERN] [--ignore-file PATH] [--generate]
```

With `--generate`, the command creates a minimal [Component Story Format (CSF)](https://storybook.js.org/docs/api/csf) story next to each component detected as missing a story:

```sh
./target/release/storymesh check src/components --framework react --generate
```

```text
Missing stories for 1 React component(s):
Card.tsx
Generated 1 story skeleton(s):
Card.stories.tsx
```

With `--generate`, the exit code indicates whether generation succeeded rather than whether stories were missing. It returns `0` if generation succeeds, including when there is nothing to generate, or `2` if generation fails. Running `check` again counts the generated stories as covered. Existing files are never overwritten.

React stories use the component's extension (for example, `Button.tsx` → `Button.stories.tsx`); Vue, Svelte, and Angular stories use `.stories.ts`. For React, the generator recognizes default exports and common named exports matching the filename. If it cannot find an importable export, it creates a `render: () => null` placeholder you can edit in Storybook. Vue and Svelte use default imports with the component file extension, and Angular assumes a conventional class name (for example, `user-card.component.ts` → `UserCardComponent`). Adjust the generated imports if your project's exports differ.

The `preact` mode uses the same filename rules, story matching, and story extension as React. For example, run `storymesh check src/components --framework preact --generate`. It generates imports for default exports or named declarations matching the component name; export lists and re-exports are not recognized. If neither is recognized, generation fails with an instruction to write the story manually. Export detection is heuristic, so inspect generated imports. JSX utilities can be counted as components; use `--ignore` or an ignore file to exclude them. Files with the same stem in one directory share a story match even when their extensions differ. The [Preact fixture](tests/fixtures/preact/README.md) records the versions and JSX configuration used to build and render simple generated stories; it does not establish compatibility with every Storybook configuration or component requiring providers.

The `solid` mode uses the same filename and matching rules and the same export detection as `preact`, including its error for unrecognized exports and the limitations described above. For example, run `storymesh check src/components --framework solid --generate`. JSX utilities can be counted and should be excluded with ignore rules if needed. The [SolidJS fixture](tests/fixtures/solid/README.md) records the integration and versions used to build and render simple generated stories; provider-dependent components and other configurations are not covered by that verification.

The `qwik` mode uses the same filename, matching and export rules as `preact`. For example, run `storymesh check src/components --framework qwik --generate`. It supports simple `component$` declarations, but cannot infer providers or identify every wrapper or re-export. Ignore non-component JSX modules and inspect generated imports. The [Qwik fixture](tests/fixtures/qwik/README.md) records the optimizer configuration, integration versions and simple rendering checks; Qwik City context is not covered.

Use `storymesh check src/components --framework web-components --generate` for JS/TS modules containing `customElements.define(tag, ...)` or Lit's `@customElement(tag)`. Story matching uses the component filename stem and the normal story directories and extensions. Test/spec/story files, declaration files, ignored paths and generated/dependency directories are excluded. A file with multiple registrations is counted once; files with the same stem and different extensions share a match. Generation requires exactly one registration with a literal, unescaped, lowercase ASCII custom-element tag containing a hyphen. Dynamic, invalid or multiple tags and `define` calls with a third argument cause an actionable error before writing; write those stories manually. Filenames containing `?` or `#` require a manual story because those characters are module query or fragment delimiters. Generated stories import the component module for registration and create the element. They may need manual attributes or context.

Detection is lexical, not a JavaScript or TypeScript parser: it skips comments, strings and template literals, and recognizes the names above without resolving bindings. Aliases, registrations inside template interpolation, indirect calls and other registries are not detected; shadowed names or unusual regex syntax can cause false positives. A registration behind a condition may not run when imported. Review generated stories and use ignore rules for non-components. The [Web Components fixture](tests/fixtures/web-components/README.md) records the versions and simple native-element and Lit rendering verified; it is not a guarantee for every builder or customized built-in element.

For HTML, explicitly name component fragments `*.component.html` and run `storymesh check src/components --framework html --generate`. Ordinary `.html` pages and names ending in `.test`, `.spec`, `.story` or `.stories` before `.component.html` are excluded, as are ignored and generated/dependency paths. A fragment matches a story with the same full stem, for example `button.component.stories.js`, including in supported story directories. `spec init` rejects duplicate generated titles, such as two `card.component.html` files in different directories. Filenames containing `?` or `#` require a manual story. The generated JS story imports the fragment with Vite's `?raw` and returns it as markup. Other builders require equivalent raw-import configuration and have not been verified. Relative assets, scripts, templating and required context may need manual adaptation; this convention does not infer components from arbitrary pages. See the [HTML fixture](tests/fixtures/html/README.md) for the verified versions and rendering scope.

For experimental Ember support, run `storymesh check app/components --framework ember --generate`. The initial convention counts `.gjs`, `.gts` and `.hbs` beneath `app/components`; a colocated `.js` or `.ts` class and its `.hbs` template count once, using the template path. Test/story suffixes and ignored paths are excluded. Multiple template sources with the same stem, or both JS and TS classes for one template, are reported as ambiguous; remove or ignore the conflicting source. A story such as `card.stories.js` matches `card.gjs`, `card.gts` or `card.hbs` beside it or in a supported story directory. Generated stories use `ember-cli-htmlbars` and the app resolver (`nested/user-card.gjs` becomes `<Nested::UserCard />`); unsupported names or `index` paths require a manual story. JS/TS-only components, legacy `app/templates/components`, addon layouts and template-local named exports are not detected. Files under `app/components` can include non-components, so use ignore rules as needed. Generated stories may require arguments, services or other context. See the [Ember fixture](tests/fixtures/ember/README.md) for the pinned integration and verified scope; `.gts` requires a TypeScript-enabled Ember build.

For React Native Web, use `react`, for example `storymesh check src/components --framework react --ignore '*.ios.tsx' --ignore '*.native.tsx'`. Platform suffixes are part of the component name: `Button.web.tsx` matches `Button.web.stories.tsx`, not `Button.stories.tsx`. Files with the same stem and different extensions share a match. The generator imports `./Button.web` for a default export, but a named export `Button` in `Button.web.tsx` is not recognized and produces the React placeholder; edit it manually. When `Button.tsx` and `Button.web.tsx` coexist, the tested Web builder resolves the generated `./Button` import to the Web variant, so the generic component's story can render the wrong file. Inspect and adjust imports for your builder, or exclude variants that should not be checked. The [React Native Web fixture](tests/fixtures/react-native-web/README.md) documents the versions and browser rendering verified. This does not establish native device compatibility, behavior of other resolvers, or support for native-only modules. `spec init` has no ignore options; restrict its scan path or use a `.storymeshignore` file to exclude platform variants.

For React Native on-device projects, the existing `react` mode likewise treats `.ios`, `.android`, and `.native` as part of the component name. The [on-device fixture](tests/fixtures/react-native/README.md) records pinned Expo / Metro bundle checks and the remaining device verification. In the tested Metro exports, a generic `./Button` import resolves to the iOS or native variant when those files coexist; platform-specific story files are also bundled on the other platform unless excluded from the Storybook configuration. Inspect the generated stories and use appropriate platform-specific scans and story globs. Native runtime registration and rendering have not been verified.

For Next.js, use `react`, for example `storymesh check app/components --framework react --generate`. Detection is based on filenames, not Next.js semantics: App Router `page.tsx`, `layout.tsx`, `loading.tsx` and other TSX special files are counted when included in the scan, even if they are not intended as Storybook components. A `route.ts` file is not counted by its lowercase name. Restrict the path or use ignore rules; `spec init` can use `.storymeshignore` and rejects duplicate titles such as two `page.tsx` files. Story matching uses the same stem and supported story directories as React, and existing stories are preserved. Generated imports and placeholders follow the React rules above and may need context or edits. The [Next.js fixture](tests/fixtures/nextjs/README.md) records the pinned integration and simple browser rendering, including an async component with Storybook's experimental RSC option. It does not verify server rendering or server-only dependencies; those may need mocks or a separate client-safe component.

For TanStack React, use `react`, for example `storymesh check src/components --framework react --generate`. The same filename rules also count files such as `src/routes/__root.tsx` and `about.tsx` if routes are scanned; an `index.tsx` under a lowercase route directory and lowercase `routeTree.gen.ts` are not counted. This does not inspect route semantics, so a route object is not necessarily a component. A typical route exporting `Route` from `about.tsx` generates a null placeholder rather than a working route story; filenames matching an exported route object can also produce an unsuitable import. Review generated stories. Restrict the scan or use `--ignore 'routes/**'` when scanning `src`; `spec init` can use `.storymeshignore` and rejects duplicate titles. Existing stories are preserved and filename-stem matching follows React rules. The [TanStack fixture](tests/fixtures/tanstack-react/README.md) records the pinned integration, router `Link` rendering and verification limits; route context, loaders and server functions may require explicit Storybook configuration or mocks.

For Nuxt, use `vue`, for example `storymesh check components --framework vue --generate`. Every `.vue` file is a candidate except test/spec/story suffixes and ignored paths, so scanning the project also counts `pages` and `layouts`; generated `.nuxt` files can be counted if they are not ignored. Restrict the scan or use ignore rules (for example `--ignore 'pages/**' --ignore 'layouts/**' --ignore '.nuxt/**'` from the project root). `spec init` can use `.storymeshignore` and rejects duplicate component titles. Story matching and non-overwrite behavior follow the Vue rules; generated stories import the SFC, but may require slots, route data or other context. The [Nuxt fixture](tests/fixtures/nuxt/README.md) records the pinned integration and browser checks, including `useRuntimeConfig`; it does not establish server rendering or compatibility with every plugin and Nuxt configuration.

For SvelteKit, use `svelte` on a component directory, for example `storymesh check src/lib --framework svelte --generate`. Scanning `src` also counts `+page.svelte` and `+layout.svelte`; test/spec/story suffixes, ignored paths and `.svelte-kit` are excluded. **Do not generate stories beside SvelteKit routes:** the generated `+page.stories.ts` and `+layout.stories.ts` are reserved `+` filenames and cause the tested SvelteKit build to fail. Restrict the path or use `--ignore 'routes/**'` when scanning `src`; `spec init` can use `.storymeshignore` and rejects `+page` and `+layout` as v1 IDs. Existing stories are preserved and the normal Svelte filename matching rules apply. The [SvelteKit fixture](tests/fixtures/sveltekit/README.md) records the pinned integration and browser checks; route data, server-only modules and other app context can require manual configuration.

For Analog, use `angular`, for example `storymesh check src/app/components --framework angular --generate`. The scanner counts `*.component.ts` by filename and other `.ts` files containing a literal `@Component(...)` outside comments and strings, including Analog `.page.ts` routes. It does not resolve imports: an aliased `@NgComponent` in a suffix-less file is missed and an unrelated decorator with that spelling can be counted. Test/spec/story suffixes and ignored paths are excluded. The generator guesses a named class from the filename; Analog's default-exported page components and differently named classes require a manual import. For example, an `index.page.ts` default export produces a nonexistent named `IndexPage` import. Restrict the path or ignore pages when appropriate, and inspect generated stories. Story matching and non-overwrite behavior follow Angular rules; `spec init` rejects duplicate titles. The [Analog fixture](tests/fixtures/analog/README.md) records the pinned Angular / Vite Storybook integration and browser rendering; it does not verify Analog routing or SSR.

### `spec check` / `spec init` / `spec import`

Declare required UI states in YAML and compare them with stories registered by Storybook. `check` looks for story **files**, while `spec check` looks for story entries in Storybook's `index.json`. Generate a fresh index from the current checkout of your Storybook project each time.

```sh
pnpm exec storybook index -o storybook-static/index.json
./target/release/storymesh spec init src/components --framework react
./target/release/storymesh spec init src/components --framework react --with-stories
./target/release/storymesh spec import --index storybook-static/index.json
./target/release/storymesh spec import --index storybook-static/index.json --merge
./target/release/storymesh spec check --spec-dir .storymesh/specs --index storybook-static/index.json
```

`spec init` requires `--framework react|preact|solid|qwik|web-components|html|ember|vue|svelte|angular` and creates YAML drafts from components in `.storymesh/specs`. Drafts normally contain `stories: []`; add the required UI states before running `spec check`. With `--with-stories`, components without story files receive a CSF `Default` story and YAML declaring that story as required. Existing story files are not edited. Regenerate the index to confirm that Storybook registers the generated stories.

`spec import` creates YAML files grouped by title from the stories in the index. It skips existing specs without overwriting them. `--merge` appends only new stories to an existing spec with an exactly matching title. To preserve comments and existing `required` values, updates are limited to simple v1 block-style `stories` lists, with either unindented or two-space-indented list items. This includes YAML generated by `spec import` and `spec init --with-stories` for all supported frameworks. Appended entries retain the existing list indentation. Anchors, aliases, flow-style lists, and multiple documents are rejected.

Example YAML v1 spec:

```yaml
version: 1
component:
  id: button
  name: Button
  title: Components/Button
stories:
  - id: default
    required: true
  - id: loading
    required: true
  - id: debug
    required: false
```

`component.title` must exactly match the title in the index. If omitted, the command looks for a title matching `name` or a title whose last segment matches it; multiple matches cause an error. To match stories, it converts the index entry's `name` to lowercase and replaces sequences of spaces, `_`, and `-` with `-` to form an ID. Renaming a story changes the match. `required: false` does not require the story to exist and does not produce a warning if it does exist.

`spec check` reports `PASS` and exits with `0` when all required stories exist, or `FAIL` and `1` when any are missing. Undeclared stories produce `WARNING`; warnings alone still result in `PASS` and `0`. Missing or invalid YAML or index files, ambiguous matches, and empty drafts result in exit code `2` without `PASS`. This command does not report percentages. It checks Storybook registration only; it does not verify rendering, args, interaction results, or scenario correctness.

### `coverage`

Display coverage as a percentage and count:

```sh
./target/release/storymesh coverage src/components --framework vue
```

```text
Vue Storybook coverage: 83.3% (5/6 components)
```

### `report`

Display both coverage and components with missing stories:

```sh
./target/release/storymesh report src/app --framework angular
```

```text
Angular Storybook coverage: 83.3% (5/6 components)
Missing: 1
profile.ts
```

If `PATH` is omitted, the command checks the current directory. For `check`, `coverage`, and `report`, `--framework` defaults to `react`.

### Excluding files

Repeat `--ignore` to exclude paths from scanning. Patterns are interpreted relative to the scan root.

```sh
./target/release/storymesh check src --ignore 'generated/**' --ignore '**/*.fixture.tsx'
```

The command automatically reads `.storymeshignore` in the scan root. It uses `.gitignore` syntax, including blank lines, `#` comments, `!` for re-inclusion, `*` / `**`, and trailing `/`.

```gitignore
# generated components are not maintained by this repository
generated/
**/*.fixture.tsx
!generated/DocumentedButton.tsx
```

Repeat `--ignore-file` to add other ignore files. Relative paths are resolved against the scan root.

Use the following commands for manual checks in the React, Vue, and Angular development test apps. The regular `storymesh:check` deliberately reports one uncovered fixture; the other two commands succeed by excluding it with `--ignore` and `--ignore-file`, respectively.

```sh
cd .storymesh-test-apps/react-app
pnpm storymesh:check
pnpm storymesh:ignore-pattern
pnpm storymesh:ignore-file

cd ../vue-app
pnpm storymesh:check
pnpm storymesh:ignore-pattern
pnpm storymesh:ignore-file

cd ../angular-app
pnpm storymesh:check
pnpm storymesh:ignore-pattern
pnpm storymesh:ignore-file
```

## Exit codes

| Code | Meaning |
| --- | --- |
| `0` | Success. No missing requirements for `check` / `spec check`, or successful generation for generation commands |
| `1` | `check` detected components without stories, or `spec check` detected missing required stories |
| `2` | An error occurred with input, ambiguous matching, path reads, output, or another operation |

`coverage`, `report`, and successful `check --generate` commands exit successfully even when stories are missing. To make missing stories fail CI, use `check` without `--generate`.

## Detection rules

### Common rules

- Stories are searched for beside the component or in an immediate `stories` / `__stories__` subdirectory.
- Supported story extensions are `.js`, `.jsx`, `.mjs`, `.cjs`, `.ts`, `.tsx`, and `.svelte`.
- The scanner skips `.git`, `.next`, `.storybook`, `.svelte-kit`, `build`, `coverage`, `dist`, `node_modules`, and `target` directories.
- Files excluded by `.storymeshignore`, `--ignore`, or `--ignore-file` count as neither components nor stories.
- `*.test.*`, `*.spec.*`, and story files themselves do not count as components.

### React

- `.tsx` / `.jsx` files count as components. Lowercase `main.tsx` / `main.jsx` files are excluded as entry points.
- `.js` / `.ts` files with PascalCase filenames (for example, `Button.js`) count as components.
- `*.d.ts` files are excluded.
- `Button.tsx` matches a story with the same name, such as `Button.stories.tsx`.
- The `Button/index.tsx` and `Button/Button.stories.tsx` layout is also supported.

### Vue

- `.vue` files count as components.
- `Button.vue` matches a story with the same name, such as `Button.stories.ts`.
- The `Button/index.vue` and `Button/Button.stories.ts` layout is also supported.

### Svelte

- `.svelte` files count as components, including lowercase filenames.
- `Button.svelte` matches `Button.stories.ts`, `Button.stories.js`, or native `Button.stories.svelte` files (and the other supported story extensions).
- The `Button/index.svelte` and `Button/Button.stories.svelte` layout is also supported, including `stories` / `__stories__` subdirectories.
- Native stories use [Storybook's Svelte CSF addon](https://storybook.js.org/addons/@storybook/addon-svelte-csf). Generation uses standard CSF `.stories.ts` files with a default import from `./Button.svelte`.

```sh
./target/release/storymesh check src/lib --framework svelte
./target/release/storymesh check src/lib --framework svelte --generate
./target/release/storymesh spec init src/lib --framework svelte --with-stories
```

### Angular

- `*.component.ts` files count as components.
- Files such as `app.ts`, generated using Angular's newer naming convention, are detected by a `@Component(...)` decorator in code, excluding comments and strings.
- `button.component.ts` matches `button.stories.ts` or `button.component.stories.ts`.
- A component without the `.component` suffix, such as `profile.ts`, matches `profile.stories.ts`.

## Known limitations

`storymesh` primarily uses paths and filenames. It does not fully parse Storybook CSF or framework ASTs.

- Stories with names that differ from their components are not matched.
- MDX documentation does not count toward coverage.
- Non-component React `.jsx` / `.tsx` files, and Vue or Svelte pages or layouts (including SvelteKit `+page.svelte` / `+layout.svelte`), may count as components.
- Components without the `.component` suffix that import Angular's `Component` under an alias are not detected.

## Development

Run development commands through `mise`. Harness checks also require `jq`.

```sh
mise run quick    # rustfmt + tests
mise run handoff  # final gate selected from the diff
mise run verify   # harness + rustfmt + Clippy + tests
mise run format
mise run lint
mise run test
mise run check
mise run npm-check
```

### Codex harness

Use `scripts/codex-task fast`, `standard`, or `deep` for mechanical changes, normal development, or complex work. The wrapper preserves the selected model; add `--exec` for non-interactive execution. Project configuration and hooks require a trusted repository; use `/hooks` in Codex CLI after cloning or changing hooks.

```sh
scripts/codex-task standard --json-log .codex-runs/task.jsonl \
  "Implement the accepted task and run mise run handoff"
scripts/codex-metrics .codex-runs/task.jsonl
```

Logs are ignored by Git. Compare uncached input, output, reasoning tokens, and cache-hit rates across similar tasks. The SessionStart hook reports dirty paths; the Stop hook checks verification evidence in `.codex-runs/`. Later edits invalidate that evidence. Gate selection is defined in the [handoff skill](.agents/skills/storymesh-handoff/SKILL.md).

### npm releases

Maintainer setup, first-release staging, version selection, tag publishing, and recovery are documented in the [npm release skill](.agents/skills/storymesh-npm-release/SKILL.md). The [release workflow](.github/workflows/release.yml) defines build and publishing behavior.

To synchronize release versions automatically, run **Prepare npm release** in Actions on `main` and enter a version without `v` (for example `0.3.0`). The workflow updates Cargo.toml, Cargo.lock, all six npm packages and their platform dependency versions, runs verification, and pushes a new `release/vVERSION` branch. Open a pull request from that branch, merge after CI succeeds, then tag the merged commit as `vVERSION`. Existing tags and release branches are rejected; this workflow does not publish packages or create tags. The repository must allow Actions to write contents. CI explicitly checks version synchronization on every PR and main push; tag releases additionally check that the tag matches the manifests. These checks detect inconsistent versions, not whether a particular feature deserves a version bump.

Both tag and bootstrap staging submit all five platform packages, including `storymesh-windows-x64`, followed by the CLI package. npm support cleared the Windows package name block and transferred write access after publishing `0.0.1-security` placeholders for `storymesh-windows-x64` and `storymesh-win32-x64`. Storymesh uses only `storymesh-windows-x64`.

For the first Windows release after the unblock, choose a new version that has never been attempted for that package; npm support confirmed that previously attempted versions cannot be reused, even if they are absent from the registry. Keep all six npm packages and the Rust version in sync as described in the release skill. Configure the trusted publisher for `storymesh-windows-x64` with owner `y2o-lab`, repository `storymesh`, workflow `release.yml`, environment `npm`, and stage-only permission before tag staging. If bootstrapping with a temporary token, ensure it includes write access to this existing package. Rerunning an old tag uses that tag's workflow and retains its Windows exclusion; do not move release tags to change this behavior.

Tag runs stage the included packages through npm trusted publishing. After successful staging, the workflow creates a draft GitHub Release named `vVERSION` targeting that tag's commit. Review and approve the platform packages on npmjs.com with 2FA before approving `storymesh`. Verify all included versions and dist-tags, then publish the draft GitHub Release manually. Build-only manual runs and failed staging do not create releases. A rerun cannot query pending stages through OIDC; approve any already staged packages before retrying a partial run. Reruns reuse an existing draft, and prerelease versions are marked as GitHub prereleases.

Release notes use GitHub's automatic generation: merged pull requests, contributors, and a full changelog link. Write descriptive PR titles because they appear in the notes; changes pushed without a PR may only be visible in the full changelog. Review the draft to add highlights, migration instructions, or installation guidance. GitHub does not generate those explanations from the code diff.

### Dependency updates

Renovate manages dependency update PRs. Repository administrators should install the [Renovate GitHub App](https://github.com/apps/renovate) for this repository. Configuration lives in [renovate.json](renovate.json) and covers Rust (Cargo), npm, and GitHub Actions updates.
