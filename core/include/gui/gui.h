#pragma once

#include <raylib.h>
#include "theme.h"

#ifdef __cplusplus
extern "C" {
#endif

extern Font g_ui_font;

/* Wrapper to replace default DrawText with our custom font */
#define DrawText(text, x, y, fontSize, color) \
    DrawTextEx(g_ui_font, text, Vector2{(float)(x), (float)(y)}, (float)(fontSize), 1.0f, color)

/* ───────────────── enums ───────────────── */
typedef enum {
    VIEW_ACTIVE = 0,
    VIEW_COMPLETED,
    VIEW_FAILED,
    VIEW_TORRENTS,
    VIEW_SETTINGS,
    VIEW_COUNT
} View;

typedef enum {
    DL_DOWNLOADING,
    DL_PAUSED,
    DL_COMPLETED,
    DL_FAILED,
    DL_QUEUED
} DlStatus;

/* ───────────────── data ────────────────── */
typedef struct {
    char     filename[128];
    char     url[256];
    float    progress;      /* 0.0 – 1.0       */
    float    speed_kbs;     /* KB/s             */
    float    size_mb;
    int      eta_s;
    DlStatus status;
} Download;

typedef struct {
    View     active_view;
    int      hovered_nav;

    Download downloads[64];
    int      dl_count;
    int      selected;          /* -1 = none        */

    bool     show_add_dlg;
    char     add_url[512];
    int      add_url_len;

    float    scroll_offset;
} GuiState;

/* ───────────────── API ─────────────────── */
void gui_init(GuiState *s);
void gui_update(GuiState *s);
void gui_render(GuiState *s);
void gui_cleanup(GuiState *s);

#ifdef __cplusplus
}
#endif
