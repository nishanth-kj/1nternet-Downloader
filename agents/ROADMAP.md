# Roadmap and known gaps

Ordered roughly by user impact. Each item names where the work happens.

## Desktop (`core/`)

- **Automated tests.** Turn `core/tests/e2e/range_server.js` into a CTest target
  that drives `Download` directly (no GUI), and add unit tests for
  `ChooseSegmentCount`, `SpeedMeter` and the header and range validation.
  `core/tests/{model,service,utils}` are reserved for this.
- **Global speed limit.** The current limit is per download, split across its
  connections. A shared token bucket across all downloads would match user
  expectations better.
- **Remote-file-changed detection.** Store `ETag`/`Last-Modified` from the first
  probe and send `If-Range` on resume, so stale parts are discarded
  (`DOWNLOAD_CORRECTNESS_CHECKLIST.md` item).
- **Content-Disposition filenames.** External adds name files from the URL path.
  Prefer the server-provided filename.
- **Persist the SHA-256 result** into the existing `checksum` column and show it.
- **Unicode paths.** The engine uses native narrow strings. Paths containing
  characters outside the ANSI code page need `std::filesystem::path`/wide strings
  end to end.
- **Dynamic segment splitting** (steal work from slow segments), like IDM.
- **Scheduler** (start or stop the queue at set times). Out of scope until requested.
- **Installer** (MSI/MSIX) that also registers `idr://`, plus signed releases.

## Android (`android/`)

- BitTorrent support (cross-compile libtorrent + boost for Android).
- Additional ABIs (`armeabi-v7a`, `x86_64`), each needing a verified vcpkg triplet.

## Browser (`browser/`)

- Handoff confirmation: the app could acknowledge receipt (e.g. through a
  localhost endpoint), so auto-capture could cancel the browser download safely.
- Replace the six copies with one source plus a small packaging script.
