# Releases and distribution

A successful build produces a development artifact; it does not automatically produce a complete installer.

## Build artifacts

- **Windows:** `InternetDownloader.exe` plus the runtime DLLs required by the linked dependencies.
- **Android:** APK under `android/app/build/outputs/apk/`.
- **Website:** static export under `site/out/`.
- **Browser:** the relevant browser extension directory can be loaded as an unpacked extension or packaged according to the target browser's distribution process.

## Release process

1. Confirm the supported platforms and known limitations in [Platform support](platforms.md).
2. Build the intended desktop and Android configurations.
3. Prepare the complete Windows runtime dependency set; do not distribute the executable alone if required DLLs are missing.
4. Publish versioned release artifacts on GitHub Releases.
5. Include release notes, supported architectures, known limitations, and SHA-256 checksums.
6. Update website download links only after the corresponding artifacts are published.

## Current limitations

- The repository does not currently provide a verified Windows installer.
- Android support is limited to `arm64-v8a`.
- Android BitTorrent support is not implemented.
- Build workflow artifacts are not the same as a signed, end-user installer.

Do not describe a platform build as production-ready until its distributable package and runtime requirements are documented.
