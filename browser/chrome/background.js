// background.js — Internet Downloader Browser Extension

const NATIVE_APP_NAME = "com.internetdownloader.nativehost";

// Initialize Context Menus
chrome.runtime.onInstalled.addListener(() => {
  chrome.contextMenus.create({
    id: "idr_download_link",
    title: "Download with Internet Downloader",
    contexts: ["link", "image", "video", "audio"]
  });

  chrome.contextMenus.create({
    id: "idr_download_page",
    title: "Download Page with Internet Downloader",
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

// Intercept browser downloads if enabled
chrome.downloads.onCreated.addListener(async (downloadItem) => {
  const settings = await chrome.storage.local.get({ interceptDownloads: false });
  if (settings.interceptDownloads && downloadItem.url) {
    // Cancel browser download and forward to Internet Downloader
    chrome.downloads.cancel(downloadItem.id);
    chrome.downloads.erase({ id: downloadItem.id });
    sendDownloadToApp(downloadItem.url, downloadItem.filename || "");
  }
});

// Dispatch URL to native app or trigger custom URI scheme
function sendDownloadToApp(url, title) {
  // Option 1: Custom protocol uri handler: idr://<url>
  const customSchemeUrl = `idr://${encodeURIComponent(url)}`;
  
  // Option 2: Try native messaging host if configured
  try {
    const port = chrome.runtime.connectNative(NATIVE_APP_NAME);
    port.postMessage({ action: "download", url: url, title: title });
    port.onDisconnect.addListener(() => {
      // Fallback: Notify user
      showNotification("Download Sent", `Forwarded to Internet Downloader: ${url.substring(0, 50)}...`);
    });
  } catch (e) {
    showNotification("Download Captured", `Captured URL: ${url.substring(0, 50)}...`);
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
