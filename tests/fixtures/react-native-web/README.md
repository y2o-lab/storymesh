# React Native Web Storybook fixture

From the repository root:

```sh
mise exec -- cargo run -- check tests/fixtures/react-native-web/src --framework react --generate
cd tests/fixtures/react-native-web
npm ci
npm run build-storybook
```

Verify `components-button--default`, `components-button-web--default` and `components-card--default` in
`storybook-static/index.json`. Serve `storybook-static` locally and open the
corresponding `iframe.html?id=...` URLs. Both Button stories display “Native web
button” even though `Button.tsx` says “Generic button”: the generated extensionless
`./Button` import resolves to `Button.web.tsx` with this builder. The Card story
displays “Native web card”. Remove generated stories before repeating generation.

Verified with React and React DOM 19.2.0, React Native 0.82.1, React Native Web
0.21.4, @storybook/react-native-web-vite and Storybook 10.6.5, Vite 7.1.12,
and Node 26.12.1. This verifies a default export in a `.web.tsx` file and a
named export in a normal file in a browser only; it does not verify device
rendering, other resolvers, native modules, or
provider-dependent components.
