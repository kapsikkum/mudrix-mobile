# Releasing Mudrix Mobile

Releases are `v*` tags. `.github/workflows/android.yml` and `ios.yml` build on the tag and attach the APK and
the IPA to the release.

## Versions

`node scripts/set-version.mjs X.Y.Z` updates `package.json` and `Cargo.toml`;
`node scripts/check-version.mjs vX.Y.Z` verifies them against a tag. The `mudrix-web` submodule should point at
the matching web tag.

## Android signing

The Android workflow builds a release-signed APK when these repository secrets exist, and falls back to a
debug-signed APK (files named `Mudrix-debug-...`) when they do not. The keystore is created once and must be
kept safe: losing it means installed apps cannot be updated in place.

```bash
keytool -genkeypair -v -keystore mudrix-release.keystore -alias mudrix -keyalg RSA -keysize 4096 -validity 10000
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

## iOS

The IPA is unsigned. Users sideload it with a tool that signs it for their own device.
