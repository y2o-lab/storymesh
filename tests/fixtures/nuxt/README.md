# Nuxt Storybook fixture

The lockfile pins Nuxt 3.21.11, Vue 3.5.22, @nuxtjs/storybook and
@storybook-vue/nuxt 10.0.3, Storybook 10.6.5, and Vite 7.1.12. Verified with
Node 26.12.1. From the repository root:

```sh
mise exec -- cargo run -- check tests/fixtures/nuxt/components --framework vue --generate
mise exec -- cargo run -- check tests/fixtures/nuxt/pages --framework vue --generate
mise exec -- cargo run -- check tests/fixtures/nuxt/layouts --framework vue --generate
cd tests/fixtures/nuxt
npm ci
npm run build-storybook
```

The generated index registers `components-button--default`,
`components-welcome--default`, `components-index--default`, and
`components-default--default`. Headless Chrome rendered the button, a component
using auto-imported `useRuntimeConfig`, and the simple page. The layout rendered
an empty `div` because the generated story does not supply slot content. This
checks browser rendering in this integration, not Nuxt server rendering,
route-dependent data, plugins, all auto-imports, or other versions. Remove
generated stories before repeating generation.
