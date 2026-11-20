#ifndef SFSTUDIO_H
#define SFSTUDIO_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/*
 * Search ranked SF Symbols metadata and encode Apple symbol names as UTF-8 JSON.
 * Returns the required/written byte length, or a negative error code. Call with
 * output == NULL and output_capacity == 0 to get the required length; a short
 * output buffer is left untouched and also returns the required length.
 * Errors: -1 invalid query pointer, -2 invalid UTF-8, -3 invalid output pointer,
 * -4 internal failure, -5 result too large for intptr_t.
 */
intptr_t sfstudio_search_json(const uint8_t *query, size_t query_len,
                              size_t limit, uint8_t *output,
                              size_t output_capacity);

#ifdef __cplusplus
}
#endif

#endif
