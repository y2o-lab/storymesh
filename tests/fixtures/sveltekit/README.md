# SvelteKit Storybook fixture

The lockfile pins SvelteKit 2.70.3, Svelte 5.57.3, @sveltejs/vite-plugin-svelte
6.2.4, @storybook/sveltekit and Storybook 10.6.5, and Vite 7.1.12. Verified with
Node 26.12.1. From the repository root:

```sh
mise exec -- cargo run -- check tests/fixtures/sveltekit/src/lib --framework svelte --generate
cd tests/fixtures/sveltekit
npm ci
npm run build-storybook
```

The generated index registers `components-button--default` and
`components-environment--default`. Both rendered in headless Chrome; the latter
uses `$app/environment` and displayed `browser: yes`. This does not establish
server rendering, route data, navigation, private environment variables or other
versions. Remove the generated stories before repeating.

Running `check src --framework svelte --generate` also generates
`src/routes/+page.stories.ts` and `+layout.stories.ts`. With this SvelteKit
version, those reserved `+` filenames cause the Storybook build to fail before
registration. Remove them and limit generation to components or ignore routes.
`spec init` rejects the `+page` and `+layout` names as invalid v1 IDs. These
observations do not cover every SvelteKit configuration.
