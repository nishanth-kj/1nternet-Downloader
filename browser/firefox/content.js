// content.js — on-video "Download" button for Internet Downloader.
//
// Shows a small button over a <video> (or large <audio> player) while it's hovered or
// playing. Clicking it lists the media files the background script saw this tab load
// and hands the chosen one to the desktop app. Runs in every frame; all UI lives in a
// closed shadow root so page styles can't affect it (and it can't affect the page).

(() => {
  if (window.__idrVideoButton) return;
  window.__idrVideoButton = true;

  const MIN_W = 200;
  const MIN_H = 110;
  const HIDE_DELAY_MS = 2500;

  let enabled = true;
  let current = null; // the media element the button is attached to
  let hideTimer = null;
  let menuOpen = false;

  const host = document.createElement("div");
  host.style.cssText = "all:initial;position:fixed;top:0;left:0;width:0;height:0;z-index:2147483647;";
  const root = host.attachShadow({ mode: "closed" });
  root.innerHTML = `
    <style>
      :host { all: initial; }
      .wrap { position: fixed; display: none; font: 12px/1.3 "Segoe UI", system-ui, sans-serif; color: #1b1f24; }
      .wrap.show { display: block; }
      .btn { display: flex; align-items: center; gap: 6px; padding: 5px 10px 5px 8px; border: 1px solid #15803d;
             border-radius: 4px 4px 0 4px; background: #16a34a; color: #fff; cursor: pointer; font: inherit; font-weight: 600;
             box-shadow: 0 2px 8px rgba(0,0,0,.35); }
      .btn:hover { background: #15803d; }
      .btn svg { width: 15px; height: 15px; }
      .close { margin-left: 4px; opacity: .7; padding: 0 2px; }
      .close:hover { opacity: 1; }
      .menu { position: absolute; top: calc(100% + 4px); right: 0; min-width: 300px; max-width: 440px; max-height: 320px; overflow: auto;
              background: #fff; border: 1px solid #c9ced6; border-radius: 6px; box-shadow: 0 6px 22px rgba(0,0,0,.3); padding: 4px; display: none; }
      .menu.show { display: block; }
      .item { display: grid; grid-template-columns: 1fr auto; gap: 2px 10px; padding: 6px 8px; border-radius: 4px; cursor: pointer; }
      .item:hover { background: #e8f5e9; }
      .item.off { cursor: default; opacity: .6; }
      .item.off:hover { background: none; }
      .name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-weight: 600; }
      .meta { color: #5f6770; font-size: 11px; grid-column: 1 / -1; }
      .tag { font-size: 10px; padding: 1px 6px; border-radius: 8px; background: #e6e9ee; color: #1b1f24; align-self: center; }
      .note { padding: 8px; color: #5f6770; }
      .foot { border-top: 1px solid #e6e9ee; margin-top: 4px; padding: 6px 8px 2px; display: flex; justify-content: space-between; }
      .link { color: #16a34a; cursor: pointer; }
      .link:hover { text-decoration: underline; }
    </style>
    <div class="wrap" part="wrap">
      <button class="btn" type="button" title="Download with Internet Downloader">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12 3v12"/><path d="M7 10l5 5 5-5"/><path d="M5 21h14"/></svg>
        <span class="label">Download video</span>
        <span class="close" title="Hide">✕</span>
      </button>
      <div class="menu" role="menu"></div>
    </div>`;

  const wrap = root.querySelector(".wrap");
  const btn = root.querySelector(".btn");
  const label = root.querySelector(".label");
  const closeEl = root.querySelector(".close");
  const menu = root.querySelector(".menu");

  function attachHost() {
    if (!host.isConnected) (document.documentElement || document.body).appendChild(host);
  }

  function siteKey() {
    return location.hostname || "local";
  }

  chrome.storage.local.get({ videoButton: true, videoButtonOff: [] }, (s) => {
    if (chrome.runtime.lastError) return;
    enabled = s.videoButton !== false && !s.videoButtonOff.includes(siteKey());
  });
  chrome.storage.onChanged.addListener((changes, area) => {
    if (area !== "local") return;
    chrome.storage.local.get({ videoButton: true, videoButtonOff: [] }, (s) => {
      enabled = s.videoButton !== false && !s.videoButtonOff.includes(siteKey());
      if (!enabled) hide(true);
    });
  });

  function bigEnough(el) {
    const r = el.getBoundingClientRect();
    return r.width >= MIN_W && r.height >= MIN_H && r.bottom > 0 && r.right > 0 &&
      r.top < innerHeight && r.left < innerWidth;
  }

  function place() {
    if (!current || !current.isConnected) return hide(true);
    const r = current.getBoundingClientRect();
    if (r.width < MIN_W || r.height < MIN_H) return hide(true);
    // Sit just above the video's top-right corner; go inside the video when there's no room.
    const BTN_H = 30;
    const top = r.top - BTN_H - 2 >= 0 ? r.top - BTN_H - 2 : Math.max(r.top + 8, 4);
    const right = Math.max(innerWidth - r.right, 4);
    wrap.style.top = `${top}px`;
    wrap.style.right = `${right}px`;
    wrap.style.left = "auto";
  }

  function show(el) {
    if (!enabled || !bigEnough(el)) return;
    attachHost();
    current = el;
    label.textContent = el.tagName === "AUDIO" ? "Download audio" : "Download video";
    place();
    wrap.classList.add("show");
    clearTimeout(hideTimer);
    reportSource(el);
  }

  function hide(now) {
    clearTimeout(hideTimer);
    const run = () => {
      if (menuOpen && !now) return;
      wrap.classList.remove("show");
      menu.classList.remove("show");
      menuOpen = false;
      current = null;
    };
    if (now) run();
    else hideTimer = setTimeout(run, HIDE_DELAY_MS);
  }

  // Plain-file sources (not blob:/MediaSource) are added to the tab's list.
  function reportSource(el) {
    const urls = [el.currentSrc, el.src, ...Array.from(el.querySelectorAll("source")).map((s) => s.src)]
      .filter((u) => u && /^https?:/i.test(u));
    if (urls.length) chrome.runtime.sendMessage({ action: "reportElementMedia", urls: [...new Set(urls)] }, () => void chrome.runtime.lastError);
  }

  function fmtSize(n) {
    if (!n) return "size unknown";
    const u = ["B", "KB", "MB", "GB"];
    let i = 0;
    while (n >= 1024 && i < u.length - 1) { n /= 1024; i++; }
    return `${n.toFixed(i ? 1 : 0)} ${u[i]}`;
  }

  function send(url) {
    chrome.runtime.sendMessage({ action: "downloadUrl", url }, (res) => {
      if (chrome.runtime.lastError) return;
      label.textContent = res && res.success ? "Sent to downloader" : "Couldn't hand off";
      setTimeout(() => { if (current) label.textContent = current.tagName === "AUDIO" ? "Download audio" : "Download video"; }, 2000);
    });
    menu.classList.remove("show");
    menuOpen = false;
  }

  function el(tag, cls, text) {
    const e = document.createElement(tag);
    if (cls) e.className = cls;
    if (text != null) e.textContent = text;
    return e;
  }

  function renderMenu(items) {
    menu.textContent = "";
    const files = items.filter((i) => i.kind === "video" || i.kind === "audio")
      .sort((a, b) => (a.kind === b.kind ? b.size - a.size : a.kind === "video" ? -1 : 1));
    const streams = items.filter((i) => i.kind === "hls" || i.kind === "dash");

    if (!files.length && !streams.length) {
      menu.appendChild(el("div", "note",
        "No downloadable file seen yet. Start playing the video, then try again. Some sites stream in pieces or use protection, and those can't be sent as one file."));
    }
    for (const f of files) {
      const row = el("div", "item");
      row.setAttribute("role", "menuitem");
      row.appendChild(el("span", "name", f.name));
      row.appendChild(el("span", "tag", f.type));
      row.appendChild(el("span", "meta", `${f.kind === "audio" ? "Audio" : "Video"} · ${fmtSize(f.size)} · ${new URL(f.url).hostname}`));
      row.title = f.url;
      row.addEventListener("click", (e) => { e.stopPropagation(); send(f.url); });
      menu.appendChild(row);
    }
    for (const s of streams) {
      const row = el("div", "item off");
      row.appendChild(el("span", "name", s.name));
      row.appendChild(el("span", "tag", s.type));
      row.appendChild(el("span", "meta", "Streaming playlist: the desktop app can't download these yet"));
      row.title = s.url;
      menu.appendChild(row);
    }
    const foot = el("div", "foot");
    const off = el("span", "link", `Hide on ${siteKey()}`);
    off.addEventListener("click", (e) => {
      e.stopPropagation();
      chrome.storage.local.get({ videoButtonOff: [] }, (s) => {
        const list = new Set(s.videoButtonOff);
        list.add(siteKey());
        chrome.storage.local.set({ videoButtonOff: [...list] });
      });
      hide(true);
    });
    foot.appendChild(off);
    foot.appendChild(el("span", "meta", `${files.length} file(s)`));
    menu.appendChild(foot);
  }

  btn.addEventListener("click", (e) => {
    e.preventDefault();
    e.stopPropagation();
    if (e.composedPath().includes(closeEl)) return hide(true);
    if (menuOpen) {
      menu.classList.remove("show");
      menuOpen = false;
      return;
    }
    chrome.runtime.sendMessage({ action: "getMedia" }, (res) => {
      if (chrome.runtime.lastError) return;
      const items = (res && res.items) || [];
      const files = items.filter((i) => i.kind === "video" || i.kind === "audio");
      if (files.length === 1 && !items.some((i) => i.kind === "hls" || i.kind === "dash")) {
        send(files[0].url);
        return;
      }
      renderMenu(items);
      menu.classList.add("show");
      menuOpen = true;
    });
  });

  wrap.addEventListener("mouseenter", () => clearTimeout(hideTimer));
  wrap.addEventListener("mouseleave", () => hide(false));

  // Event delegation: works for players added later without scanning the DOM.
  document.addEventListener("mouseover", (e) => {
    const t = e.target;
    const media = t instanceof HTMLMediaElement ? t : null;
    if (media) return show(media);
    // Many players cover the <video> with an overlay; find a video under the pointer.
    if (!enabled) return;
    for (const v of document.querySelectorAll("video")) {
      const r = v.getBoundingClientRect();
      if (e.clientX >= r.left && e.clientX <= r.right && e.clientY >= r.top && e.clientY <= r.bottom) return show(v);
    }
    if (current && !menuOpen) hide(false);
  }, { passive: true, capture: true });

  document.addEventListener("play", (e) => {
    if (e.target instanceof HTMLMediaElement) {
      reportSource(e.target);
      show(e.target);
      hide(false);
    }
  }, true);

  document.addEventListener("click", (e) => {
    if (menuOpen && !e.composedPath().includes(host)) {
      menu.classList.remove("show");
      menuOpen = false;
      hide(false);
    }
  }, true);

  addEventListener("scroll", () => current && place(), { passive: true, capture: true });
  addEventListener("resize", () => current && place(), { passive: true });
  document.addEventListener("fullscreenchange", () => hide(true));
})();
