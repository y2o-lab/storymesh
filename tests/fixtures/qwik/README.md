# Qwik Storybook fixture

From the repository root:

```sh
mise exec -- cargo run -- check tests/fixtures/qwik/src --framework qwik --ignore root.tsx --generate
cd tests/fixtures/qwik
npm ci
npm run build-storybook
```

Check `storybook-static/index.json` for `components-button--default` and
`components-card--default`. Serve `storybook-static` locally and open each
`iframe.html?id=components-button--default` and `iframe.html?id=components-card--default`.
Expected content: a button labelled “Qwik button” and the text “Qwik card”.
Remove the generated stories before repeating generation.

Verified with @builder.io/qwik 1.20.3, storybook-framework-qwik 0.6.1,
Storybook 10.6.5, Vite 7.1.12, TypeScript 5.9.3 and Node 26.12.1. The Qwik
optimizer and JSX configuration are required for rendering; a successful build
without them produced an empty canvas. The root module satisfies the optimizer
and is excluded from the component scan. This covers simple `component$` default
and named exports, not components requiring Qwik City context or other providers.
