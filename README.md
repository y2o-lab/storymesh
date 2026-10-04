# storymesh

English | [日本語ドキュメント](README.ja.md)

`storymesh` is a Rust CLI that checks whether components have corresponding Storybook story files and reports story coverage. It supports React, Vue, and Angular.

Use it to:

- List components with missing stories.
- Generate minimal stories for components without them.
- View Storybook coverage as counts and percentages.
- Detect missing stories in CI.

## Supported frameworks

| Framework | Component files | `--framework` |
| --- | --- | --- |
| React | `.tsx`, `.jsx`, and PascalCase `.ts` / `.js` files | `react` |
| Vue | `.vue` | `vue` |
| Angular | `*.component.ts` and `.ts` files with `@Component(...)` | `angular` |

## Installation

Install with npm in your project, or run the CLI with npx:

```sh
npm install --save-dev storymesh
npx storymesh check src/components --framework react
```

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

See the [AI agent skill distribution and publishing guide](docs/skills-publishing.md) for details (in Japanese).

The npm distribution supports glibc Linux x64/ARM64, macOS x64/ARM64, and Windows x64. Node.js 18 or later is required.

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
./target/release/storymesh check [PATH] [--framework react|vue|angular] [--ignore PATTERN] [--ignore-file PATH] [--generate]
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

React stories use the component's extension (for example, `Button.tsx` → `Button.stories.tsx`); Vue and Angular stories use `.stories.ts`. For React, the generator recognizes default exports and common named exports matching the filename. If it cannot find an importable export, it creates a `render: () => null` placeholder you can edit in Storybook. Vue assumes a default export, and Angular assumes a conventional class name (for example, `user-card.component.ts` → `UserCardComponent`). Adjust the generated imports if your project's exports differ.

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

`spec init` requires `--framework react|vue|angular` and creates YAML drafts from components in `.storymesh/specs`. Drafts normally contain `stories: []`; add the required UI states before running `spec check`. With `--with-stories`, components without story files receive a CSF `Default` story and YAML declaring that story as required. Existing story files are not edited. Regenerate the index to confirm that Storybook registers the generated stories.

`spec import` creates YAML files grouped by title from the stories in the index. It skips existing specs without overwriting them. `--merge` appends only new stories to an existing spec with an exactly matching title. To preserve comments and existing `required` values, updates are limited to simple v1 block-style `stories` lists. Anchors, aliases, flow-style lists, and multiple documents are rejected.

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
- Supported story extensions are `.js`, `.jsx`, `.mjs`, `.cjs`, `.ts`, and `.tsx`.
- The scanner skips `.git`, `.next`, `.storybook`, `build`, `coverage`, `dist`, `node_modules`, and `target` directories.
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

### Angular

- `*.component.ts` files count as components.
- Files such as `app.ts`, generated using Angular's newer naming convention, are detected by a `@Component(...)` decorator in code, excluding comments and strings.
- `button.component.ts` matches `button.stories.ts` or `button.component.stories.ts`.
- A component without the `.component` suffix, such as `profile.ts`, matches `profile.stories.ts`.

## Known limitations

`storymesh` primarily uses paths and filenames. It does not fully parse Storybook CSF or framework ASTs.

- Stories with names that differ from their components are not matched.
- MDX documentation does not count toward coverage.
- Non-component React `.jsx` / `.tsx` files, and Vue pages or layouts, may count as components.
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

See [docs/codex-harness.md](docs/codex-harness.md) for the Codex development harness guide, and the [npm publishing guide](docs/npm-publishing.md) for maintainer release instructions (both in Japanese).

### Dependency updates

Renovate manages dependency update PRs. Repository administrators should install the [Renovate GitHub App](https://github.com/apps/renovate) for this repository. Configuration lives in [renovate.json](renovate.json) and covers Rust (Cargo), npm, and GitHub Actions updates.
