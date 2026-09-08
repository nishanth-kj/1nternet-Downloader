// background.js — Internet Downloader Universal Browser Extension
//
// Hands links off to the desktop app via the "idr://" custom protocol, which
// InternetDownloader.exe registers with Windows (see core/scripts/register_protocol.ps1).
// Opening idr://<percent-encoded-url> in a background tab makes the OS launch (or
// forward to an already-running) Internet Downloader with that URL, exactly like
// clicking a magnet: or mailto: link — no native-messaging host install required.
// Firefox exposes the same chrome.* callback-style APIs used here as a compatibility
// shim, so this file is shared with the Chromium-based extensions.

// Initialize Context Menus
chrome.runtime.onInstalled.addListener(() => {
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

// Handle Context Menu clicks
chrome.contextMenus.onClicked.addListener((info) => {
  let targetUrl = info.linkUrl || info.srcUrl || info.pageUrl;
  if (targetUrl) {
    sendDownloadToApp(targetUrl);
  }
});

// Intercept browser downloads (including .torrent, .zip, .exe, etc.) if enabled
chrome.downloads.onCreated.addListener(async (downloadItem) => {
  const settings = await chrome.storage.local.get({
    interceptDownloads: true,
    interceptTorrents: true
  });

  const isTorrent = downloadItem.url && (downloadItem.url.startsWith("magnet:") || downloadItem.url.endsWith(".torrent") || (downloadItem.filename && downloadItem.filename.endsWith(".torrent")));

  if ((settings.interceptDownloads || (isTorrent && settings.interceptTorrents)) && downloadItem.url) {
    chrome.downloads.cancel(downloadItem.id);
    chrome.downloads.erase({ id: downloadItem.id });
    sendDownloadToApp(downloadItem.url);
  }
});

// Hands a URL/magnet link off to the desktop app via the idr:// protocol handler.
// Opens it in a short-lived background tab: by the time we remove the tab, the
// browser has already dispatched the "Open Internet Downloader?" OS prompt.
function sendDownloadToApp(url) {
  const customSchemeUrl = `idr://${encodeURIComponent(url)}`;

  chrome.tabs.create({ url: customSchemeUrl, active: false }, (tab) => {
    if (chrome.runtime.lastError || !tab) {
      showNotification("Could Not Reach Internet Downloader", url.length > 70 ? url.substring(0, 70) + "..." : url);
      return;
    }
    setTimeout(() => {
      chrome.tabs.remove(tab.id, () => void chrome.runtime.lastError);
    }, 1500);
  });

  showNotification("Sent to Internet Downloader", url.length > 70 ? url.substring(0, 70) + "..." : url);
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

// Message listener from popup
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  if (request.action === "downloadUrl") {
    sendDownloadToApp(request.url);
    sendResponse({ success: true });
  }
  return true;
});
