// popup.js — Internet Downloader Extension Controller

document.addEventListener("DOMContentLoaded", async () => {
  const urlInput = document.getElementById("urlInput");
  const pasteBtn = document.getElementById("pasteBtn");
  const downloadBtn = document.getElementById("downloadBtn");
  const interceptToggle = document.getElementById("interceptToggle");
  const torrentToggle = document.getElementById("torrentToggle");
  const openAppBtn = document.getElementById("openAppBtn");
  const downloadPageMediaBtn = document.getElementById("downloadPageMediaBtn");
  const toast = document.getElementById("toast");

  // Load saved settings
  const settings = await chrome.storage.local.get({
    interceptDownloads: false,
    interceptTorrents: false
  });
  interceptToggle.checked = settings.interceptDownloads;
  if (torrentToggle) torrentToggle.checked = settings.interceptTorrents;

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

    chrome.runtime.sendMessage({ action: "downloadUrl", url }, () => {
      if (chrome.runtime.lastError) {
        showToast("Could not contact the extension service");
        return;
      }
      showToast("Handoff requested; confirm the desktop app opens");
      urlInput.value = "";
    });
  });

  // Open desktop app via custom protocol
  openAppBtn.addEventListener("click", () => {
    window.location.href = "idr://open";
    showToast("Opening Internet Downloader...");
  });

  // Grab page media / links
  downloadPageMediaBtn.addEventListener("click", async () => {
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
    if (tab && tab.id) {
      chrome.scripting.executeScript({
        target: { tabId: tab.id },
        func: extractMediaUrls
      }, (results) => {
        if (results && results[0] && results[0].result && results[0].result.length > 0) {
          const urls = results[0].result;
          showToast(`Found ${urls.length} downloadable links on page`);
          urlInput.value = urls[0];
        } else {
          showToast("No downloadable files/torrents found on page");
        }
      });
    }
  });

  function showToast(msg) {
    toast.textContent = msg;
    toast.classList.remove("hidden");
    setTimeout(() => {
      toast.classList.add("hidden");
    }, 2500);
  }
});

function extractMediaUrls() {
  const media = [];
  document.querySelectorAll("a[href], video source[src], audio source[src], img[src]").forEach((el) => {
    const src = el.href || el.src;
    if (src && (src.startsWith("magnet:") || src.endsWith(".torrent") ||
                src.endsWith(".zip") || src.endsWith(".rar") || src.endsWith(".exe") ||
                src.endsWith(".iso") || src.endsWith(".tar.gz") || src.endsWith(".mp4") ||
                src.endsWith(".mp3") || src.endsWith(".bin") || src.endsWith(".mkv"))) {
      media.push(src);
    }
  });
  return media;
}
