# React Native on-device Storybook fixture (device verification pending)

This Expo fixture pins Expo 54.0.20, React Native 0.81.5, React 19.1.0,
@storybook/react-native 10.7.1 and Storybook 10.6.5 in `package-lock.json`.
It follows the in-app Storybook setup and uses Node 26.12.1 for the checks below.
From the repository root:

```sh
mise exec -- cargo run -- check tests/fixtures/react-native/components --framework react --generate
cd tests/fixtures/react-native
npm ci
STORYBOOK_ENABLED=true npx expo export --platform ios --no-minify --no-bytecode --source-maps
STORYBOOK_ENABLED=true npx expo export --platform android --no-minify --no-bytecode --source-maps
```

Both Metro exports succeeded. Their generated `require.context` maps contain all
four story files: `Button`, `Button.ios`, `Button.native`, and `Card`. Inspecting
the unminified dependency maps shows that the generic Button story's `./Button`
import resolves to `Button.ios.tsx` for iOS and `Button.native.tsx` for Android,
not to `Button.tsx`. The explicitly suffixed imports resolve to those suffix
files in both exports; in particular, the iOS story is included in the Android
bundle too. A successful bundle and context entry do not prove runtime story
registration or rendering.

No iOS simulator or Android emulator was available for this verification
(`xcrun simctl` was unavailable and `adb` / `emulator` were not on PATH); no
physical device was used. Run `npm run storybook` in an appropriate Expo 54
device environment, open the on-device Storybook, and check all four stories,
including the resolved Button variants, before treating device behavior as
verified. Remove generated stories before repeating the generation step.
