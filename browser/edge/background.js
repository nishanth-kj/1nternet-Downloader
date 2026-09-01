// background.js — Internet Downloader Universal Browser Extension

const NATIVE_APP_NAME = "com.internetdownloader.nativehost";

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
chrome.contextMenus.onClicked.addListener((info, tab) => {
  let targetUrl = info.linkUrl || info.srcUrl || info.pageUrl;
  if (targetUrl) {
    sendDownloadToApp(targetUrl, tab ? tab.title : "");
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
    sendDownloadToApp(downloadItem.url, downloadItem.filename || "");
  }
});

// Dispatch URL to native app
function sendDownloadToApp(url, title) {
  // Option 1: Custom protocol uri handler: idr://<url>
  const customSchemeUrl = `idr://${encodeURIComponent(url)}`;

  // Option 2: Try native messaging host if configured
  try {
    const port = chrome.runtime.connectNative(NATIVE_APP_NAME);
    port.postMessage({ action: "download", url: url, title: title });
    port.onDisconnect.addListener(() => {
      showNotification("Download Forwarded", `Sent to Internet Downloader: ${url.substring(0, 55)}...`);
    });
  } catch (e) {
    showNotification("Download Captured", `Captured: ${url.substring(0, 55)}...`);
  }
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
    sendDownloadToApp(request.url, request.title || "");
    sendResponse({ success: true });
  }
  return true;
});
