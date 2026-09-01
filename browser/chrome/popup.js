// popup.js — Internet Downloader Extension Popup Controller

document.addEventListener("DOMContentLoaded", async () => {
  const urlInput = document.getElementById("urlInput");
  const pasteBtn = document.getElementById("pasteBtn");
  const downloadBtn = document.getElementById("downloadBtn");
  const interceptToggle = document.getElementById("interceptToggle");
  const openAppBtn = document.getElementById("openAppBtn");
  const downloadPageMediaBtn = document.getElementById("downloadPageMediaBtn");
  const toast = document.getElementById("toast");

  // Load saved settings
  const settings = await chrome.storage.local.get({ interceptDownloads: false });
  interceptToggle.checked = settings.interceptDownloads;

  // Handle intercept toggle change
  interceptToggle.addEventListener("change", (e) => {
    chrome.storage.local.set({ interceptDownloads: e.target.checked });
    showToast(e.target.checked ? "Auto-capture enabled" : "Auto-capture disabled");
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
      showToast("Please enter a valid URL");
      urlInput.focus();
      return;
    }

    chrome.runtime.sendMessage({ action: "downloadUrl", url: url }, (response) => {
      showToast("Download sent to desktop app!");
      urlInput.value = "";
    });
  });

  // Open desktop app via custom protocol
  openAppBtn.addEventListener("click", () => {
    window.location.href = "idr://open";
    showToast("Launching Internet Downloader...");
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
          showToast(`Found ${urls.length} media items on page`);
          // Send first item to input
          urlInput.value = urls[0];
        } else {
          showToast("No downloadable media found on page");
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

// Function executed in page context
function extractMediaUrls() {
  const media = [];
  document.querySelectorAll("a[href], video source[src], audio source[src], img[src]").forEach((el) => {
    const src = el.href || el.src;
    if (src && (src.endsWith(".zip") || src.endsWith(".rar") || src.endsWith(".exe") ||
                src.endsWith(".iso") || src.endsWith(".tar.gz") || src.endsWith(".mp4") ||
                src.endsWith(".mp3") || src.endsWith(".pdf") || src.endsWith(".bin"))) {
      media.push(src);
    }
  });
  return media;
}
