# Internet Downloader

A desktop-first download manager with companion Android and browser integrations.

## Features

- HTTP and HTTPS downloads
- Pause, resume, cancel, and queued downloads
- Parallel byte-range downloads when supported by the server
- SQLite download history on desktop
- BitTorrent downloads on desktop
- Android downloader powered by a native C++ engine
- Browser extension handoff to the desktop application

## Components

| Directory | Description | Status |
| --- | --- | --- |
| `core/` | Native Windows desktop application (C++17, wxWidgets) | Primary application |
| `android/` | Android application (Java + JNI/C++) | arm64-v8a; no BitTorrent |
| `browser/` | Browser extensions | Uses the desktop `idr://` protocol |
| `site/` | Project website (Next.js static export) | GitHub Pages |

See [Platform support](docs/platforms.md) for exact limitations.

## Quick start

### Windows desktop

**Requirements:** Windows 10/11, Visual Studio C++ build tools, CMake 3.21+, Git, and [vcpkg](https://github.com/microsoft/vcpkg).

1. Install vcpkg and set `VCPKG_ROOT` to its installation directory.
2. Open a terminal with the Visual Studio C++ build tools available.
3. Run:

```powershell
cd core
cmake --preset default
cmake --build --preset default
```

The executable is created under `core/build/`. See [Development setup](docs/development.md) for Debug, Release, and troubleshooting instructions.

### Android

**Requirements:** JDK 17, Android SDK, Android NDK 27.0.12077973, CMake 3.22.1, and vcpkg.

Set `VCPKG_ROOT` and `ANDROID_NDK_HOME`, build the Android curl dependency for the project's overlay triplet, then run:

```bash
cd android
./gradlew assembleDebug
```

The exact setup is documented in [Platform support](docs/platforms.md).

### Browser extension

Install the desktop application, register the `idr://` protocol, then load the appropriate unpacked extension from `browser/<browser>/`. Automatic interception is disabled by default.

## Documentation

Project documentation lives in [docs/](docs/README.md):

- [Architecture](docs/architecture.md)
- [Development setup](docs/development.md)
- [Platform support and limitations](docs/platforms.md)
- [Release and distribution](docs/releases.md)

## Project structure

```text
.
├── android/              Android application and JNI downloader
├── browser/              Browser extensions
├── core/                 C++ desktop application
├── docs/                 Project and developer documentation
├── site/                 Static project website
└── .github/workflows/    Build and deployment workflows
```

## Security

Do not include credentials, private download URLs, or personal data in bug reports. See [SECURITY.md](SECURITY.md) for vulnerability reporting.

## License

This project uses a custom non-commercial license. It is not an OSI-approved open-source license. Read [LICENSE](LICENSE) before using, modifying, or redistributing the software.
