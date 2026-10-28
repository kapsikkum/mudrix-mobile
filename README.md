# Mudrix Mobile

The [Mudrix](https://github.com/kapsikkum/mudrix-web) web client in a Tauri v2 shell for Android and iOS.
The web client is the `mudrix-web` submodule; this repo only holds the native shell and CI.

## Downloads

Each [release](https://github.com/kapsikkum/mudrix-mobile/releases/latest) has a signed Android APK and an
unsigned iOS IPA. The IPA has to be sideloaded (AltStore, Sideloadly, TrollStore).

## Build

- **Android (local):** `npm ci`, then `npm run android:build` (needs the Android SDK/NDK 27.2 and JDK 21).
  On Windows the Tauri CLI needs Developer Mode for a symlink; the alternative is to copy the built
  `libmudrix_mobile_lib.so` into `src-tauri/gen/android/app/src/main/jniLibs/<abi>/` and run Gradle.
- **iOS:** needs macOS and Xcode. The `iOS (unsigned IPA)` workflow builds an IPA you can sideload.
- **CI:** `.github/workflows/android.yml` and `ios.yml` run on `v*` tags or manually.

Release signing and versioning are covered in `RELEASING.md`.

## Native code

- **Push (Android):** notifications arrive over [UnifiedPush](https://unifiedpush.org) when a distributor is
  installed. Without one, an optional foreground service keeps the app's sync connection alive in the
  background instead. The bridge is `src-tauri/src/push.rs`, the Kotlin side lives in
  `src-tauri/gen/android/app/src/main/java/chat/mudrix/mobile/`.
- **iOS web view:** `src-tauri/gen/apple` is not committed (CI runs `tauri ios init`), so native iOS code
  lives in local Tauri plugins. `src-tauri/plugins/ios-webview` stops the WKWebView from scrolling or panning
  and resizes it to the area above the keyboard, like Android's `MainActivity` does; its contract with the
  web client is documented in its `src/lib.rs`.

## Status

Experimental. Android builds run and receive UnifiedPush notifications. On iOS there are no push
notifications yet.

License: AGPL-3.0, derived from WizardChat.
