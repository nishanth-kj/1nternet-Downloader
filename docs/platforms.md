# Platform support

This document describes the current repository configuration. Support should not be inferred beyond the targets listed here.

| Component | Technology | Current target | Limitations |
| --- | --- | --- | --- |
| Desktop | C++17, wxWidgets, libcurl, SQLite, libtorrent | Windows x64 | Windows-focused build |
| Android | Java, JNI, C++, libcurl | arm64-v8a; min SDK 24 | BitTorrent/magnet downloads are not implemented |
| Browser | Manifest V3, JavaScript | Chrome-compatible browsers and browser-specific folders | Requires desktop app and `idr://` protocol registration |
| Website | Next.js, React, TypeScript | Static export on GitHub Pages | Not a hosted backend or download service |

## Desktop

The desktop application supports HTTP/HTTPS downloads, queue management, pause/resume/cancel, segmented transfers when the server supports byte ranges, SQLite metadata, and BitTorrent through libtorrent.

## Android

The Android app uses a native C++ download engine through JNI. The current Gradle configuration filters to `arm64-v8a`. The Android engine does not provide the desktop BitTorrent implementation.

## Browser extensions

Browser extensions hand URLs to the desktop application. They do not download files independently. Automatic interception is disabled by default because a custom-protocol request cannot confirm that the desktop app accepted the URL.

## Website

The website is configured for Next.js static export and GitHub Pages. Release download links must point to published artifacts; a successful website deployment does not create application installers.
