# Web Components and Lit Storybook fixture

From the repository root:

```sh
mise exec -- cargo run -- check tests/fixtures/web-components/src --framework web-components --generate
cd tests/fixtures/web-components
npm ci
npm run build-storybook
```

Confirm `components-counter--default` and `components-greeting--default` in
`storybook-static/index.json`. Serve `storybook-static` locally and open the
corresponding `iframe.html?id=...` URLs. Expected text is “Native element” and
“Lit greeting”. Remove generated stories before repeating generation.

Verified in a browser with Lit 3.3.2, @storybook/web-components-vite and
Storybook 10.6.5, Vite 7.1.12, TypeScript 5.9.3 and Node 26.12.1. The
TypeScript fixture enables experimental decorators. These simple examples do
not exercise required attributes, providers, customized built-in elements, or
other builders.
