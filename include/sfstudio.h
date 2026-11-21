#ifndef SFSTUDIO_H
#define SFSTUDIO_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Allocate a writable byte buffer for a C ABI call; null means size is zero or allocation failed. */
uint8_t *sfstudio_alloc(size_t size);

/* Free a buffer returned by sfstudio_alloc using the same size. */
void sfstudio_free(uint8_t *buffer, size_t size);

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
 * Interpolate a symbol's outline points for a weight and optical scale.
 * scale: 0 medium, 1 small, 2 large. Returns the required/written count of
 * float coordinates in screen coordinates (y down). output_capacity is a
 * count of floats, not bytes. A short buffer is left untouched. Errors: -1
 * invalid name pointer, -2 invalid UTF-8, -3 invalid output pointer, -4
 * internal failure, -5 too large, -6 unknown symbol, -9 invalid scale, -10
 * non-finite weight.
 */
intptr_t sfstudio_glyph_points(const uint8_t *symbol_name,
                               size_t symbol_name_len, int32_t scale,
                               float weight, float *output,
                               size_t output_capacity);

/*
 * Evaluate Apple-style symbol effects and optional replace transitions from a
 * UTF-8 JSON request. `time` and transition `time` are finite, nonnegative
 * seconds. Request: {"symbol":"wifi","time":0.4,
 * "effect":{"type":"bounce","direction":"up","scope":"by_layer"}}.
 * Draw On/Off additionally accept "individually":true to run one motion
 * group at a time; "scope":"by_layer" staggers the group start times.
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
 * Build 48-step Draw On/Off reveal levels from a target-rendered 112x112
 * binary mask. Input JSON: {"mask":[0,1,...],"origin":[x,y],"cell":size,
 * "symbolBounds":[left,top,right,bottom],"ties":[u,v,...]}. The caller
 * rasterizes the path to preserve its platform's antialiasing. Output JSON
 * contains row-major levels (0..47; 255 outside the shape), or null for an
 * empty mask. Invalid draw requests return -12; other errors use the common
 * buffer/error contract described above.
 */
intptr_t sfstudio_draw_field_json(const uint8_t *input, size_t input_len,
                                  uint8_t *output, size_t output_capacity);

/*
 * Generate the JSON MCP client configuration shown in desktop Settings.
 * `app_path_present` is 1 for an installed app launcher, or 0 with a zero
 * `app_path_len` to generate the development classpath configuration. The
 * development library path is resolved by the host. Invalid flags return
 * -13; other errors use the common buffer/error contract above.
 */
intptr_t sfstudio_mcp_client_config_json(
    const uint8_t *app_path, size_t app_path_len, int32_t app_path_present,
    const uint8_t *development_lib_path, size_t development_lib_path_len,
    uint8_t *output, size_t output_capacity);

/*
 * Catalog filter selection transition. action: 0 category, 1 favorites,
 * 2 all. category_present distinguishes null from an empty category string.
 * Returns a JSON pair [selectedCategory, onlyPinned]. Error -14 means invalid
 * action or inconsistent category arguments; other errors use the common
 * buffer/error contract above.
 */
intptr_t sfstudio_filter_selection_json(int32_t action,
                                        const uint8_t *category,
                                        size_t category_len,
                                        int32_t category_present,
                                        uint8_t *output,
                                        size_t output_capacity);

/* Resolve theme mode: 0 system, 1 light, 2 dark. Returns 0 light, 1 dark,
 * or -15 for an invalid mode/system-dark value. */
int32_t sfstudio_theme_mode_is_dark(int32_t theme_mode,
                                    int32_t system_is_dark);

/* Clamp the desktop symbol grid scale to the existing [0.5, 1.6] range. */
float sfstudio_grid_scale_clamp(float scale);

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
