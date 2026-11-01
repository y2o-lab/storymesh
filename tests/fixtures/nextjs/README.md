# Next.js Storybook fixture

The lockfile pins Next.js 15.5.28, React / React DOM 19.2.0,
@storybook/nextjs-vite and Storybook 10.6.5, and Vite 7.1.12. Verified with
Node 26.12.1. From the repository root:

```sh
mise exec -- cargo run -- check tests/fixtures/nextjs/app/components --framework react --generate
cd tests/fixtures/nextjs
npm ci
npm run build-storybook
```

The generated index registers `components-button--default`,
`components-badge--default`, and `components-servercard--default`. Serving the
build and opening their `iframe.html?id=...` URLs rendered the client Button,
named Badge, and simple async ServerCard in headless Chrome. The fixture enables
Storybook's experimental RSC feature. This is browser rendering, not a Next.js
server render, and does not cover server-only modules, data access, routing,
providers, or other Next.js / Storybook versions. Remove generated stories to
repeat generation.

Scanning all of `app` counts the route `page.tsx`, `layout.tsx`, and
`dashboard/loading.tsx` files as components. The two pages have the same spec
title; limit the scan to components or exclude routes when they are not intended
to be counted.
