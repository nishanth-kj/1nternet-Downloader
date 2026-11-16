# Workflows — build, run, verify, release

## Desktop (`core/`, Windows)

Prerequisites: Visual Studio 2022+ with the C++ workload, CMake ≥ 3.21, and vcpkg.
`build.ps1` looks for vcpkg in `VCPKG_ROOT`, then `C:\vcpkg`, then the copy
bundled with Visual Studio (`<VS>\VC\vcpkg`).

```powershell
cd core
./scripts/build.ps1            # Debug → core/build/Debug/InternetDownloader.exe
./scripts/build.ps1 -Preset release
./scripts/build.ps1 -Clean     # wipe core/build (needed if the toolchain path changed)
./scripts/register_protocol.ps1   # once per machine: idr:// → this exe (HKCU, no admin)
```

- The first configure with new dependencies builds boost, OpenSSL and libtorrent
  from source (10–20+ min). That's normal.
- vcpkg packages are installed into `core/vcpkg_installed/` (gitignored).
- Debug builds print wx assertion messages to stderr. Treat any new one as a bug.

### Runtime files

`%APPDATA%\InternetDownloader\` contains `downloads.db`, `settings.json` and
`logs\idr.log`. Delete the folder to reset the app. A `downloads.db` in the repo
root is never used; it is gitignored and must not be committed.

### Verifying engine changes

No automated test suite exists, and a successful compile proves nothing about
downloads. After any change under `core/src/service/`, run the scripted
end-to-end check in [`core/tests/e2e/README.md`](../core/tests/e2e/README.md). It
uses a local Node server that covers segmented, flaky, single-stream, range-ignoring
and 404 cases, and you compare SHA-256s. Pause/resume/restart/remove still need
manual GUI checks.

## Android (`android/`)

Requires `VCPKG_ROOT`, `ANDROID_NDK_HOME` = NDK 27.0.12077973, JDK ≤ 21 (Gradle 8.9
rejects 24+), and `android/local.properties` with `sdk.dir=C:/…` (forward slashes).

```bash
vcpkg install curl:arm64-android --classic --overlay-triplets=android/vcpkg-triplets   # once
cd android && ./gradlew assembleDebug
```

Test downloads on a physical arm64 device. An APK build is not a test.

## Browser extensions (`browser/`)

No build step. Load `browser/<browser>/` unpacked (Developer mode). Change
`browser/chrome/` first, then copy it:

```bash
for b in edge brave opera; do cp browser/chrome/{background,popup}.js browser/$b/; done
cp browser/chrome/popup.js browser/firefox/ && cp browser/chrome/popup.js browser/safari/
# firefox/safari background.js: keep their extra header comment, copy the code below it
for b in chrome edge brave opera firefox safari; do
  node --check browser/$b/background.js && node --check browser/$b/popup.js
done
```

## Website (`site/`)

```bash
cd site && npm ci && npm run lint && npm run build   # static export → site/out/
```

Read `node_modules/next/dist/docs/` before using Next.js APIs (see `site/AGENTS.md`).

## CI (`.github/workflows/`)

| Workflow | Checks |
| --- | --- |
| `build-core.yml` | syntax and drift check for all six extensions; Windows Release build |
| `build-android.yml` | cross-compiles curl and builds the APK |
| `site-ci.yml` / `deploy-site.yml` | lint + build; deploy to GitHub Pages |

## Release

Work through [`docs/DOWNLOAD_CORRECTNESS_CHECKLIST.md`](../docs/DOWNLOAD_CORRECTNESS_CHECKLIST.md)
before you publish. Attach versioned artifacts and checksums to GitHub Releases.
