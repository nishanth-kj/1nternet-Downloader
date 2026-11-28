# browser/ — MV3 extension (six copies)

Docs: [docs/browser/extension.md](../docs/browser/extension.md)

`chrome/`, `edge/`, `brave/`, `opera/`, `firefox/`, `safari/`. No build step; load a
folder unpacked in Developer mode.

## Rules

- **Edit `chrome/` first, then copy.** `popup.js`, `content.js`, `popup.html` and
  `popup.css` are identical everywhere. `background.js` differs only in the header
  comments of `firefox/` and `safari/`. CI fails on any other drift.
- Manifests differ on purpose (Firefox uses `background.scripts` and a `gecko` id);
  edit each one. They all need `scripting`, `webRequest` and `<all_urls>`.
- Capturing browser downloads stays **off by default**. Cancel the browser's download
  only after the handoff tab was created.
- Forward only `http:`, `https:` and `magnet:` links. Say "handoff requested", never
  "download started".
- Don't try to download DRM-protected media.
- Use the `chrome.*` callback APIs and check `chrome.runtime.lastError`.
- Check every copy: `node --check <copy>/{background,popup,content}.js`.
