#ifndef IDR_FAST_OPS_H
#define IDR_FAST_OPS_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* High-speed block CRC32 computation in C using slice-by-8/lookup */
uint32_t idr_fast_crc32(uint32_t crc, const uint8_t* buffer, size_t length);

/* Fast memory zero-check (for sparse file range validation) */
int idr_fast_is_all_zero(const uint8_t* buffer, size_t length);

#ifdef __cplusplus
}
#endif

#endif /* IDR_FAST_OPS_H */
