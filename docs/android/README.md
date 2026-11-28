# Android app

A separate codebase from the desktop app. It shares the same rules for validating
downloads but no code.

## Build

Needs:

- `VCPKG_ROOT`;
- `ANDROID_NDK_HOME` pointing at NDK 27.0.12077973;
- JDK 17–21 (Gradle 8.9 rejects newer JDKs);
- `android/local.properties` with `sdk.dir=C:/…` (forward slashes; it's gitignored, so
  recreate it on a new machine).

```bash
vcpkg install curl:arm64-android --classic --overlay-triplets=android/vcpkg-triplets   # once
cd android
./gradlew assembleDebug      # or assembleRelease
```

The APK is written to `android/app/build/outputs/apk/`.

## Layout

| Path | Role |
| --- | --- |
| `app/src/main/cpp/native-lib.cpp` | HTTP engine: HEAD probe, ranged GET workers, reassembly, JSON status over JNI |
| `app/src/main/java/.../MainActivity.java` | UI; handles `magnet:` and `idr:` intents |
| `app/src/main/java/.../TorrentEngine.java` | Torrents through libtorrent4j |
| `app/build.gradle` | Chainloads vcpkg's toolchain through the NDK so `find_package(CURL)` works |
| `vcpkg-triplets/arm64-android.cmake` | Overlay triplet at API 24 |

## Rules

- Keep the triplet at **API 24** (matches `minSdk 24`). The stock API-28 triplet fails
  to link with fortify symbol errors.
- Only `arm64-v8a` is built. Add an ABI to `abiFilters` only after its triplet is shown
  to cross-compile curl.
- Validate ranged responses (206 and the exact `Content-Range`) before writing, and
  check segments before publishing the output.
- An APK that builds isn't tested. Verify downloads on a physical arm64 device.
