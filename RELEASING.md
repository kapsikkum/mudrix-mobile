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

## Pre-releases from `next`

Test builds come from the `next` branch and use a suffixed version of the release they lead up to:

```
26.11.0-next.1  <  26.11.0-next.2  <  26.11.0-rc.1  <  26.11.0
```

Use the **same version string as the web client and the desktop app**. A tag with a `-` in it
(`v26.11.0-next.1`, `-rc.1`, `-beta.1`) is published as a **pre-release**, not "latest": both workflows set
that from the tag. Both still build and attach their file (`Mudrix-...apk`, `Mudrix-unsigned.ipa`).

1. Release the web client first (`RELEASING.md` in `mudrix-web`) and push its tag; the commit has to be on
   GitHub before CI can check it out.
2. On `next`, point the submodule at the web commit or tag you just released:
   ```sh
   git checkout next
   git -C mudrix-web fetch origin
   git -C mudrix-web checkout v26.11.0-next.1
   git add mudrix-web
   ```
3. Set the version and commit (`set-version` updates `package.json`, `Cargo.toml` and `Cargo.lock`, then
   runs `check-version`):
   ```sh
   node scripts/set-version.mjs 26.11.0-next.1
   git commit -am "chore(release): prepare v26.11.0-next.1"
   ```
4. `git tag v26.11.0-next.1 && git push origin next v26.11.0-next.1`.

Both workflows also have **Run workflow** (`workflow_dispatch`) for a build from a branch without a tag; there
the files are only uploaded as run artifacts, not attached to a release.

**Android version code.** Tauri derives it from the numeric part of the version (`major * 1000000 + minor *
1000 + patch`, so 26.10.19 is 26010019) and ignores the suffix. Every `26.11.0-next.N` and the stable `26.11.0`
therefore share the code 26011000: a pre-release installs over 26.10.19 (a higher code), but installing the next
pre-release over the previous one relies on Android accepting an equal code from the package installer.
That has not been tried; if it refuses, uninstall first (app data is lost) or bump the patch number for test
builds (`26.11.1-next.1`).

## iOS

The IPA is unsigned. Users sideload it with a tool that signs it for their own device.
