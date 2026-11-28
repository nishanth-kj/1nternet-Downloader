# Testing

A successful build says nothing about whether downloads are correct. Say what you
verified, and what you didn't, in every change.

## Automated

| Product | Command |
| --- | --- |
| Desktop | `cd core && cargo test` |
| Extension | `node --check browser/<name>/{background,popup,content}.js`, plus the drift check in CI |
| Website | `cd site && npm run lint && npm run build` |

## Desktop: manual end-to-end check

Run this after any change under `core/src/service/` or `core/cpp/`:

1. Start a local test server, either `node core/tests/e2e/range_server.js` (port 8765:
   segmented, flaky, small, range-ignoring and 404 files) or one of your own.
2. Run the app with a scratch `APPDATA` (see [building](../desktop/building.md)).
3. Send it URLs: `target\debug\1idm.exe "http://127.0.0.1:8765/big.bin"` hands the
   URL to the running instance.
4. Compare SHA-256s against `%TEMP%\idr-e2e-expected.json`. Check pause, resume,
   restart and remove by hand.
5. For torrents, use a legal test torrent (for example an official Linux image), check
   that it reaches Downloading with data on disk, then remove it with its files.

`core/tests/e2e/README.md` still describes the old C++ app; follow the steps above
instead.

## Release checklist

### HTTP(S)

- [ ] A normal 200 download matches the expected SHA-256.
- [ ] Ranged downloads get `206` with a matching `Content-Range`.
- [ ] A server that ignores `Range` is never treated as a successful segmented download.
- [ ] Redirects, 401/403, 404, 429 and 5xx are reported as errors, not Complete.
- [ ] Unknown length and chunked responses don't show a false 100%.
- [ ] A truncated or changed remote file isn't marked Complete.

### Lifecycle

- [ ] Pause stops writing and keeps a resumable partial file.
- [ ] Resume checks the offset and the server's `Content-Range`.
- [ ] Remove, restart and closing the app don't leave workers writing.
- [ ] After a restart, unfinished downloads aren't shown as Complete.

### Files

- [ ] Existing files, missing folders, permission errors and a full disk give clear errors.
- [ ] A failed download never leaves a partial file that looks complete.

### Torrents

- [ ] Magnet, http(s) `.torrent` and local `.torrent` sources all start.
- [ ] Pause sticks, resume works, and remove with files deletes the data.

### Extension

- [ ] Capture is off by default; a failed handoff leaves the browser's download running.
- [ ] Only http, https and magnet links are forwarded.

### Android

- [ ] Range validation, segment checks and pause/resume, tested on a physical device.

### Release

- [ ] Clean Windows and Android builds; website lint and build.
- [ ] Artifacts on a versioned GitHub release with SHA-256 checksums and notes.
- [ ] Skipped checks and known limits are listed in the release notes.
