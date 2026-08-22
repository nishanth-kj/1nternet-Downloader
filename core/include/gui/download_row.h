#pragma once
#include "gui.h"

#ifdef __cplusplus
extern "C" {
#endif

void download_row_render(GuiState  *s,
                         Download  *dl,
                         Rectangle  bounds,
                         bool       selected,
                         bool       hovered);

#ifdef __cplusplus
}
#endif
