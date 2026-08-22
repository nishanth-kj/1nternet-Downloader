/*
 * toolbar.cpp – top action bar
 */
#include "../../include/gui/toolbar.h"
#include <raylib.h>

static bool icon_button(const char *label, Rectangle r, Color bg, Color hover_bg, Color tc) {
    bool h = CheckCollisionPointRec(GetMousePosition(), r);
    DrawRectangleRounded(r, 0.3f, 8, h ? hover_bg : bg);
    int tw = MeasureText(label, FZ_SM);
    DrawText(label,
             (int)(r.x + (r.width  - tw)  / 2),
             (int)(r.y + (r.height - FZ_SM) / 2),
             FZ_SM, tc);
    return h && IsMouseButtonReleased(MOUSE_LEFT_BUTTON);
}

extern "C" void toolbar_render(GuiState *s) {
    int sw  = GetScreenWidth();
    int cx  = SIDEBAR_W;
    int cw  = sw - cx;

    DrawRectangle(cx, 0, cw, TOOLBAR_H, COL_SURFACE2);
    DrawRectangle(cx, TOOLBAR_H - 1, cw, 1, COL_BORDER);

    float x  = cx + PAD;
    float cy = TOOLBAR_H / 2.f;

    /* Add Download */
    if (icon_button("+ Add Download",
                    {x, cy - 14.f, 132.f, 28.f},
                    COL_ACCENT, COL_ACCENT_LT, COL_WHITE))
    {
        s->show_add_dlg = true;
        s->add_url[0]   = '\0';
        s->add_url_len  = 0;
    }
    x += 142.f;

    /* Pause All */
    icon_button("Pause All",
                {x, cy - 14.f, 86.f, 28.f},
                COL_SURFACE3, COL_SURFACE, COL_TEXT_MUTED);
    x += 96.f;

    /* Resume All */
    icon_button("Resume All",
                {x, cy - 14.f, 94.f, 28.f},
                COL_SURFACE3, COL_SURFACE, COL_TEXT_MUTED);

    /* Speed readout */
    DrawText("2.4 MB/s \x19",
             sw - 126, (int)(cy - 7), FZ_SM, COL_SUCCESS);
}
