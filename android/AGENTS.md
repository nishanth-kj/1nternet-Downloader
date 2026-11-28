# android/ — Android app

Docs: [docs/android/README.md](../docs/android/README.md)

A separate codebase from `core/`; it shares no code, only the same validation rules.

```bash
vcpkg install curl:arm64-android --classic --overlay-triplets=android/vcpkg-triplets   # once
./gradlew assembleDebug
```

Needs `VCPKG_ROOT`, `ANDROID_NDK_HOME` (NDK 27.0.12077973), JDK 17–21, and
`local.properties` with `sdk.dir=C:/…`.

## Rules

- Keep the vcpkg triplet at API 24 (matches `minSdk 24`).
- Only `arm64-v8a` is built; add ABIs only once their triplet cross-compiles curl.
- HTTP: validate `206` and the exact `Content-Range` before writing; check segments
  before publishing the file.
- Torrents go through libtorrent4j (`TorrentEngine.java`); don't fake progress.
- An APK that builds isn't tested. Verify on a physical arm64 device.
