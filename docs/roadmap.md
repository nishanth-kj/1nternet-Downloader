# Roadmap

## What works today (desktop)

- Segmented HTTP(S) downloads with work stealing, a single stream when ranges aren't
  supported, and live speed, time left and segment map.
- Torrents through libtorrent: magnet, http(s) `.torrent` and local `.torrent`
  sources; pause, resume, remove with files; restored after restart.
- Classic UI: categories, 13 sortable columns, download window, "Download complete",
  batch add, clipboard monitoring, themes and download colour, start with Windows.
- Browser handoff through `idr://` and a single instance with a named pipe.
- Browser extension with video and audio detection and an on-video download button.

## Known bugs (fix first)

1. **Ranged replies aren't validated.** `stream_range` accepts `200` and doesn't check
   `Content-Range`, so a server that ignores `Range` corrupts the file.
2. **Stolen segments overrun.** After work stealing shrinks a segment, its worker keeps
   writing to the old end.
3. **HTTP errors are shown as Complete.** For example, a 404 finishes as "Complete".
4. **Segmented resume isn't saved.** `segments` helpers exist but the engine never
   calls them.
5. **Limits aren't enforced.** Simultaneous downloads and the speed limit are saved but
   ignored.

## Tooling to update

- `core/tests/e2e/README.md` describes the C++ app (it needs the owner's OK to change).
- Local `core/build/` and `core/out/` folders are leftovers from the C++ build and can be deleted.

## Torrent stages

qBittorrent features, written in Rust (with C++ only inside the libtorrent bridge).
qBittorrent is used as a reference, not copied.

1. **Done:** add magnet, http(s) and local `.torrent` links; pause, resume, remove;
   seeds and peers in the list.
2. **Stage 2:** torrent details window with files (choose and prioritise), trackers,
   peers and pieces; upload speed and ratio; seeding limits; saved fast-resume data.
3. **Stage 3:** categories and tags; global and per-torrent speed limits; an
   alternative-speed schedule; queue limits.
4. **Stage 4:** RSS feeds with auto-download rules; a torrent creator.
5. **Stage 5:** torrent search in Rust. You add the search sources; none are bundled.

## Desktop features after that

Scheduler and named queues, site logins and cookies, proxy settings, user agent,
per-download speed limit, HLS/DASH stream downloads, and an installer.

## Android and browser

- Android: test range validation and pause/resume on a physical device each release.
- Extension: stream (HLS/DASH) downloads once the desktop app supports them.
