# browser/ — MV3 extensions

Six copies of one extension: `chrome/`, `edge/`, `brave/`, `opera/`, `firefox/`, `safari/`.
There is no build step; load a folder unpacked in Developer mode.

## How it works

`background.js` registers context menus and, when the user opts in, auto-captures
browser downloads. To hand a link to the desktop app it opens
`idr://<encodeURIComponent(url)>` in a short-lived background tab. Windows passes
that tab's URL to `InternetDownloader.exe` (registered by
`core/scripts/register_protocol.ps1`). `popup.js` provides manual add, toggles,
and "Grab Page Media" (`chrome.scripting` + `activeTab`).

## Rules

- **Edit `chrome/` first, then copy the change to the other five.** `popup.js` is
  byte-identical everywhere. `background.js` differs only in the header comments of
  `firefox/` and `safari/`. CI (`build-core.yml`) fails on any other drift.
- Auto-capture stays **off by default**. A custom-protocol handoff can't confirm
  receipt, so cancel the browser download only after the handoff tab was created.
- Forward only `http:`, `https:` and `magnet:` URLs.
- Manifests differ on purpose (Firefox: `background.scripts` + `gecko` id).
  Any manifest that uses `chrome.scripting` needs the `scripting` permission.
- Use the `chrome.*` callback APIs and check `chrome.runtime.lastError`.
- Check every copy with `node --check <browser>/background.js` and `node --check <browser>/popup.js`.
