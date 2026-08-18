#include <stdio.h>
#include <raylib.h>
#include "raygui.h"

int main(int argc, char *argv[]) {
    const int screenWidth = 800;
    const int screenHeight = 600;

    InitWindow(screenWidth, screenHeight, "Internet Downloader");

    SetTargetFPS(60);

    while (!WindowShouldClose()) {
        BeginDrawing();
            ClearBackground(RAYWHITE);
            
            if (GuiButton((Rectangle){ 300, 250, 200, 50 }, "Click Me!")) {
                printf("Button clicked!\n");
            }
            
        EndDrawing();
    }

    CloseWindow();
    return 0;
}
