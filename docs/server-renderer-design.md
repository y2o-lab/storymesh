# Server renderer investigation: Express fixture

Issue #40 investigates one concrete contract, not generic server framework
support. The reproducible [Express fixture](../tests/fixtures/server-express/README.md)
uses Express 5.1.0 and Storybook server-webpack5 10.6.5 on Node 26.12.1. A
Storybook JSON entry maps `Components/Button` / `Default` to server id `button`;
Storybook requests `/storybook_preview/button`, and the fixture's explicit route
returns the HTML from `components/button.html`. The second file `card.html` has
no story or route. The pinned Storybook build produced an index containing the
Button story, and the endpoint returned the expected HTML. The fixture documents
commands and contains a captured index for offline tests.

`spec check` with only the Button declaration passed despite the unrepresented
Card file. Adding a Card/Default declaration caused a missing-story failure.
This demonstrates declared-state checking against registered titles and names,
not discovery of server components or endpoint validation. Storybook's server
id is an endpoint key and does not itself identify a source file. See the
[Storybook server contract](https://github.com/storybookjs/storybook/blob/next/code/frameworks/server-webpack5/README.md).

## Proposed follow-up, not implemented

An opt-in manifest owned by the application could map a stable component id to
one or more explicit source paths, a Storybook title, and allowed server ids.
A dedicated adapter would validate duplicate ids/titles, missing paths, unknown
stories, and ambiguous or absent mappings before calculating coverage. It should
consume a fresh Storybook index for registration and treat endpoint fetch/render
validation as a separate optional integration test, with an explicit base URL,
timeout and authorization policy. Discovery must be scoped to a user-specified
component root and file convention, not inferred from arbitrary server files.
Any generated JSON skeleton would need an application-supplied endpoint mapping;
without it, generate a draft or an error rather than claiming a working story.

Open work: define and version the manifest schema, establish how args and server
ids map to states, test collisions and renamed sources, validate endpoint failure
and HTML behavior, and build a second real integration before considering shared
abstractions. This Express example does not prove Rails, Django, other server
frameworks, authentication, dynamic routes, or universal component discovery.
