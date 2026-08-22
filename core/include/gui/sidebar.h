#pragma once
#include "gui.h"

#ifdef __cplusplus
extern "C" {
#endif

void sidebar_render(GuiState *s,
                    int dl_active,
                    int dl_done,
                    int dl_failed,
                    int dl_torrent);

#ifdef __cplusplus
}
#endif
