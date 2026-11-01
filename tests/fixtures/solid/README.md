# SolidJS Storybook fixture

From the repository root:

```sh
mise exec -- cargo run -- check tests/fixtures/solid/src --framework solid --generate
cd tests/fixtures/solid
npm ci
npm run build-storybook
```

Check `storybook-static/index.json` for `components-button--default` and
`components-card--default`. Serve `storybook-static` locally and open each
`iframe.html?id=components-button--default` and `iframe.html?id=components-card--default`.
Expected content: a button labelled “Solid button” and the text “Solid card”.
Remove the generated stories before repeating generation.

Verified with SolidJS 1.9.18, storybook-solidjs-vite 10.7.4, Storybook 10.6.5,
Vite 7.1.12, vite-plugin-solid 2.11.15, TypeScript 5.9.3, and Node 26.12.1.
This covers simple default and named exports, not components requiring providers.

The Card source uses an export list with the component after a helper. The
repository harness regenerates stories in a temporary copy, builds Storybook
and checks Default entries: `mise run fixture-check`. Install each guarded
fixture's dependencies before running it. This gate does not render a browser.
