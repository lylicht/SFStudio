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
 * Search with optional exact category and favorites filters. Empty category
 * disables category filtering. favorites_only is zero/false or nonzero/true;
 * when true, favorites_json must contain a JSON string array. Error -8 means
 * invalid favorites JSON. Uses the same UTF-8 output buffer contract above.
 */
intptr_t sfstudio_search_filtered_json(const uint8_t *query, size_t query_len,
                                       const uint8_t *category,
                                       size_t category_len,
                                       const uint8_t *favorites_json,
                                       size_t favorites_json_len,
                                       int32_t favorites_only, size_t limit,
                                       uint8_t *output,
                                       size_t output_capacity);

/*
 * Resolve Apple or Pascal name to SfSymbolMetadata JSON:
 * appleName, pascalName, categories, and isRestricted. Error -6 means unknown
 * symbol. Uses the same UTF-8 output buffer contract as sfstudio_search_json.
 */
intptr_t sfstudio_symbol_json(const uint8_t *symbol_name,
                              size_t symbol_name_len, uint8_t *output,
                              size_t output_capacity);

/*
 * Decode a symbol's geometry, animation hints, and layer tree as UTF-8 JSON.
 * scale: 0 medium, 1 small, 2 large. weight is the variable font wght axis
 * value. Geometry contains interpolated screen-coordinate points (y down) and
 * regular/light/heavy masters in raw font coordinates (y up); bounds are tight
 * visible outline bounds in screen coordinates. JSON includes path ops, layer
 * flags, colors, tags, contours, anchors, and optical variant status. Error -6
 * means unknown symbol, -9 invalid scale, and -10 non-finite weight. Other
 * errors use the common buffer/error contract described above.
 */
intptr_t sfstudio_glyph_json(const uint8_t *symbol_name,
                             size_t symbol_name_len, int32_t scale, float weight,
                             uint8_t *output, size_t output_capacity);

/*
 * Evaluate Apple-style symbol effects and optional replace transitions from a
 * UTF-8 JSON request. `time` and transition `time` are finite, nonnegative
 * seconds. Request: {"symbol":"wifi","time":0.4,
 * "effect":{"type":"bounce","direction":"up","scope":"by_layer"}}.
 * Effect types: appear, disappear, bounce, pulse, variable_color, scale,
 * wiggle, rotate, breathe, draw_on, draw_off. Each result layer contains its
 * transform and separate variableAlpha/variableDraw values. Optional
 * variableValue applies Apple's variable-layer ordering. Optional transition
 * is {"direction":"down_up","time":0.1}; directions are down_up, off_up,
 * and up_up. Optional magicReplace is {"from":"wifi","to":"wifi",
 * "progress":0.5}, with optional fromWeight/toWeight and fromScale/toScale
 * (small/medium/large). It returns matchedLayers with outline path operations
 * and morphed points. Returns the normal required/written JSON byte count;
 * -11 means an invalid request and -6 means unknown symbol.
 * Optional rendering is {"mode":"palette","tint":[r,g,b,a],
 * "palette":[[r,g,b,a],...],"gradient":true}; each animation layer then
 * includes its resolved color and optional top-to-bottom gradient stops.
 */
intptr_t sfstudio_animation_json(const uint8_t *input, size_t input_len,
                                 uint8_t *output, size_t output_capacity);

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
