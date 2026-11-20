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

/*
 * Returns raw UTF-8 Kotlin source for the snippet, using the same required-size
 * buffer contract. Mode: 0 dualtone, 1 monochrome. Boolean arguments use
 * zero/nonzero. Negative errors: -1 invalid symbol pointer, -2 invalid UTF-8,
 * -3 invalid output pointer, -4 internal failure, -5 result too large,
 * -6 unknown symbol, -7 invalid mode.
 */
intptr_t sfstudio_compose_snippet(const uint8_t *symbol_name, size_t symbol_len,
                                  int32_t mode, int32_t tint_argb,
                                  int32_t tint_enabled, int32_t background_argb,
                                  int32_t background_enabled, uint8_t *output,
                                  size_t output_capacity);

/*
 * Toggle a name in a JSON array of favorites, preserving order. Negative errors:
 * -1 invalid input pointer, -2 invalid UTF-8, -3 invalid output pointer,
 * -4 internal failure, -5 result too large, -8 invalid favorites JSON.
 */
intptr_t sfstudio_favorites_toggle_json(const uint8_t *favorites_json,
                                        size_t favorites_json_len,
                                        const uint8_t *symbol_name,
                                        size_t symbol_name_len, uint8_t *output,
                                        size_t output_capacity);

#ifdef __cplusplus
}
#endif

#endif
