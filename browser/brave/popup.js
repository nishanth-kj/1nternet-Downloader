// popup.js — Internet Downloader Extension Controller

document.addEventListener("DOMContentLoaded", async () => {
  const urlInput = document.getElementById("urlInput");
  const pasteBtn = document.getElementById("pasteBtn");
  const downloadBtn = document.getElementById("downloadBtn");
  const interceptToggle = document.getElementById("interceptToggle");
  const torrentToggle = document.getElementById("torrentToggle");
  const videoButtonToggle = document.getElementById("videoButtonToggle");
  const openAppBtn = document.getElementById("openAppBtn");
  const downloadPageMediaBtn = document.getElementById("downloadPageMediaBtn");
  const clearMediaBtn = document.getElementById("clearMediaBtn");
  const mediaList = document.getElementById("mediaList");
  const hiddenSites = document.getElementById("hiddenSites");
  const appStatus = document.getElementById("appStatus");
  const toast = document.getElementById("toast");

  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  let pageLinks = []; // links found by "Scan links", shown with the network-detected media

  // Load saved settings
  const settings = await chrome.storage.local.get({
    interceptDownloads: false,
    interceptTorrents: false,
    videoButton: true,
    videoButtonOff: []
  });
  interceptToggle.checked = settings.interceptDownloads;
  if (torrentToggle) torrentToggle.checked = settings.interceptTorrents;
  videoButtonToggle.checked = settings.videoButton !== false;
  renderHiddenSites(settings.videoButtonOff);

  // Handle intercept toggles
  interceptToggle.addEventListener("change", (e) => {
    chrome.storage.local.set({ interceptDownloads: e.target.checked });
    showToast(e.target.checked ? "Auto-capture downloads enabled" : "Auto-capture downloads disabled");
  });

  if (torrentToggle) {
    torrentToggle.addEventListener("change", (e) => {
      chrome.storage.local.set({ interceptTorrents: e.target.checked });
      showToast(e.target.checked ? "Torrent capture enabled" : "Torrent capture disabled");
    });
  }

  videoButtonToggle.addEventListener("change", (e) => {
    chrome.storage.local.set({ videoButton: e.target.checked });
    showToast(e.target.checked ? "Download button shown on videos" : "Download button hidden on videos");
  });

  // Paste button handler
  pasteBtn.addEventListener("click", async () => {
    try {
      const text = await navigator.clipboard.readText();
      urlInput.value = text;
      urlInput.focus();
    } catch (err) {
      showToast("Unable to read clipboard");
    }
  });

  // Download button handler
  downloadBtn.addEventListener("click", () => {
    const url = urlInput.value.trim();
    if (!url) {
      showToast("Enter an HTTP(S) URL or magnet link");
      urlInput.focus();
      return;
    }

    let parsed;
    try {
      parsed = new URL(url);
    } catch {
      showToast("Enter a valid URL");
      urlInput.focus();
      return;
    }
    if (!["http:", "https:", "magnet:"].includes(parsed.protocol)) {
      showToast("Only HTTP, HTTPS, and magnet links are supported");
      urlInput.focus();
      return;
    }

    handOff(url, () => { urlInput.value = ""; });
  });

  // Open desktop app via custom protocol
  openAppBtn.addEventListener("click", () => {
    window.location.href = "idr://open";
    showToast("Opening Internet Downloader...");
  });

  // Scan the page's links and media elements for downloadable files.
  downloadPageMediaBtn.addEventListener("click", () => {
    if (!tab || !tab.id) return;
    chrome.scripting.executeScript({
      target: { tabId: tab.id, allFrames: true },
      func: extractMediaUrls
    }, (results) => {
      if (chrome.runtime.lastError) {
        showToast("This page can't be scanned");
        return;
      }
      const found = new Set();
      for (const r of results || []) for (const u of (r && r.result) || []) found.add(u);
      pageLinks = [...found];
      showToast(pageLinks.length ? `Found ${pageLinks.length} file link(s) on the page` : "No downloadable file links found on the page");
      loadMedia();
    });
  });

  clearMediaBtn.addEventListener("click", () => {
    if (!tab || tab.id == null) return;
    pageLinks = [];
    chrome.runtime.sendMessage({ action: "clearMedia", tabId: tab.id }, () => {
      void chrome.runtime.lastError;
      loadMedia();
    });
  });

  function handOff(url, onOk) {
    chrome.runtime.sendMessage({ action: "downloadUrl", url }, (response) => {
      if (chrome.runtime.lastError) {
        showToast("Could not contact the extension service");
        return;
      }
      if (!response || !response.success) {
        showToast("Could not start the handoff; browser download was not started");
        return;
      }
      showToast("Handoff requested; confirm the desktop app opens");
      if (onOk) onOk();
    });
  }

  function loadMedia() {
    if (!tab || tab.id == null) {
      renderMedia([]);
      return;
    }
    chrome.runtime.sendMessage({ action: "getMedia", tabId: tab.id }, (res) => {
      if (chrome.runtime.lastError) {
        renderMedia([]);
        return;
      }
      const items = (res && res.items) || [];
      const known = new Set(items.map((i) => i.url));
      for (const url of pageLinks) {
        if (known.has(url)) continue;
        items.push({ url, name: nameFromUrl(url), kind: url.startsWith("magnet:") ? "magnet" : "link", type: typeFromUrl(url), size: 0 });
      }
      renderMedia(items);
    });
  }

  function renderMedia(items) {
    mediaList.textContent = "";
    const order = { video: 0, audio: 1, link: 2, magnet: 3, hls: 4, dash: 5 };
    items.sort((a, b) => (order[a.kind] - order[b.kind]) || (b.size - a.size));
    const files = items.filter((i) => i.kind === "video" || i.kind === "audio").length;
    appStatus.textContent = files ? `${files} media found` : "Ready";

    if (!items.length) {
      const empty = document.createElement("div");
      empty.className = "media-empty";
      empty.textContent = "Nothing found yet. Play a video or audio on this page and it appears here, or use \"Scan links\" to list file links.";
      mediaList.appendChild(empty);
      return;
    }

    for (const item of items) {
      const unsupported = item.kind === "hls" || item.kind === "dash";
      const row = document.createElement("div");
      row.className = "media-item" + (unsupported ? " unsupported" : "");
      row.title = item.url;

      const name = document.createElement("div");
      name.className = "media-name";
      name.textContent = item.name;

      const meta = document.createElement("div");
      meta.className = "media-meta";
      const tag = document.createElement("span");
      tag.className = "media-tag";
      tag.textContent = item.type;
      meta.appendChild(tag);
      meta.appendChild(document.createTextNode(unsupported
        ? "Streaming playlist, not supported by the app yet"
        : `${kindLabel(item.kind)} · ${fmtSize(item.size)} · ${hostOf(item.url)}`));

      const actions = document.createElement("div");
      actions.className = "media-actions";
      const copy = document.createElement("button");
      copy.className = "mini-btn ghost";
      copy.textContent = "Copy";
      copy.addEventListener("click", async () => {
        try {
          await navigator.clipboard.writeText(item.url);
          showToast("Link copied");
        } catch {
          showToast("Couldn't copy the link");
        }
      });
      const dl = document.createElement("button");
      dl.className = "mini-btn";
      dl.textContent = "Download";
      dl.disabled = unsupported;
      dl.addEventListener("click", () => handOff(item.url));
      actions.appendChild(copy);
      actions.appendChild(dl);

      row.appendChild(name);
      row.appendChild(actions);
      row.appendChild(meta);
      mediaList.appendChild(row);
    }
  }

  function renderHiddenSites(list) {
    hiddenSites.textContent = "";
    if (!list || !list.length) return;
    hiddenSites.appendChild(document.createTextNode(`Video button hidden on: ${list.join(", ")}`));
    const reset = document.createElement("button");
    reset.className = "link-btn";
    reset.textContent = "Show everywhere";
    reset.addEventListener("click", () => {
      chrome.storage.local.set({ videoButtonOff: [] }, () => {
        renderHiddenSites([]);
        showToast("Video button shown on all sites");
      });
    });
    hiddenSites.appendChild(reset);
  }

  function showToast(msg) {
    toast.textContent = msg;
    toast.classList.remove("hidden");
    setTimeout(() => {
      toast.classList.add("hidden");
    }, 2500);
  }

  loadMedia();
});

function kindLabel(kind) {
  return { video: "Video", audio: "Audio", link: "Page link", magnet: "Magnet" }[kind] || "File";
}

function fmtSize(n) {
  if (!n) return "size unknown";
  const units = ["B", "KB", "MB", "GB"];
  let i = 0;
  while (n >= 1024 && i < units.length - 1) {
    n /= 1024;
    i++;
  }
  return `${n.toFixed(i ? 1 : 0)} ${units[i]}`;
}

function hostOf(url) {
  try {
    return new URL(url).hostname;
  } catch {
    return "";
  }
}

function nameFromUrl(url) {
  if (url.startsWith("magnet:")) {
    const dn = /[?&]dn=([^&]+)/.exec(url);
    return dn ? decodeURIComponent(dn[1].replace(/\+/g, " ")) : "Magnet link";
  }
  try {
    const last = new URL(url).pathname.split("/").filter(Boolean).pop();
    return last ? decodeURIComponent(last) : hostOf(url);
  } catch {
    return url;
  }
}

function typeFromUrl(url) {
  if (url.startsWith("magnet:")) return "MAGNET";
  const m = /\.([a-z0-9]{2,5})(?:$|[?#])/i.exec(url.split("?")[0]);
  return m ? m[1].toUpperCase() : "FILE";
}

// Runs inside the page: collects links and media sources that point at files.
function extractMediaUrls() {
  const fileExt = /\.(torrent|zip|rar|7z|tar|gz|tgz|bz2|xz|exe|msi|apk|dmg|iso|img|bin|pdf|epub|docx?|xlsx?|pptx?|mp4|m4v|mkv|webm|mov|avi|wmv|flv|mp3|m4a|aac|ogg|opus|wav|flac)(?:$|[?#])/i;
  const out = new Set();
  document.querySelectorAll("a[href], video[src], audio[src], video source[src], audio source[src]").forEach((el) => {
    const src = el.href || el.currentSrc || el.src;
    if (!src) return;
    if (src.startsWith("magnet:")) {
      out.add(src);
      return;
    }
    if (!/^https?:/i.test(src)) return;
    try {
      if (fileExt.test(new URL(src).pathname)) out.add(src);
    } catch {
      // ignore malformed URLs
    }
  });
  return [...out].slice(0, 200);
}
