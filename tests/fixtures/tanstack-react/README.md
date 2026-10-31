# TanStack React Storybook fixture

The lockfile pins TanStack React Router 1.170.46, React / React DOM 19.2.0,
@storybook/tanstack-react and Storybook 10.6.5, Vite 7.1.12 and
@vitejs/plugin-react 5.2.0. Verified with Node 26.12.1. From the repository root:

```sh
mise exec -- cargo run -- check tests/fixtures/tanstack-react/src/components --framework react --generate
mise exec -- cargo run -- check tests/fixtures/tanstack-react/src/routes --framework react --generate
cd tests/fixtures/tanstack-react
npm ci
npm run build-storybook
```

The generated index registers `components-badge--default`,
`components-navlink--default`, `components-root--default`, and
`components-about--default`. Serving the build and opening their
`iframe.html?id=...` URLs rendered the named Badge and router `Link` in
headless Chrome; the `about` story was empty because it is a generated null
placeholder. The integration supplies router context, but this fixture does not
verify route parameters, loaders, navigation, TanStack Start server functions,
or other versions. The route files illustrate file naming; `routeTree.gen.ts`
is a minimal naming sample, not a generated working application route tree.
Remove generated stories before repeating.
