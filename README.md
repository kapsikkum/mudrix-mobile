# Mudrix Mobile

The [Mudrix](https://github.com/kapsikkum/mudrix-web) web client in a Tauri v2 shell for Android and iOS.
The web client is the `mudrix-web` submodule; this repo only holds the native shell and CI.

## Build

- **Android (local):** `npm ci`, then `npm run android:build` (needs the Android SDK/NDK 27.2 and JDK 21).
  On Windows the Tauri CLI needs Developer Mode for a symlink; the alternative is to copy the built
  `libmudrix_mobile_lib.so` into `src-tauri/gen/android/app/src/main/jniLibs/<abi>/` and run Gradle.
- **iOS:** needs macOS and Xcode; the `iOS (unsigned IPA)` workflow builds an IPA you can sideload
  (AltStore, Sideloadly, TrollStore).
- **CI:** `.github/workflows/android.yml` and `ios.yml` run on `v*` tags or manually.

## Releases

`node scripts/set-version.mjs X.Y.Z` updates `package.json` and `Cargo.toml`;
`node scripts/check-version.mjs vX.Y.Z` verifies them against a tag.

## Status

Experimental. Android debug builds run; iOS and push notifications are not done yet.

License: AGPL-3.0, derived from WizardChat.
