#pragma once
#include <raylib.h>

/* ─────────────────────────────────────────────
 *  Dark-indigo colour palette
 * ───────────────────────────────────────────── */
#define COL_BG          Color{ 10,  10,  18, 255}
#define COL_SURFACE     Color{ 16,  16,  26, 255}
#define COL_SURFACE2    Color{ 24,  24,  40, 255}
#define COL_SURFACE3    Color{ 32,  32,  52, 255}
#define COL_BORDER      Color{ 42,  42,  66, 255}
#define COL_ACCENT      Color{ 99, 102, 241, 255}
#define COL_ACCENT_LT   Color{129, 140, 248, 255}
#define COL_ACCENT_DK   Color{ 79,  70, 229, 255}
#define COL_TEXT        Color{226, 226, 242, 255}
#define COL_TEXT_MUTED  Color{120, 120, 158, 255}
#define COL_TEXT_DIM    Color{ 60,  60,  92, 255}
#define COL_SUCCESS     Color{ 34, 197,  94, 255}
#define COL_WARNING     Color{245, 158,  11, 255}
#define COL_ERROR       Color{239,  68,  68, 255}
#define COL_PAUSE       Color{249, 115,  22, 255}
#define COL_WHITE       Color{255, 255, 255, 255}
#define COL_ALPHA(c,a)  Color{(c).r,(c).g,(c).b,(unsigned char)(a)}

/* ─────────────────────────────────────────────
 *  Layout
 * ───────────────────────────────────────────── */
#define SIDEBAR_W   220
#define TOOLBAR_H    56
#define PAD          14
#define ROW_H       110
#define ROW_GAP       8

/* ─────────────────────────────────────────────
 *  Font sizes
 * ───────────────────────────────────────────── */
#define FZ_LG  18
#define FZ_MD  15
#define FZ_SM  13
#define FZ_XS  11
