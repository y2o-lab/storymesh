# Framework support roadmap

Status: roadmap. Phases 1–3 and 5–7 have shipped modes (Ember is experimental); phases 4 and 8 remain proposals. See the [README](../README.md) for supported modes. Research snapshot: 2026-10-31. Priorities are based on implementation dependencies and verifiability, not measured adoption. Deliver each phase as a separate, reversible change; reconsider the order when user demand or upstream compatibility changes.

## Current baseline

`check`, `coverage`, `report`, and `spec init` select one of React, Preact, SolidJS, Qwik, Web Components, HTML, Ember, Vue, Svelte, or Angular. Matching is primarily filename- and path-based, not an AST or Storybook registration check. `check --generate` and `spec init --with-stories` share the story skeleton generator; `spec import` and `spec check` consume a Storybook index and do not select a framework. See [README](../README.md#detection-rules) and [CLI options](../src/cli/args.rs).

React currently treats most `.jsx` / `.tsx` files and PascalCase `.js` / `.ts` files as components; it may produce false positives, and its generator falls back to a `render: () => null` placeholder when it cannot identify an export. SvelteKit pages and layouts can also be counted. New modes must document comparable limitations rather than claim semantic detection.

## Common delivery contract

For each mode, define candidate extensions, component conventions, story matching, exclusions, generated CSF, and ambiguous cases before implementation. Extend the domain enum, CLI, detector and generator only where needed; preserve existing defaults, exit codes, matching and no-overwrite behavior. Keep framework-specific behavior separate even where low-level helpers are shared. Do not add dependencies without a demonstrated need.

Each delivery includes behavior tests for covered/missing components, non-components, story extensions and directory layouts, ignored paths, generation and existing-file protection, and spec initialization. Exercise `check`, `coverage`, `report`, `check --generate`, and `spec init --with-stories` through the CLI. Test generated stories against a pinned, minimal real Storybook fixture: build or index them and verify `Default` registration; a successful index alone does not prove rendering. Record package versions, run instructions, and any manual rendering check. Run focused tests, affected CLI commands, then `mise run handoff`; review the final diff. Document supported versions, conventions, limitations and examples in README. If real integration cannot be validated, do not claim full support or silently generate a misleading story.

## Ordered phases

1. **Preact** — Add `preact` mode for `.jsx` / `.tsx` and conventional JS/TS components. Reuse filename and export-detection helpers where their behavior applies, not React's branding. Verify generated CSF with a pinned Preact Storybook fixture, including named/default exports and unknown exports. Decide whether a placeholder is acceptable after checking renderer behavior; test it explicitly.
2. **SolidJS** — Add `solid` with analogous JSX candidates, checking Solid's renderer and CSF examples rather than assuming React compatibility. Verify `Default` registration and rendering, named/default exports, and JSX utilities incorrectly identified as components. Keep Preact and Solid independently testable.
3. **Qwik** — Add `qwik` for conventional JSX/TSX components, including a `component$` example and named/default exports. Pin the community integration and validate the skeleton with it; establish behavior for wrappers and exports that cannot be identified. Avoid requiring Qwik-specific parsing unless fixtures show it is necessary.
4. **React Native and React Native Web** — First verify normal React components using the existing `react` mode with pinned native and web fixtures. Test platform-specific names such as `Button.ios.tsx` and `Button.native.tsx`, story matching and generator import resolution; define collision behavior before changing matching. Add a dedicated mode only if platform-aware behavior cannot be expressed safely in the existing mode. Native on-device validation may require a separate documented environment; do not claim it from a web-only test.
5. **Web Components / Lit** — Design `web-components` around a narrowly specified, testable convention for `.js` / `.ts` custom-element definitions, including Lit. Determine whether a tag can be reliably extracted from `customElements.define` or decorators without matching comments and strings; test unrecognized and dynamic registrations. CSF generation must register the element and use its tag as required by the renderer; if the tag cannot be resolved, return an actionable error or a clearly documented, validated placeholder. Avoid treating arbitrary JS modules as elements.
6. **HTML** — Define an explicit convention or opt-in for component files (for example, a specified component directory and `.html` files); arbitrary HTML pages are not necessarily components. Validate story matching and renderer-specific generation with a real fixture. Decide whether importing raw HTML needs bundler configuration and document it. If no portable convention is agreed, deliver a design and leave generation unsupported rather than inventing coverage.
7. **Ember** — Research a pinned Ember/Storybook integration and representative `.gjs`, `.gts`, `.hbs`, and JS/TS component layouts before selecting the initial supported subset. Specify template/class pairing and story locations so one component is not double-counted. Verify generation against the actual integration; document unsupported layouts and version constraints.
8. **Server renderer** — Discovery phase first: Storybook's server integration can use generated JSON/YAML stories and framework-specific server endpoints, so filenames alone cannot establish a universal component-to-story mapping. Pick one concrete server framework and fixture, define the input contract and mapping, then propose a separate implementation. Do not offer generic server coverage until its semantics and generation are demonstrably portable. Existing index-based `spec check` can already check declared registered stories, but cannot discover missing components.

## Application framework compatibility

After the relevant base modes above, validate these separately with representative fixtures and document the verified subset, rather than automatically adding aliases:

- **Next.js and TanStack React:** use `react`; verify normal components and explicitly test route/page conventions and generator output. Next.js special files may be counted by the current JSX rule; do not promise full app-router or server-component support without evidence.
- **Nuxt:** use `vue`; test component directories and page/layout false positives. Document exclusions or introduce narrowly scoped opt-in handling only if needed.
- **SvelteKit:** use `svelte`; test `+page.svelte` and `+layout.svelte` and document existing false positives. Any change to defaults requires regression tests and a compatibility decision.
- **Analog:** use `angular`; verify representative component conventions, decorators and stories; Angular decorator aliases are currently a documented detection gap.

Treat Vite, Webpack and Rspack as builders, not separate component detection modes. Validation of one builder does not imply another is verified. Determine supported upstream versions from pinned fixtures in each phase, not from this roadmap.

## Decisions to resolve during implementation

- Agree on the expected false-positive rate and explicit inclusion mechanisms for HTML and Web Components; filenames alone may be insufficient.
- Define how platform suffixes and multiple components in one file map to a single story; prevent false coverage from name collisions.
- Decide per renderer whether unresolvable exports/tags should produce an error or an explicitly documented placeholder; never silently suggest a placeholder proves rendering.
- Choose fixture maintenance and CI costs before adding integrations. Native device execution and server backends may require infrastructure not present in this repository.
- Recheck community integration maintenance and supported Storybook versions before each phase. The list is scoped to the earlier inventory, not every integration in existence.

## Research sources

- [Storybook framework overview](https://storybook.js.org/docs/) and [installation project types](https://storybook.js.org/docs/get-started/install/).
- [Storybook framework feature matrix (v9)](https://storybook.js.org/docs/9/configure/integration/frameworks-feature-support): historical comparison, not a current compatibility guarantee.
- [Qwik Storybook integration](https://next.qwik.dev/docs/integrations/storybook/).
- [Storybook server renderer](https://github.com/storybookjs/storybook/blob/next/code/frameworks/server-webpack5/README.md).
