// background.js — Internet Downloader Universal Browser Extension
//
// Hands links off to the desktop app via the "idr://" custom protocol, which
// InternetDownloader.exe registers with Windows (see core/scripts/register_protocol.ps1).
// Opening idr://<percent-encoded-url> in a background tab makes the OS launch (or
// forward to an already-running) Internet Downloader with that URL, exactly like
// clicking a magnet: or mailto: link — no native-messaging host install required.
// Note: on macOS this extension must be packaged as a Safari Web Extension (via
// Xcode's safari-web-extension-converter) for the idr:// handoff to reach a macOS
// build of Internet Downloader; this file targets the shared WebExtension API surface.
//
// It also watches network responses for audio/video files and streaming playlists,
// keeps a per-tab list (shown in the popup and by the on-video download button in
// content.js), and badges the toolbar icon with the count.

// ---- media detection --------------------------------------------------------------

const MEDIA_EXT = /\.(mp4|m4v|webm|mkv|mov|avi|wmv|flv|3gp|ogv|mp3|m4a|aac|ogg|oga|opus|wav|flac|wma|m3u8|mpd)(?:$|[?#])/i;
const SEGMENT_EXT = /\.(ts|m4s|aac\.part|cmfv|cmfa)(?:$|[?#])/i;
const MIN_FILE_BYTES = 256 * 1024; // smaller responses are usually previews, ads or fragments
const MAX_ITEMS_PER_TAB = 60;

function headerValue(headers, name) {
  const h = (headers || []).find((x) => x.name.toLowerCase() === name);
  return h ? h.value : "";
}

// Classifies a response as a downloadable file, a streaming playlist, or nothing.
function classifyMedia(url, mime) {
  let parsed;
  try {
    parsed = new URL(url);
  } catch {
    return null;
  }
  if (!["http:", "https:"].includes(parsed.protocol)) return null;
  const path = parsed.pathname;
  const type = (mime || "").split(";")[0].trim().toLowerCase();

  if (type === "application/vnd.apple.mpegurl" || type === "application/x-mpegurl" || /\.m3u8$/i.test(path)) {
    return { kind: "hls", type: "HLS stream" };
  }
  if (type === "application/dash+xml" || /\.mpd$/i.test(path)) {
    return { kind: "dash", type: "DASH stream" };
  }
  // Pieces of adaptive streams are useless on their own.
  if (SEGMENT_EXT.test(path) || type === "video/mp2t" || type === "video/iso.segment") return null;
  // Players that fetch byte ranges via query parameters are streaming fragments.
  if (/[?&](range|bytestart|byteend)=/i.test(parsed.search)) return null;

  const isMediaType = type.startsWith("video/") || type.startsWith("audio/");
  if (!isMediaType && !MEDIA_EXT.test(path)) return null;
  const ext = (path.match(/\.([a-z0-9]{2,5})$/i) || [])[1];
  const label = ext ? ext.toUpperCase() : (type.split("/")[1] || "media").toUpperCase();
  return { kind: type.startsWith("audio/") || /^(mp3|m4a|aac|ogg|oga|opus|wav|flac|wma)$/i.test(ext || "") ? "audio" : "video", type: label };
}

// Total size from Content-Range ("bytes 0-99/12345") or Content-Length.
function responseSize(headers, statusCode) {
  const range = headerValue(headers, "content-range");
  const m = /\/(\d+)\s*$/.exec(range);
  if (m) return Number(m[1]);
  const len = Number(headerValue(headers, "content-length"));
  return statusCode === 200 && Number.isFinite(len) ? len : 0;
}

function nameFromUrl(url) {
  try {
    const last = decodeURIComponent(new URL(url).pathname.split("/").filter(Boolean).pop() || "");
    return last || new URL(url).hostname;
  } catch {
    return url;
  }
}

// The list lives in storage.session so it survives the service worker being suspended.
function tabKey(tabId) {
  return `media_${tabId}`;
}

function getTabMedia(tabId) {
  return new Promise((resolve) => {
    chrome.storage.session.get({ [tabKey(tabId)]: [] }, (data) => {
      resolve(chrome.runtime.lastError ? [] : data[tabKey(tabId)]);
    });
  });
}

function setTabMedia(tabId, items) {
  chrome.storage.session.set({ [tabKey(tabId)]: items }, () => void chrome.runtime.lastError);
  const n = items.length;
  chrome.action.setBadgeText({ tabId, text: n ? String(n) : "" }, () => void chrome.runtime.lastError);
  if (n) chrome.action.setBadgeBackgroundColor({ tabId, color: "#16a34a" }, () => void chrome.runtime.lastError);
}

// storage.session writes are async, so serialize updates per tab to avoid lost items.
const tabLocks = new Map();
function updateTabMedia(tabId, fn) {
  const prev = tabLocks.get(tabId) || Promise.resolve();
  const next = prev.then(async () => {
    const items = await getTabMedia(tabId);
    const out = fn(items);
    if (out) setTabMedia(tabId, out);
  });
  tabLocks.set(tabId, next.catch(() => {}));
  return next;
}

chrome.webRequest.onHeadersReceived.addListener(
  (details) => {
    if (details.tabId < 0 || details.statusCode < 200 || details.statusCode >= 300) return;
    const mime = headerValue(details.responseHeaders, "content-type");
    const media = classifyMedia(details.url, mime);
    if (!media) return;
    const size = responseSize(details.responseHeaders, details.statusCode);
    if ((media.kind === "video" || media.kind === "audio") && size > 0 && size < MIN_FILE_BYTES) return;

    updateTabMedia(details.tabId, (items) => {
      const existing = items.find((i) => i.url === details.url);
      if (existing) {
        if (size > existing.size) {
          existing.size = size;
          return items;
        }
        return null;
      }
      items.push({
        url: details.url,
        name: nameFromUrl(details.url),
        kind: media.kind,
        type: media.type,
        size,
        frame: details.frameId === 0 ? "page" : "frame",
        seenAt: Date.now(),
      });
      return items.slice(-MAX_ITEMS_PER_TAB);
    });
  },
  { urls: ["<all_urls>"], types: ["media", "xmlhttprequest", "other", "object", "sub_frame"] },
  ["responseHeaders"]
);

// A new page in the tab starts a fresh list.
chrome.tabs.onUpdated.addListener((tabId, changeInfo) => {
  if (changeInfo.status === "loading" && changeInfo.url) {
    updateTabMedia(tabId, () => []);
  }
});

chrome.tabs.onRemoved.addListener((tabId) => {
  chrome.storage.session.remove(tabKey(tabId), () => void chrome.runtime.lastError);
  tabLocks.delete(tabId);
});

// Best single pick for "download this video": the largest direct file, video before audio.
function bestMedia(items) {
  const files = items.filter((i) => i.kind === "video" || i.kind === "audio");
  files.sort((a, b) => (a.kind === b.kind ? b.size - a.size : a.kind === "video" ? -1 : 1));
  return files[0] || null;
}

// ---- context menus ----------------------------------------------------------------

chrome.runtime.onInstalled.addListener(() => {
  chrome.storage.local.get({ videoButton: null }, (s) => {
    if (s.videoButton === null) chrome.storage.local.set({ videoButton: true });
  });

  chrome.contextMenus.create({
    id: "idr_download_link",
    title: "Download with Internet Downloader",
    contexts: ["link", "image", "video", "audio"]
  });

  chrome.contextMenus.create({
    id: "idr_download_torrent",
    title: "Download Torrent / Magnet with Internet Downloader",
    contexts: ["link"]
  });

  chrome.contextMenus.create({
    id: "idr_download_page",
    title: "Download All Media on Page",
    contexts: ["page"]
  });
});

chrome.contextMenus.onClicked.addListener(async (info, tab) => {
  let targetUrl = info.linkUrl || info.srcUrl || info.pageUrl;
  // Players that stream through MediaSource expose a blob: URL; use what the network saw instead.
  if ((info.mediaType === "video" || info.mediaType === "audio") && targetUrl && !/^https?:/i.test(targetUrl) && tab) {
    const pick = bestMedia(await getTabMedia(tab.id));
    if (!pick) {
      showNotification("No downloadable file found", "This player streams in pieces or uses protection, so there's no single file to send.");
      return;
    }
    targetUrl = pick.url;
  }
  if (targetUrl) {
    sendDownloadToApp(targetUrl);
  }
});

// ---- download capture -------------------------------------------------------------

// Intercept browser downloads (including .torrent, .zip, .exe, etc.) if enabled
chrome.downloads.onCreated.addListener(async (downloadItem) => {
  const settings = await chrome.storage.local.get({
    interceptDownloads: false,
    interceptTorrents: false
  });

  const isTorrent = downloadItem.url && (downloadItem.url.startsWith("magnet:") || downloadItem.url.endsWith(".torrent") || (downloadItem.filename && downloadItem.filename.endsWith(".torrent")));

  // Auto-capture is opt-in: a custom-protocol handoff cannot confirm that
  // the desktop app accepted the URL, so cancelling by default can lose downloads.
  if ((settings.interceptDownloads || (isTorrent && settings.interceptTorrents)) && downloadItem.url) {
    const allowedScheme = /^https?:/i.test(downloadItem.url) || /^magnet:/i.test(downloadItem.url);
    if (!allowedScheme) return;

    // Dispatch first. If the extension cannot even create the handoff tab,
    // leave the browser's original download untouched.
    sendDownloadToApp(downloadItem.url, (dispatched) => {
      if (!dispatched) return;
      chrome.downloads.cancel(downloadItem.id, () => {
        if (chrome.runtime.lastError) return;
        chrome.downloads.erase({ id: downloadItem.id });
      });
    });
  }
});

// Hands a URL/magnet link off to the desktop app via the idr:// protocol handler.
// Opens it in a short-lived background tab: by the time we remove the tab, the
// browser has already dispatched the "Open Internet Downloader?" OS prompt.
function sendDownloadToApp(url, onDispatched = () => {}) {
  let parsed;
  try {
    parsed = new URL(url);
  } catch {
    showNotification("Unsupported download URL", "Enter a valid HTTP(S) URL or magnet link.");
    onDispatched(false);
    return;
  }

  if (!["http:", "https:", "magnet:"].includes(parsed.protocol)) {
    showNotification("Unsupported download URL", "Only HTTP, HTTPS, and magnet links are supported.");
    onDispatched(false);
    return;
  }

  const customSchemeUrl = `idr://${encodeURIComponent(url)}`;
  chrome.tabs.create({ url: customSchemeUrl, active: false }, (tab) => {
    if (chrome.runtime.lastError || !tab) {
      showNotification("Could Not Reach Internet Downloader", url.length > 70 ? url.substring(0, 70) + "..." : url);
      onDispatched(false);
      return;
    }
    showNotification("Handoff requested", "Confirm that Internet Downloader opens before relying on the transfer.");
    setTimeout(() => {
      chrome.tabs.remove(tab.id, () => void chrome.runtime.lastError);
    }, 1500);
    onDispatched(true);
  });
}

function showNotification(title, message) {
  chrome.notifications.create({
    type: "basic",
    iconUrl: "icons/icon48.png",
    title: title,
    message: message,
    priority: 1
  });
}

// ---- messages from the popup and content script -----------------------------------

chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  if (request.action === "downloadUrl") {
    sendDownloadToApp(request.url, (dispatched) => {
      sendResponse({ success: dispatched });
    });
    return true;
  }

  if (request.action === "getMedia") {
    const tabId = request.tabId ?? (sender.tab && sender.tab.id);
    if (tabId == null) {
      sendResponse({ items: [] });
      return false;
    }
    getTabMedia(tabId).then((items) => sendResponse({ items }));
    return true;
  }

  // The content script reports <video>/<audio> sources that are plain files.
  if (request.action === "reportElementMedia" && sender.tab && Array.isArray(request.urls)) {
    updateTabMedia(sender.tab.id, (items) => {
      let changed = false;
      for (const url of request.urls.slice(0, 20)) {
        if (items.some((i) => i.url === url)) continue;
        const media = classifyMedia(url, "");
        if (!media) continue;
        items.push({ url, name: nameFromUrl(url), kind: media.kind, type: media.type, size: 0, frame: "element", seenAt: Date.now() });
        changed = true;
      }
      return changed ? items.slice(-MAX_ITEMS_PER_TAB) : null;
    });
    return false;
  }

  if (request.action === "clearMedia") {
    updateTabMedia(request.tabId, () => []).then(() => sendResponse({ ok: true }));
    return true;
  }

  return false;
});
