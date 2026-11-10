# Preact Storybook fixture

From the repository root, run:

```sh
mise exec -- cargo run -- check tests/fixtures/preact/src --framework preact --generate
cd tests/fixtures/preact
npm ci
npm run build-storybook
```

The generated `storybook-static/index.json` should contain `components-button--default`
and `components-card--default`. Serve `storybook-static` locally and open each
`iframe.html?id=components-button--default` and `iframe.html?id=components-card--default`.
The expected content is a button labelled “Preact button” and the text “Preact card”.
Remove the generated stories before repeating the generation step.

Verified with Preact 10.27.2, Storybook and @storybook/preact-vite 10.6.5,
Vite 7.1.12 and Node 26.12.1. The `viteFinal` configuration selects Preact's
JSX runtime; without it this standalone fixture's JSX compiles to React calls.
This checks simple default and named exports only, not provider-dependent components.

The Card source uses an export list with the component after a helper. The
repository harness regenerates stories in a temporary copy, builds Storybook
and checks Default entries: `mise run fixture-check`. Install each guarded
fixture's dependencies before running it. This gate does not render a browser.
