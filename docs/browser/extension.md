# Browser extension

Manifest V3, plain JavaScript, no build step. One extension, kept as six copies:
`browser/chrome`, `edge`, `brave`, `opera`, `firefox`, `safari`.

## Files

| File | Role |
| --- | --- |
| `background.js` | Service worker: context menus, optional download capture, `idr://` handoff, media detection, toolbar badge |
| `content.js` | "Download video" button over playing videos, with a quality picker |
| `popup.html/.js/.css` | Popup: add a link, media found on this page, toggles |
| `manifest.json` | Differs per browser on purpose (Firefox: `background.scripts` and a `gecko` id) |

## Handing a link to the app

`background.js` opens `idr://<encodeURIComponent(url)>` in a background tab and
closes it after 1.5 s. Windows passes the URL to `1idm.exe`.

A custom protocol can't confirm that the app accepted the link, so:

- Messages say "handoff requested", never "download started".
- Capturing browser downloads is **off by default**. When it's on, the browser's own
  download is cancelled only after the handoff tab was created.
- Only `http:`, `https:` and `magnet:` links are forwarded.

## Video and audio detection

- `webRequest.onHeadersReceived` watches responses in each tab. It records:
  - **files:** `video/*`, `audio/*`, or a media extension, at least 256 KB;
  - **streams:** HLS (`.m3u8`) and DASH (`.mpd`) playlists.

  It skips stream fragments (`.ts`, `.m4s`, `range=` query parameters).
- The list is kept per tab in `chrome.storage.session` (it survives the service worker
  being suspended). It's cleared on navigation, and its size is shown as the toolbar
  badge.
- `content.js` shows a green button above a hovered or playing video (200×110 px or
  larger). One file sends straight away; several open a picker sorted by size.
  "Hide on this site" stores the host in `videoButtonOff`.
- For players that use `blob:` sources, the video context menu sends the largest
  detected file instead.
- HLS/DASH entries are listed but disabled: the desktop app can't download streams yet.
  DRM-protected media isn't supported and won't be.

## Settings (`chrome.storage.local`)

`interceptDownloads`, `interceptTorrents` (both off by default), `videoButton` (on),
and `videoButtonOff` (list of hosts).

## Keeping the six copies in sync

Edit `browser/chrome/` first, then copy:

- `popup.js`, `content.js`, `popup.html` and `popup.css` are identical in every copy.
- `background.js` is identical except for extra header comments in `firefox/` and
  `safari/`.
- Manifests are edited one by one. Any manifest using `chrome.scripting` needs the
  `scripting` permission. Media detection needs `webRequest` and `<all_urls>`.

CI (`build-core.yml`) runs `node --check` on each script and fails on any drift.

## Trying it

Load `browser/<name>/` as an unpacked extension (Developer mode). To test without
your normal profile, use a temporary browser profile, for example
`msedge --user-data-dir=<temp> --load-extension=<path>`.
