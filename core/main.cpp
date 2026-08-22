/*
 * main.cpp — Internet Downloader entry point
 *
 * The core/ directory is its own self-contained C++ project.
 * This file initialises Raylib, bootstraps the GUI state, and
 * runs the main render loop.  All logic lives in core/src/.
 */

#include <raylib.h>
#include "include/gui/gui.h"      /* GuiState, gui_init/update/render/cleanup */

/* ─────────────────────────────────────────────────────────────────
 *  Entry point
 * ───────────────────────────────────────────────────────────────── */
int main(void)
{
    /* ── window ── */
    SetConfigFlags(FLAG_WINDOW_RESIZABLE | FLAG_MSAA_4X_HINT);
    InitWindow(1200, 750, "Internet Downloader");
    SetTargetFPS(60);
    SetWindowMinSize(900, 600);

    /* ── state ── */
    GuiState state;
    gui_init(&state);

    /* ── main loop ── */
    while (!WindowShouldClose())
    {
        gui_update(&state);

        BeginDrawing();
            gui_render(&state);
        EndDrawing();
        SwapScreenBuffer();
        PollInputEvents();
    }

    /* ── cleanup ── */
    gui_cleanup(&state);
    CloseWindow();
    return 0;
}
