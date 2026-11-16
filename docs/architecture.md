# Architecture

Internet Downloader is a small multi-platform repository. Each client has its own entry point; the desktop and Android applications use separate native download engines.

## Components

### Desktop — `core/`

- **Application entry:** `main.cpp` starts the wxWidgets application and handles incoming `idr://` URLs.
- **GUI:** `src/gui/` contains the main window and system-tray integration.
- **Download service:** `src/service/download/` manages downloads, workers, segments, queue state, pause, and resume.
- **Network service:** `src/service/network/` wraps libcurl and provides HTTP requests, responses, headers, cookies, and proxy configuration.
- **Torrent service:** `src/service/torrent/` integrates libtorrent for desktop BitTorrent downloads.
- **Persistence:** `src/model/` stores download metadata in SQLite.
- **Utilities:** `src/utils/` contains filesystem, logging, validation, and Windows IPC helpers.

The desktop application is Windows-focused and built with C++17, CMake, and vcpkg.

### Android — `android/`

The Android UI is written in Java. `MainActivity.java` calls the native C++ downloader through JNI; `native-lib.cpp` implements HTTP(S) transfers with libcurl. The Android engine is independent of the desktop engine.

### Browser integrations — `browser/`

Browser extensions use Manifest V3 and send supported URLs to the desktop application through the registered `idr://` protocol. The extension does not itself perform the download.

### Website — `site/`

The Next.js application is exported as a static site and deployed through GitHub Pages.

## HTTP download flow

1. The UI submits a URL to the download manager.
2. The downloader inspects server capabilities.
3. If byte ranges are supported and appropriate, the file is downloaded in parallel segments; otherwise it uses a single stream.
4. Download data is written to disk and progress is exposed to the UI.
5. Desktop download metadata is persisted in SQLite.

## Browser handoff

1. The extension requests an `idr://` URL.
2. The operating system resolves the registered protocol handler.
3. The desktop application receives the URL.
4. The desktop application adds the URL to its download queue.

The handoff depends on the desktop application and protocol registration being installed correctly.

## Important implementation boundaries

- BitTorrent is supported by the desktop engine only.
- Android currently targets `arm64-v8a`.
- Browser extensions are integrations, not standalone download engines.
- Do not assume that a successful compile proves runtime behavior on every platform.
