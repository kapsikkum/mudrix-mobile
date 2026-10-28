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

## Development (live-reload build on the emulator)

The dev build (`chat.mudrix.mobile.dev`) loads the web client from a Vite dev server on the PC over
`adb reverse`, not over the emulator's network:

```sh
# 1. in mudrix-web: the dev server must answer on IPv4 127.0.0.1:8080 (adb reverse connects there;
#    a plain `npm start` on Windows may bind only to [::1])
npm start -- --host 127.0.0.1
# 2. after every emulator (re)start, and for a USB phone too
adb reverse tcp:8080 tcp:8080
```

Why not `10.0.2.2`: current emulators route app traffic over the netsim virtual Wi-Fi (`wlan0`),
whose NAT refuses every TCP connection to the host loopback at `10.0.2.2` (ping still works, which
hides it). `src-tauri/tauri.attach.conf.json` therefore points `devUrl` at `http://127.0.0.2:8080`:
a loopback IP, so `adb reverse` serves it, but not `localhost`/`127.0.0.1`, which the Tauri CLI on
Windows always swaps for the PC's LAN address (that changes with the network). An IP (not a
hostname) also keeps the app on the `http://tauri.localhost` origin, so the session in the installed
dev app stays signed in. Vite's HMR socket falls back to `ws://localhost:8080`, also covered.

Building the dev APK (x86_64 emulator; Windows without Developer Mode, so the CLI's symlink step
fails after compiling and the library is copied by hand):

```sh
ORG_GRADLE_PROJECT_mudrixDev=true npx tauri android dev --config src-tauri/tauri.attach.conf.json --no-watch --no-dev-server-wait
cp src-tauri/target/x86_64-linux-android/debug/libmudrix_mobile_lib.so src-tauri/gen/android/app/src/main/jniLibs/x86_64/
cd src-tauri/gen/android && ANDROID_HOME=<sdk> ./gradlew assembleX86_64Debug -PmudrixDev -x rustBuildX86_64Debug
adb install -r app/build/outputs/apk/x86_64/debug/app-x86_64-debug.apk   # -r keeps the app's data
```

A dev session signed in to the local test server at `http://localhost:6167` also needs that port
reversed. Windows can reserve 6167 (`netsh int ipv4 show excludedportrange protocol=tcp`); then
forward a free host port to the test server and map it: `adb reverse tcp:6167 tcp:16167`.

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
