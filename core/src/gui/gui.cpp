/*
 * gui.cpp – main GUI state, update, render loop
 */
#include "../../include/gui/gui.h"
#include "../../include/gui/sidebar.h"
#include "../../include/gui/toolbar.h"
#include "../../include/gui/download_row.h"
#include <cstring>
#include <cstdio>
#include <raylib.h>

Font g_ui_font = { 0 };

/* ── demo data ───────────────────────────── */
static void add_dl(GuiState *s, const char *name, const char *url,
                   float prog, float spd, float sz, int eta, DlStatus st)
{
    if (s->dl_count >= 64) return;
    Download *d = &s->downloads[s->dl_count++];
    strncpy(d->filename, name, 127);
    strncpy(d->url,      url,  255);
    d->progress  = prog;
    d->speed_kbs = spd;
    d->size_mb   = sz;
    d->eta_s     = eta;
    d->status    = st;
}

/* ── simulate download progress ─────────── */
static void simulate(GuiState *s) {
    float dt = GetFrameTime();
    for (int i = 0; i < s->dl_count; i++) {
        Download *d = &s->downloads[i];
        if (d->status != DL_DOWNLOADING || d->progress >= 1.f) continue;
        float chunk = (d->speed_kbs / 1024.f) * dt / d->size_mb;
        d->progress += chunk;
        if (d->progress >= 1.f) {
            d->progress  = 1.f;
            d->status    = DL_COMPLETED;
            d->speed_kbs = 0.f;
            d->eta_s     = 0;
        } else {
            float rem_mb = d->size_mb * (1.f - d->progress);
            d->eta_s = (d->speed_kbs > 0.f)
                       ? (int)(rem_mb * 1024.f / d->speed_kbs) : 0;
        }
    }
}

/* ── count helpers ───────────────────────── */
static int count_status(GuiState *s, DlStatus st) {
    int n = 0;
    for (int i = 0; i < s->dl_count; i++)
        if (s->downloads[i].status == st) n++;
    return n;
}

/* ── Add-download dialog ─────────────────── */
static void draw_add_dialog(GuiState *s) {
    if (!s->show_add_dlg) return;
    int sw = GetScreenWidth(), sh = GetScreenHeight();
    DrawRectangle(0, 0, sw, sh, {0, 0, 0, 160});

    Rectangle dlg = {(float)(sw/2 - 280), (float)(sh/2 - 100), 560, 200};
    DrawRectangleRounded(dlg, 0.07f, 12, COL_SURFACE2);
    DrawRectangleRoundedLinesEx(dlg, 0.07f, 12, 1.5f, COL_BORDER);

    DrawText("Add New Download", (int)(dlg.x + 20), (int)(dlg.y + 18), FZ_LG, COL_TEXT);
    DrawText("Enter URL or magnet link:", (int)(dlg.x+20), (int)(dlg.y+55), FZ_SM, COL_TEXT_MUTED);

    /* input */
    Rectangle inp = {dlg.x+20, dlg.y+78, dlg.width-40, 36};
    DrawRectangleRounded(inp, 0.2f, 8, COL_SURFACE3);
    DrawRectangleRoundedLinesEx(inp, 0.2f, 8, 1.5f, COL_BORDER);
    DrawText(s->add_url[0] ? s->add_url : "https://...",
             (int)(inp.x+10), (int)(inp.y+10), FZ_SM,
             s->add_url[0] ? COL_TEXT : COL_TEXT_DIM);
    /* blinking cursor */
    if (s->add_url_len > 0 && (int)(GetTime()*2) % 2 == 0) {
        int tw = MeasureText(s->add_url, FZ_SM);
        DrawRectangle((int)(inp.x+10+tw), (int)(inp.y+8), 2, 20, COL_ACCENT);
    }

    /* buttons */
    Rectangle cancel = {dlg.x+dlg.width-180, dlg.y+152, 76, 30};
    Rectangle ok_btn = {dlg.x+dlg.width- 96, dlg.y+152, 76, 30};
    bool ch = CheckCollisionPointRec(GetMousePosition(), cancel);
    bool oh = CheckCollisionPointRec(GetMousePosition(), ok_btn);
    DrawRectangleRounded(cancel, 0.3f, 8, ch ? COL_SURFACE3 : COL_SURFACE);
    DrawRectangleRounded(ok_btn, 0.3f, 8, oh ? COL_ACCENT_LT : COL_ACCENT);
    DrawText("Cancel", (int)(cancel.x+14), (int)(cancel.y+8), FZ_SM, COL_TEXT_MUTED);
    DrawText("Add",    (int)(ok_btn.x+22), (int)(ok_btn.y+8), FZ_SM, COL_WHITE);

    /* keyboard */
    int key = GetCharPressed();
    while (key > 0) {
        if (key >= 32 && s->add_url_len < 511)
            s->add_url[s->add_url_len++] = (char)key;
        s->add_url[s->add_url_len] = '\0';
        key = GetCharPressed();
    }
    if (IsKeyPressed(KEY_BACKSPACE) && s->add_url_len > 0)
        s->add_url[--s->add_url_len] = '\0';

    bool confirm = IsKeyPressed(KEY_ENTER) || (oh && IsMouseButtonReleased(MOUSE_LEFT_BUTTON));
    bool cancel_act = IsKeyPressed(KEY_ESCAPE) || (ch && IsMouseButtonReleased(MOUSE_LEFT_BUTTON));
    if (confirm && s->add_url_len > 0)
        add_dl(s, "New Download", s->add_url, 0.f, 0.f, 0.f, 0, DL_QUEUED);
    if (confirm || cancel_act) s->show_add_dlg = false;
}

/* ── content area ────────────────────────── */
static void draw_content(GuiState *s) {
    int sw = GetScreenWidth(), sh = GetScreenHeight();
    int cx = SIDEBAR_W, cy = TOOLBAR_H;
    int cw = sw - cx, ch = sh - cy;

    DrawRectangle(cx, cy, cw, ch, COL_BG);

    static const char *titles[VIEW_COUNT] = {
        "Active Downloads", "Completed", "Failed", "Torrents", "Settings"
    };
    DrawText(titles[s->active_view], cx + PAD, cy + PAD, FZ_LG, COL_TEXT);

    if (s->active_view == VIEW_SETTINGS) {
        DrawText("General settings coming soon.",
                 cx + PAD, cy + PAD + FZ_LG + 20, FZ_MD, COL_TEXT_MUTED);
        return;
    }

    int list_top = cy + PAD + FZ_LG + PAD;
    int y = list_top - (int)s->scroll_offset;
    int count = 0;

    for (int i = 0; i < s->dl_count; i++) {
        Download *dl = &s->downloads[i];

        /* filter */
        bool show = false;
        switch (s->active_view) {
            case VIEW_ACTIVE:
                show = (dl->status == DL_DOWNLOADING ||
                        dl->status == DL_PAUSED      ||
                        dl->status == DL_QUEUED);
                break;
            case VIEW_COMPLETED: show = (dl->status == DL_COMPLETED); break;
            case VIEW_FAILED:    show = (dl->status == DL_FAILED);    break;
            case VIEW_TORRENTS:
                show = (strncmp(dl->url, "magnet:", 7) == 0 || dl->size_mb >= 600.f);
                break;
            default: break;
        }
        if (!show) continue;

        Rectangle row = {(float)(cx + PAD), (float)y,
                         (float)(cw - PAD * 2), (float)ROW_H};

        /* only render if on-screen */
        if (y + ROW_H > list_top && y < sh) {
            bool rh = CheckCollisionPointRec(GetMousePosition(), row);
            bool rs = (s->selected == i);
            download_row_render(s, dl, row, rs, rh);
            if (rh && IsMouseButtonReleased(MOUSE_LEFT_BUTTON) && !s->show_add_dlg)
                s->selected = rs ? -1 : i;
        }
        y += ROW_H + ROW_GAP;
        count++;
    }

    /* empty state */
    if (count == 0) {
        const char *msg = "Nothing here yet.";
        int tw = MeasureText(msg, FZ_MD);
        DrawText(msg, cx + (cw - tw)/2, cy + ch/2, FZ_MD, COL_TEXT_DIM);
    }

    /* scroll */
    if (!s->show_add_dlg) {
        float wheel = GetMouseWheelMove();
        s->scroll_offset -= wheel * 44.f;
        float max_s = (float)(count * (ROW_H + ROW_GAP));
        if (s->scroll_offset < 0.f) s->scroll_offset = 0.f;
        if (s->scroll_offset > max_s) s->scroll_offset = max_s;
    }
}

/* ════════════════════════════════════════════
 *  Public C API
 * ════════════════════════════════════════════ */
extern "C" void gui_init(GuiState *s) {
    memset(s, 0, sizeof(*s));
    s->active_view = VIEW_ACTIVE;
    s->selected    = -1;

    /* Load system font */
    g_ui_font = LoadFontEx("C:\\Windows\\Fonts\\segoeui.ttf", 32, 0, 0);
    SetTextureFilter(g_ui_font.texture, TEXTURE_FILTER_BILINEAR);

    add_dl(s, "Ubuntu-24.04-desktop-amd64.iso",
           "https://releases.ubuntu.com/24.04/ubuntu-24.04-desktop-amd64.iso",
           0.95f, 2340.f, 5300.f, 82,  DL_DOWNLOADING);

    add_dl(s, "VSCode-win32-x64-1.92.0.exe",
           "https://update.code.visualstudio.com/1.92.0/win32-x64/stable",
           0.62f, 1120.f, 94.f, 28,   DL_DOWNLOADING);

    add_dl(s, "node-v20.12.0-x64.msi",
           "https://nodejs.org/dist/v20.12.0/node-v20.12.0-x64.msi",
           0.30f, 540.f,  30.f, 37,   DL_DOWNLOADING);

    add_dl(s, "python-3.12.3-amd64.exe",
           "https://www.python.org/ftp/python/3.12.3/python-3.12.3-amd64.exe",
           0.00f, 0.f,    25.f, 0,    DL_QUEUED);

    add_dl(s, "chrome_installer.exe",
           "https://dl.google.com/chrome/install/375.126/chrome_installer.exe",
           0.45f, 0.f,    83.f, 0,    DL_PAUSED);

    add_dl(s, "Big.Buck.Bunny.1080p.torrent",
           "magnet:?xt=urn:btih:dd8255ecdc7ca55fb0bbf81323d87062db1f6d1c",
           0.38f, 820.f,  726.f, 720, DL_DOWNLOADING);

    add_dl(s, "git-2.45.0-64-bit.exe",
           "https://github.com/git-for-windows/git/releases/v2.45.0/git-2.45.0-64-bit.exe",
           1.00f, 0.f,    61.f, 0,    DL_COMPLETED);

    add_dl(s, "vlc-3.0.21-win64.exe",
           "https://get.videolan.org/vlc/3.0.21/win64/vlc-3.0.21-win64.exe",
           1.00f, 0.f,    44.f, 0,    DL_COMPLETED);

    add_dl(s, "debian-12.5.0-amd64-DVD-1.iso",
           "https://cdimage.debian.org/debian-cd/12.5.0/amd64/iso-dvd/",
           0.00f, 0.f,    3900.f, 0,  DL_FAILED);
}

extern "C" void gui_update(GuiState *s) {
    simulate(s);
}

extern "C" void gui_render(GuiState *s) {
    ClearBackground(COL_BG);

    int n_active = count_status(s, DL_DOWNLOADING)
                 + count_status(s, DL_PAUSED)
                 + count_status(s, DL_QUEUED);
    int n_done   = count_status(s, DL_COMPLETED);
    int n_fail   = count_status(s, DL_FAILED);

    int n_torrent = 0;
    for (int i = 0; i < s->dl_count; i++)
        if (strncmp(s->downloads[i].url, "magnet:", 7) == 0 ||
            s->downloads[i].size_mb >= 600.f) n_torrent++;

    sidebar_render(s, n_active, n_done, n_fail, n_torrent);
    toolbar_render(s);
    draw_content(s);
    draw_add_dialog(s);
}

extern "C" void gui_cleanup(GuiState *s) {
    (void)s;
}
