# Manual end-to-end check for the desktop download engine

There is no automated test suite. This harness exercises the real app the same way
the browser extensions do (`idr://` handoff) against a local server whose failure
modes are deterministic. Run it after changing anything under `core/src/service/`.

## 1. Start the server

```bash
node core/tests/e2e/range_server.js      # listens on http://127.0.0.1:8765
```

It writes the expected SHA-256 of every file to `%TEMP%\idr-e2e-expected.json` and a
request log to `%TEMP%\idr-e2e-requests.log`.

| Path | Behaviour | What it proves |
| --- | --- | --- |
| `/big.bin` | 24 MiB, honours `Range` | segmented download + assembly |
| `/flaky.bin` | 24 MiB, drops the first connection of every segment after 1 MiB | per-segment retry resumes from the `.partN` size |
| `/small.bin` | 300 KiB | single-stream path |
| `/norange.bin` | HEAD advertises ranges, GET ignores them (200 full body) | validator rejects before writing, then single-stream fallback |
| `/missing.bin` | 404 | permanent error, no file created, message in Status column |

## 2. Point the app at a scratch folder

Close Internet Downloader, then write `%APPDATA%\InternetDownloader\settings.json`
(or use **Downloads > Options...**). Use Node or the dialog — hand-escaping
backslashes in JSON is error-prone:

```bash
node -e "require('fs').writeFileSync(process.env.APPDATA+'/InternetDownloader/settings.json', JSON.stringify({downloadDir: process.argv[1], maxConcurrentDownloads: 1, maxSegments: 8, maxRetries: 3, speedLimitKBps: 4096}))" "C:\\temp\\idr-e2e"
```

`maxConcurrentDownloads: 1` exercises the queue; `speedLimitKBps: 4096` makes
`big.bin` take ~6 s instead of <1 s.

## 3. Hand off URLs like the browser extension does

```bash
EXE=core/build/Debug/InternetDownloader.exe
enc(){ node -e "console.log(encodeURIComponent(process.argv[1]))" "$1"; }
"$EXE" "idr://$(enc http://127.0.0.1:8765/big.bin)" &      # first launch opens the window
sleep 3
for f in flaky.bin small.bin norange.bin missing.bin; do
  "$EXE" "idr://$(enc http://127.0.0.1:8765/$f)"           # forwarded over the named pipe
  sleep 1
done
```

## 4. Check results

- `sha256sum` of each finished file matches `%TEMP%\idr-e2e-expected.json`.
- `%APPDATA%\InternetDownloader\logs\idr.log` shows: segment retries for `flaky.bin`,
  "falling back to a single stream" for `norange.bin`, `HTTP 404` for `missing.bin`,
  and (with the queue at 1) each download starting only after the previous one completes.
- No `.partN` / `.assembling` files remain next to completed downloads.

Pause/resume, restart and removal still need clicking through the GUI; see
`docs/DOWNLOAD_CORRECTNESS_CHECKLIST.md` for the full release checklist.

Afterwards delete the scratch folder and `%APPDATA%\InternetDownloader` if you
don't want to keep the test history.
