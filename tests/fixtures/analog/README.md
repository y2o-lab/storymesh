# Analog Angular Storybook fixture

The lockfile pins Angular 20.3.34, Angular CLI / build-angular 20.3.30,
@analogjs/storybook-angular 2.9.3, @analogjs/vite-plugin-angular 2.9.2,
@storybook/angular and Storybook 10.6.5, TypeScript 5.9.3, and Vite 7.1.12.
Verified with Node 26.12.1. From the repository root:

```sh
mise exec -- cargo run -- check tests/fixtures/analog/src/app/components --framework angular --generate
cd tests/fixtures/analog
npm ci
npm run build-storybook
```

The generated index registers `components-button--default` and
`components-card--default`; both rendered in headless Chrome with Zone.js
loaded by the preview. This checks standalone Angular components using Analog's
Storybook / Vite integration and a representative `.page.ts` source layout,
not a running Analog application, file routing, SSR, providers, loaders or
other versions. Remove generated stories before repeating.

The default-exported `index.page.ts` is detected and generation now uses a
default import. CLI and generation regression tests cover this import. The repository
harness (`mise run fixture-check`) also generates stories for the entire `src`
directory, builds them in a temporary copy and checks Default index entries,
including the page. Browser rendering above was verified only for the component
directory. The aliased `@NgComponent` in `alias.ts` is not detected.
