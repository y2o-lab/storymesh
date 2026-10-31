# Express server-renderer investigation

This fixture is an intentionally small Express 5.1.0 application and
`@storybook/server-webpack5` / Storybook 10.6.5 on Node 26.12.1. It defines
`GET /storybook_preview/button` and a `button.stories.json` entry with server
id `button`. The endpoint returns `components/button.html`; `card.html` is a
second component with no endpoint or story. This mapping is deliberately local
to the fixture, not a generic Express or Storybook convention.

From this directory run `npm ci`, `npm run build-storybook`, then `npm start`.
The endpoint `http://127.0.0.1:8771/storybook_preview/button` returns the button
HTML. The Storybook preview must be served separately; CORS is enabled here
only to permit this local example. `index.sample.json` is an index captured from
this pinned build. Rebuild for a fresh index in real use. From the repository
root:

```sh
mise exec -- cargo run -- spec check --spec-dir tests/fixtures/server-express/specs-button --index tests/fixtures/server-express/storybook-static/index.json
mise exec -- cargo run -- spec check --spec-dir tests/fixtures/server-express/specs-both --index tests/fixtures/server-express/storybook-static/index.json
```

The first exits 0 although `components/card.html` has no story: no spec declares
it. The second exits 1 and reports `Card/default` missing, because that spec is
explicitly supplied. Neither result establishes that an endpoint returns correct
HTML; `spec check` compares declared states and index entries only.
