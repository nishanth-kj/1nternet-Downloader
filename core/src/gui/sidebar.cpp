/*
 * sidebar.cpp – left navigation panel
 */
#include "../../include/gui/sidebar.h"
#include <cstdio>
#include <cstring>
#include <raylib.h>

/* ── helpers ─────────────────────────────── */
static void draw_nav_icon(int x, int y, const char *ch, Color bg, Color fg) {
    DrawRectangleRounded({(float)x, (float)y, 26.f, 26.f}, 0.4f, 6, bg);
    int tw = MeasureText(ch, FZ_SM);
    DrawText(ch, x + (26 - tw) / 2, y + 6, FZ_SM, fg);
}

static bool is_hovered(Rectangle r) {
    return CheckCollisionPointRec(GetMousePosition(), r);
}

/* ── public ──────────────────────────────── */
extern "C" void sidebar_render(GuiState *s,
                               int dl_active,
                               int dl_done,
                               int dl_failed,
                               int dl_torrent)
{
    int sw = SIDEBAR_W;
    int sh = GetScreenHeight();

    /* background */
    DrawRectangle(0, 0, sw, sh, COL_SURFACE);
    /* gradient header */
    DrawRectangleGradientV(0, 0, sw, TOOLBAR_H, COL_SURFACE2, COL_SURFACE);
    /* right border */
    DrawRectangle(sw - 1, 0, 1, sh, COL_BORDER);

    /* ── logo ── */
    DrawRectangleRounded({(float)PAD, 12.f, 32.f, 32.f}, 0.35f, 8, COL_ACCENT);
    DrawText("ID", PAD + 7, 21, FZ_SM, COL_WHITE);
    DrawText("Internet",    PAD + 40, 12, FZ_SM, COL_TEXT);
    DrawText("Downloader",  PAD + 40, 28, FZ_XS, COL_TEXT_MUTED);

    /* ── nav items ── */
    static const char *labels[VIEW_COUNT] = {
        "Active", "Completed", "Failed", "Torrents", "Settings"
    };
    static const char *icons[VIEW_COUNT]  = { "D", "C", "F", "T", "S" };
    static Color icon_bg[VIEW_COUNT] = {
        {99,102,241,30}, {34,197,94,30}, {239,68,68,30},
        {245,158,11,30}, {70,70,96,30}
    };
    static Color icon_fg[VIEW_COUNT] = {
        COL_ACCENT_LT, COL_SUCCESS, COL_ERROR, COL_WARNING, COL_TEXT_MUTED
    };

    int badges[VIEW_COUNT] = { dl_active, dl_done, dl_failed, dl_torrent, 0 };

    for (int i = 0; i < VIEW_COUNT; i++) {
        bool active  = (s->active_view == (View)i);
        Rectangle item = { 0.f, (float)(TOOLBAR_H + 10 + i * 50), (float)sw, 46.f };
        bool hovered = is_hovered(item) && !active;

        if (active) {
            DrawRectangle(0, (int)item.y, 3, 46, COL_ACCENT);
            DrawRectangle(3, (int)item.y, sw - 3, 46, COL_ALPHA(COL_ACCENT, 18));
        } else if (hovered) {
            DrawRectangle(0, (int)item.y, sw, 46, COL_ALPHA(COL_WHITE, 6));
        }

        Color ib = active ? COL_ALPHA(COL_ACCENT, 50) : icon_bg[i];
        Color ig = active ? COL_ACCENT_LT : icon_fg[i];
        draw_nav_icon(PAD, (int)item.y + 10, icons[i], ib, ig);

        DrawText(labels[i], PAD + 34, (int)item.y + 16,
                 FZ_MD, active ? COL_TEXT : COL_TEXT_MUTED);

        if (badges[i] > 0) {
            char buf[8];
            snprintf(buf, sizeof(buf), "%d", badges[i]);
            int bw = MeasureText(buf, FZ_XS) + 10;
            int bx = sw - bw - PAD;
            DrawRectangleRounded({(float)bx,(float)(item.y+14),(float)bw,18.f},
                                 0.5f, 6,
                                 active ? COL_ACCENT : COL_SURFACE3);
            DrawText(buf, bx + 5, (int)item.y + 17, FZ_XS, COL_WHITE);
        }

        if (is_hovered(item) && IsMouseButtonReleased(MOUSE_LEFT_BUTTON))
            s->active_view = (View)i;
    }

    /* ── version ── */
    DrawText("v1.0.0", PAD, sh - 22, FZ_XS, COL_TEXT_DIM);
}
