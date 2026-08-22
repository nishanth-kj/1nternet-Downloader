/*
 * download_row.cpp – one card per download item
 */
#include "../../include/gui/download_row.h"
#include <cstdio>
#include <cstring>
#include <raylib.h>

/* ── helpers ──────────────────────────────── */
static const char *status_label(DlStatus s) {
    switch (s) {
        case DL_DOWNLOADING: return "Downloading";
        case DL_PAUSED:      return "Paused";
        case DL_COMPLETED:   return "Completed";
        case DL_FAILED:      return "Failed";
        case DL_QUEUED:      return "Queued";
        default:             return "Unknown";
    }
}

static Color status_color(DlStatus s) {
    switch (s) {
        case DL_DOWNLOADING: return COL_ACCENT;
        case DL_PAUSED:      return COL_PAUSE;
        case DL_COMPLETED:   return COL_SUCCESS;
        case DL_FAILED:      return COL_ERROR;
        default:             return COL_TEXT_MUTED;
    }
}

static void fmt_size(float mb, char *buf, int n) {
    if (mb >= 1024.f) snprintf(buf, n, "%.1f GB", mb / 1024.f);
    else              snprintf(buf, n, "%.1f MB", mb);
}

static void fmt_speed(float kbs, char *buf, int n) {
    if (kbs >= 1024.f) snprintf(buf, n, "%.1f MB/s", kbs / 1024.f);
    else               snprintf(buf, n, "%.0f KB/s",  kbs);
}

static void fmt_eta(int s, char *buf, int n) {
    if (s <= 0)        snprintf(buf, n, "--");
    else if (s < 60)   snprintf(buf, n, "%ds",     s);
    else if (s < 3600) snprintf(buf, n, "%dm %ds", s/60, s%60);
    else               snprintf(buf, n, "%dh %dm", s/3600, (s%3600)/60);
}

/* ── public ───────────────────────────────── */
extern "C" void download_row_render(GuiState  *s,
                                    Download  *dl,
                                    Rectangle  b,
                                    bool       selected,
                                    bool       hovered)
{
    (void)s;
    Color sc  = status_color(dl->status);

    /* card bg */
    Color bg = selected ? COL_ALPHA(COL_ACCENT, 22)
             : hovered  ? COL_ALPHA(COL_WHITE,   5)
             :            COL_SURFACE;
    DrawRectangleRounded(b, 0.10f, 8, bg);
    if (selected)
        DrawRectangleRoundedLinesEx(b, 0.10f, 8, 1.5f, COL_ALPHA(COL_ACCENT, 80));

    /* left status stripe */
    DrawRectangle((int)b.x, (int)(b.y + 10), 3, (int)(b.height - 20), sc);

    float cx = b.x + PAD + 8;
    float ty = b.y + PAD;
    float rw = b.width - PAD * 2 - 8;

    /* ── row 1: filename ── */
    DrawText(dl->filename, (int)cx, (int)ty, FZ_MD, COL_TEXT);

    /* size (right) */
    char sz[32]; fmt_size(dl->size_mb, sz, sizeof(sz));
    int szw = MeasureText(sz, FZ_SM);
    DrawText(sz, (int)(b.x + b.width - PAD - szw), (int)ty, FZ_SM, COL_TEXT_MUTED);

    /* status badge */
    const char *sl = status_label(dl->status);
    int bw = MeasureText(sl, FZ_XS) + 10;
    float bx = b.x + b.width - PAD - szw - bw - 8.f;
    DrawRectangleRounded({bx, ty - 1.f, (float)bw, 18.f}, 0.5f, 6, COL_ALPHA(sc, 30));
    DrawText(sl, (int)(bx + 5), (int)(ty + 2), FZ_XS, sc);

    /* ── row 2: URL ── */
    DrawText(dl->url, (int)cx, (int)(ty + 20), FZ_XS, COL_TEXT_DIM);

    /* ── row 3: progress bar ── */
    float bar_y = ty + 42.f;
    DrawRectangleRounded({cx, bar_y, rw, 6.f}, 0.5f, 4, COL_SURFACE3);
    if (dl->progress > 0.f) {
        float fw = rw * dl->progress;
        if (fw > 4.f) {
            DrawRectangleRounded({cx, bar_y, fw, 6.f}, 0.5f, 4, sc);
            DrawRectangleRounded({cx, bar_y, fw, 3.f}, 0.5f, 4, COL_ALPHA(COL_WHITE, 15));
        }
    }

    /* ── row 4: stats ── */
    float iy = bar_y + 14.f;
    char pct[16]; snprintf(pct, sizeof(pct), "%.0f%%", dl->progress * 100.f);
    DrawText(pct, (int)cx, (int)iy, FZ_SM, sc);

    if (dl->status == DL_DOWNLOADING || dl->status == DL_PAUSED) {
        char spd[32], eta[32], info[80];
        fmt_speed(dl->speed_kbs, spd, sizeof(spd));
        fmt_eta(dl->eta_s, eta, sizeof(eta));
        snprintf(info, sizeof(info), "%s  \xb7  ETA %s", spd, eta);
        DrawText(info, (int)(cx + 48), (int)iy, FZ_SM, COL_TEXT_MUTED);
    }

    /* ── action buttons ── */
    float bby = iy - 2.f;
    float bbx = b.x + b.width - PAD;
    int   bsz = 22;

    /* Delete */
    bbx -= bsz + 4.f;
    Rectangle del = {bbx, bby, (float)bsz, (float)bsz};
    bool dh = CheckCollisionPointRec(GetMousePosition(), del);
    DrawRectangleRounded(del, 0.3f, 6, dh ? COL_ALPHA(COL_ERROR, 40) : COL_SURFACE2);
    DrawText("x", (int)(bbx + (bsz - MeasureText("x", FZ_XS)) / 2),
             (int)(bby + (bsz - FZ_XS) / 2), FZ_XS,
             dh ? COL_ERROR : COL_TEXT_DIM);

    /* Pause / Resume */
    if (dl->status == DL_DOWNLOADING || dl->status == DL_PAUSED) {
        bbx -= bsz + 6.f;
        const char *ic = (dl->status == DL_DOWNLOADING) ? "||" : ">";
        Rectangle pb = {bbx, bby, (float)bsz, (float)bsz};
        bool ph = CheckCollisionPointRec(GetMousePosition(), pb);
        DrawRectangleRounded(pb, 0.3f, 6, ph ? COL_SURFACE3 : COL_SURFACE2);
        int iw = MeasureText(ic, FZ_XS);
        DrawText(ic, (int)(bbx + (bsz - iw) / 2),
                 (int)(bby + (bsz - FZ_XS) / 2), FZ_XS, COL_TEXT_MUTED);
        if (ph && IsMouseButtonReleased(MOUSE_LEFT_BUTTON))
            dl->status = (dl->status == DL_DOWNLOADING) ? DL_PAUSED : DL_DOWNLOADING;
    }
}
