# android/ — Android app

Separate codebase from `core/`; it shares no code, only the same range-validation rules.

## Build

Requires `VCPKG_ROOT`, `ANDROID_NDK_HOME` = NDK 27.0.12077973, a JDK ≤ 21 (Gradle 8.9
rejects 24+), and a local `local.properties` with `sdk.dir=C:/...` (forward slashes;
gitignored, so recreate it on a fresh machine).

```bash
vcpkg install curl:arm64-android --classic --overlay-triplets=android/vcpkg-triplets   # once
./gradlew assembleDebug
```

## Layout

- `app/src/main/cpp/native-lib.cpp`: native engine. HEAD probe, ranged GET
  workers, reassembly, JSON status over JNI (strings are escaped).
- `app/src/main/java/.../MainActivity.java`: UI. Handles `magnet:`/`idr:` intents
  and calls `nativeStartDownload`.
- `app/build.gradle`: chainloads vcpkg's toolchain through the NDK's
  (`VCPKG_CHAINLOAD_TOOLCHAIN_FILE`) so `find_package(CURL)` resolves.
- `vcpkg-triplets/arm64-android.cmake`: overlay triplet at **API 24**.

## Rules

- Keep the triplet at API 24 (matches `minSdk 24`). The stock API-28 triplet links
  with `__sendto_chk`-style fortify symbol errors.
- Only `arm64-v8a` is built. Add an ABI to `abiFilters` only after its triplet is
  proven to cross-compile curl.
- Torrent/magnet downloads return `Status::Error` **on purpose**. Don't stub fake progress.
- Validate ranged responses (206 + exact `Content-Range`) before writing, and
  verify segments before publishing the output, as the desktop engine does.
- An APK build is not a test. Verify downloads on a physical arm64 device.
