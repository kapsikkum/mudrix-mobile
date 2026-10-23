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

## Native code

`src-tauri/gen/apple` is not committed (CI runs `tauri ios init`), so native iOS code lives in local
Tauri plugins. `src-tauri/plugins/ios-webview` stops the WKWebView from scrolling or panning and
resizes it to the area above the keyboard, like Android's `MainActivity` does; its contract with the
web client is documented in its `src/lib.rs`.

## Releases

`node scripts/set-version.mjs X.Y.Z` updates `package.json` and `Cargo.toml`;
`node scripts/check-version.mjs vX.Y.Z` verifies them against a tag.

## Release signing

The Android workflow builds a release-signed APK when these repository secrets exist, and falls
back to a debug-signed APK (files named `Mudrix-debug-...`) when they do not. Create a keystore once
and keep it safe; losing it means users cannot update in place.

```bash
keytool -genkeypair -v -keystore mudrix-release.keystore -alias mudrix   -keyalg RSA -keysize 4096 -validity 10000
base64 -w0 mudrix-release.keystore > keystore.b64   # macOS: base64 -i mudrix-release.keystore
```

Add four secrets under Settings > Secrets and variables > Actions:

| Secret | Value |
|---|---|
| `ANDROID_KEYSTORE_BASE64` | contents of `keystore.b64` |
| `ANDROID_KEYSTORE_PASSWORD` | the keystore password |
| `ANDROID_KEY_ALIAS` | `mudrix` (the `-alias` above) |
| `ANDROID_KEY_PASSWORD` | the key password |

For a local signed build, put `storeFile`, `storePassword`, `keyAlias` and `keyPassword` in
`src-tauri/gen/android/keystore.properties` (gitignored), or set the `ANDROID_KEYSTORE_PATH`,
`ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS` and `ANDROID_KEY_PASSWORD` environment variables,
then run `npx tauri android build --apk`.

## Status

Experimental. Android debug builds run; iOS and push notifications are not done yet.

License: AGPL-3.0, derived from WizardChat.
