# HTML Storybook fixture

From the repository root:

```sh
mise exec -- cargo run -- check tests/fixtures/html/src --framework html --generate
cd tests/fixtures/html
npm ci
npm run build-storybook
```

The index contains `components-button--default` and `components-card--default`.
Serve `storybook-static` locally and open their `iframe.html?id=...` URLs. The
expected content is a button labelled “HTML fragment button” and the text “HTML
fragment card”. `page.html` is intentionally not a component. Remove generated
stories before repeating generation.

Verified in a browser with @storybook/html-vite and Storybook 10.6.5, Vite
7.1.12 and Node 26.12.1. The fixture covers static markup, not scripts,
relative asset resolution, framework-specific templates or non-Vite builders.
