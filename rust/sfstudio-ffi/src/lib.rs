//! C ABI for calling SFStudio's shared Rust catalog, code generation, and favorites from other languages.

use std::collections::HashSet;
use std::panic::{catch_unwind, AssertUnwindSafe};

const UNKNOWN_SYMBOL: isize = -6;
const INVALID_MODE: isize = -7;
const INVALID_FAVORITES_JSON: isize = -8;
const INVALID_QUERY: isize = -1;
const INVALID_UTF8: isize = -2;
const INVALID_OUTPUT: isize = -3;
const INTERNAL_ERROR: isize = -4;
const OUTPUT_TOO_LARGE: isize = -5;
const INVALID_SCALE: isize = -9;
const INVALID_WEIGHT: isize = -10;
const INVALID_ANIMATION: isize = -11;

unsafe fn read_utf8<'a>(input: *const u8, input_len: usize) -> Result<&'a str, isize> {
    if input_len == 0 {
        return Ok("");
    }
    if input.is_null() {
        return Err(INVALID_QUERY);
    }
    // SAFETY: The calling C ABI function requires its caller to provide `input_len` readable bytes.
    let bytes = unsafe { std::slice::from_raw_parts(input, input_len) };
    std::str::from_utf8(bytes).map_err(|_| INVALID_UTF8)
}

unsafe fn write_bytes(output: *mut u8, output_capacity: usize, bytes: &[u8]) -> isize {
    if output_capacity > 0 && output.is_null() {
        return INVALID_OUTPUT;
    }
    let required = match isize::try_from(bytes.len()) {
        Ok(required) => required,
        Err(_) => return OUTPUT_TOO_LARGE,
    };
    if output_capacity == 0 || output_capacity < bytes.len() {
        return required;
    }
    // SAFETY: The calling C ABI function requires `output_capacity` writable bytes,
    // and this branch proves the copy fits.
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), output, bytes.len()) };
    required
}

/// Search the SF Symbols catalog and write a JSON array of Apple symbol names.
///
/// The return value is the number of JSON bytes required or written. Call once
/// with `output = NULL` and `output_capacity = 0` to size the buffer, then call
/// again with that many writable bytes. A negative return value indicates an
/// error: -1 invalid query pointer, -2 invalid UTF-8, -3 invalid output pointer,
/// -4 internal panic/serialization failure, -5 result length exceeds `isize`.
///
/// # Safety
/// For nonzero `query_len`, `query` must point to that many readable bytes. If
/// `output_capacity` is nonzero, `output` must point to that many writable
/// bytes. The buffers may overlap; the function finishes reading the query
/// before writing output.
#[no_mangle]
pub unsafe extern "C" fn sfstudio_search_json(
    query: *const u8,
    query_len: usize,
    limit: usize,
    output: *mut u8,
    output_capacity: usize,
) -> isize {
    let query_text = match unsafe { read_utf8(query, query_len) } {
        Ok(text) => text,
        Err(code) => return code,
    };
    if output_capacity > 0 && output.is_null() {
        return INVALID_OUTPUT;
    }

    match catch_unwind(AssertUnwindSafe(|| {
        let names: Vec<_> = sfstudio_core::catalog()
            .search(query_text, limit)
            .iter()
            .map(|symbol| symbol.apple_name.as_str())
            .collect();
        serde_json::to_vec(&names).map_err(|_| INTERNAL_ERROR)
    })) {
        Ok(Ok(json)) => unsafe { write_bytes(output, output_capacity, &json) },
        Ok(Err(code)) => code,
        Err(_) => INTERNAL_ERROR,
    }
}

/// Resolve a catalog name to its stable public metadata as UTF-8 JSON.
///
/// The JSON shape is `SfSymbolMetadata`: `appleName`, `pascalName`, `categories`,
/// and `isRestricted`. The return value and output-buffer contract match
/// `sfstudio_search_json`; -6 indicates an unknown symbol.
///
/// # Safety
/// For nonzero `symbol_name_len`, `symbol_name` must point to that many readable
/// bytes. If `output_capacity` is nonzero, `output` must point to that many
/// writable bytes. The buffers may overlap because output is built before it is
/// written.
#[no_mangle]
pub unsafe extern "C" fn sfstudio_symbol_json(
    symbol_name: *const u8,
    symbol_name_len: usize,
    output: *mut u8,
    output_capacity: usize,
) -> isize {
    let symbol_name = match unsafe { read_utf8(symbol_name, symbol_name_len) } {
        Ok(name) => name,
        Err(code) => return code,
    };
    if output_capacity > 0 && output.is_null() {
        return INVALID_OUTPUT;
    }

    match catch_unwind(AssertUnwindSafe(|| {
        let symbol = sfstudio_core::catalog()
            .find(symbol_name)
            .ok_or(UNKNOWN_SYMBOL)?;
        serde_json::to_vec(&serde_json::json!({
            "appleName": symbol.apple_name,
            "pascalName": symbol.pascal_name,
            "categories": symbol.categories,
            "isRestricted": symbol.is_restricted,
        }))
        .map_err(|_| INTERNAL_ERROR)
    })) {
        Ok(Ok(json)) => unsafe { write_bytes(output, output_capacity, &json) },
        Ok(Err(code)) => code,
        Err(_) => INTERNAL_ERROR,
    }
}

fn layer_json(
    node: &sfstudio_core::vector::LayerNode,
    tags: &[String],
    colors: &[String],
) -> serde_json::Value {
    let children: Vec<_> = node
        .children
        .iter()
        .map(|child| layer_json(child, tags, colors))
        .collect();
    let names: Vec<_> = node.tags.iter().filter_map(|tag| tags.get(*tag)).collect();
    let color = node.color_id.and_then(|id| colors.get(id));
    serde_json::json!({
        "isGroup": node.is_group,
        "canPulse": node.can_pulse,
        "canRotate": node.can_rotate,
        "clearBehind": node.clear_behind,
        "animateTogether": node.animate_together,
        "variableDraw": node.variable_draw,
        "autoVariable": node.auto_variable,
        "monoErase": node.mono_erase,
        "shapeErase": node.shape_erase,
        "hierarchical": node.hierarchical,
        "color": color,
        "opacity": node.opacity,
        "rootU": node.ties.first(),
        "rootV": node.ties.get(1),
        "ties": node.ties,
        "contours": node.contours,
        "tags": names,
        "children": children,
    })
}

/// Decode one symbol's weighted vector masters, layers, and optical-size outline.
///
/// `scale` is 0 for medium, 1 for small, and 2 for large. `weight` is the
/// variable font's `wght` axis value. The JSON includes raw masters, points
/// interpolated at that weight in screen coordinates (y points down), and tight
/// visible bounds. -6 means unknown symbol, -9 means an invalid scale, and -10
/// means a non-finite weight. Other errors use the common C ABI codes.
///
/// # Safety
/// For nonzero `symbol_name_len`, `symbol_name` must point to that many readable
/// UTF-8 bytes. If `output_capacity` is nonzero, `output` must point to that many
/// writable bytes. The buffers may overlap because the complete result is built
/// before writing output.
#[no_mangle]
pub unsafe extern "C" fn sfstudio_glyph_json(
    symbol_name: *const u8,
    symbol_name_len: usize,
    scale: i32,
    weight: f32,
    output: *mut u8,
    output_capacity: usize,
) -> isize {
    let symbol_name = match unsafe { read_utf8(symbol_name, symbol_name_len) } {
        Ok(name) => name,
        Err(code) => return code,
    };
    if output_capacity > 0 && output.is_null() {
        return INVALID_OUTPUT;
    }
    if !weight.is_finite() {
        return INVALID_WEIGHT;
    }
    let scale = match scale {
        0 => sfstudio_core::vector::OpticalScale::Medium,
        1 => sfstudio_core::vector::OpticalScale::Small,
        2 => sfstudio_core::vector::OpticalScale::Large,
        _ => return INVALID_SCALE,
    };

    match catch_unwind(AssertUnwindSafe(|| {
        let catalog = sfstudio_core::vector::vector_catalog().map_err(|_| INTERNAL_ERROR)?;
        let symbol = catalog.find(symbol_name).ok_or(UNKNOWN_SYMBOL)?;
        let glyph = catalog
            .glyph_for(symbol)
            .map_err(|_| INTERNAL_ERROR)?
            .ok_or(UNKNOWN_SYMBOL)?;
        let geometry = catalog
            .geometry_at_scale(symbol, &glyph, scale)
            .map_err(|_| INTERNAL_ERROR)?;
        let bounds = sfstudio_core::vector::visible_bounds(&glyph, &geometry, weight)
            .map_err(|_| INTERNAL_ERROR)?;
        let scale_name = match scale {
            sfstudio_core::vector::OpticalScale::Medium => "medium",
            sfstudio_core::vector::OpticalScale::Small => "small",
            sfstudio_core::vector::OpticalScale::Large => "large",
        };
        let used_optical_variant = match scale {
            sfstudio_core::vector::OpticalScale::Medium => false,
            sfstudio_core::vector::OpticalScale::Small => symbol.small.is_some(),
            sfstudio_core::vector::OpticalScale::Large => symbol.large.is_some(),
        };
        let tree: Vec<_> = glyph
            .tree
            .iter()
            .map(|node| layer_json(node, catalog.tags(), catalog.colors()))
            .collect();
        serde_json::to_vec(&serde_json::json!({
            "name": symbol.name,
            "scale": scale_name,
            "weight": weight,
            "usedOpticalVariant": used_optical_variant,
            "bounds": {
                "left": bounds.left,
                "top": bounds.top,
                "right": bounds.right,
                "bottom": bounds.bottom,
            },
            "geometry": {
                "ops": geometry.ops,
                "points": sfstudio_core::vector::interpolate_points(&geometry, weight),
                "regular": geometry.regular,
                "light": geometry.light,
                "heavy": geometry.heavy,
            },
            "layersExact": glyph.layers_exact,
            "backToFront": glyph.back_to_front,
            "gapless": glyph.gapless,
            "defaultVariableRendering": glyph.default_variable_rendering,
            "wiggle": glyph.wiggle,
            "wiggleAngle": glyph.wiggle_angle,
            "rotation": glyph.rotation,
            "anchors": glyph.anchors,
            "tree": tree,
        }))
        .map_err(|_| INTERNAL_ERROR)
    })) {
        Ok(Ok(json)) => unsafe { write_bytes(output, output_capacity, &json) },
        Ok(Err(code)) => code,
        Err(_) => INTERNAL_ERROR,
    }
}

/// Search the catalog while applying an optional category and favorites filter.
///
/// An empty category disables category filtering. `favorites_only` uses zero
/// for false and nonzero for true; when true, `favorites_json` must be a JSON
/// array of strings. The return value and buffer protocol match
/// `sfstudio_search_json`; -8 indicates invalid favorites JSON.
///
/// # Safety
/// Each nonzero input length must describe readable bytes. If `output_capacity`
/// is nonzero, `output` must point to that many writable bytes. The input and
/// output buffers may overlap because the result is created before writing.
#[no_mangle]
pub unsafe extern "C" fn sfstudio_search_filtered_json(
    query: *const u8,
    query_len: usize,
    category: *const u8,
    category_len: usize,
    favorites_json: *const u8,
    favorites_json_len: usize,
    favorites_only: i32,
    limit: usize,
    output: *mut u8,
    output_capacity: usize,
) -> isize {
    let query = match unsafe { read_utf8(query, query_len) } {
        Ok(query) => query,
        Err(code) => return code,
    };
    let category = match unsafe { read_utf8(category, category_len) } {
        Ok(category) => category,
        Err(code) => return code,
    };
    let favorites_json = match unsafe { read_utf8(favorites_json, favorites_json_len) } {
        Ok(json) => json,
        Err(code) => return code,
    };
    if output_capacity > 0 && output.is_null() {
        return INVALID_OUTPUT;
    }

    match catch_unwind(AssertUnwindSafe(|| {
        let favorites = if favorites_only != 0 {
            let parsed = sfstudio_core::favorites::Favorites::from_json(favorites_json)
                .map_err(|_| INVALID_FAVORITES_JSON)?;
            Some(parsed.as_slice().iter().cloned().collect::<HashSet<_>>())
        } else {
            None
        };
        let names: Vec<_> = sfstudio_core::catalog()
            .search_filtered(
                query,
                (!category.is_empty()).then_some(category),
                favorites.as_ref(),
                limit,
            )
            .iter()
            .map(|symbol| symbol.apple_name.as_str())
            .collect();
        serde_json::to_vec(&names).map_err(|_| INTERNAL_ERROR)
    })) {
        Ok(Ok(json)) => unsafe { write_bytes(output, output_capacity, &json) },
        Ok(Err(code)) => code,
        Err(_) => INTERNAL_ERROR,
    }
}

/// Generate the Kotlin Compose snippet for a catalog symbol.
///
/// `mode` is 0 for dualtone and 1 for monochrome. `tint_enabled` and
/// `background_enabled` use zero for false and any nonzero value for true.
/// The return value is the required/written UTF-8 byte count, or a negative
/// error: -1 invalid symbol pointer, -2 invalid UTF-8, -3 invalid output pointer,
/// -4 internal panic, -5 result too large, -6 unknown symbol, -7 invalid mode.
/// Use `output = NULL` and `output_capacity = 0` to query the required size.
///
/// # Safety
/// For nonzero `symbol_len`, `symbol` must point to that many readable bytes. If
/// `output_capacity` is nonzero, `output` must point to that many writable
/// bytes. The buffers may overlap because the function creates the full result
/// before writing it.
#[no_mangle]
pub unsafe extern "C" fn sfstudio_compose_snippet(
    symbol_name: *const u8,
    symbol_len: usize,
    mode: i32,
    tint_argb: i32,
    tint_enabled: i32,
    background_argb: i32,
    background_enabled: i32,
    output: *mut u8,
    output_capacity: usize,
) -> isize {
    let symbol_name = match unsafe { read_utf8(symbol_name, symbol_len) } {
        Ok(name) => name,
        Err(code) => return code,
    };
    if output_capacity > 0 && output.is_null() {
        return INVALID_OUTPUT;
    }
    let mode = match mode {
        0 => sfstudio_core::codegen::SymbolMode::Dualtone,
        1 => sfstudio_core::codegen::SymbolMode::Monochrome,
        _ => return INVALID_MODE,
    };

    match catch_unwind(AssertUnwindSafe(|| {
        let symbol = sfstudio_core::catalog()
            .find(symbol_name)
            .ok_or(UNKNOWN_SYMBOL)?;
        Ok(sfstudio_core::codegen::compose_snippet(
            symbol,
            mode,
            tint_argb,
            tint_enabled != 0,
            (background_enabled != 0).then_some(background_argb),
        )
        .into_bytes())
    })) {
        Ok(Ok(snippet)) => unsafe { write_bytes(output, output_capacity, &snippet) },
        Ok(Err(code)) => code,
        Err(_) => INTERNAL_ERROR,
    }
}

/// Toggle a favorite in a JSON array while preserving its order.
///
/// The return value is the required/written UTF-8 byte count, or a negative
/// error: -1 invalid input pointer, -2 invalid UTF-8, -3 invalid output pointer,
/// -4 internal panic/serialization failure, -5 result too large, -8 invalid
/// favorites JSON. Use `output = NULL` and `output_capacity = 0` to query size.
///
/// # Safety
/// Each nonzero input length must describe readable bytes. If `output_capacity`
/// is nonzero, `output` must point to that many writable bytes. The buffers may
/// overlap because the function owns its inputs' result before writing output.
#[no_mangle]
pub unsafe extern "C" fn sfstudio_favorites_toggle_json(
    favorites_json: *const u8,
    favorites_json_len: usize,
    symbol_name: *const u8,
    symbol_name_len: usize,
    output: *mut u8,
    output_capacity: usize,
) -> isize {
    let favorites_json = match unsafe { read_utf8(favorites_json, favorites_json_len) } {
        Ok(json) => json,
        Err(code) => return code,
    };
    let symbol_name = match unsafe { read_utf8(symbol_name, symbol_name_len) } {
        Ok(name) => name,
        Err(code) => return code,
    };
    if output_capacity > 0 && output.is_null() {
        return INVALID_OUTPUT;
    }

    match catch_unwind(AssertUnwindSafe(|| {
        let mut favorites = sfstudio_core::favorites::Favorites::from_json(favorites_json)
            .map_err(|_| INVALID_FAVORITES_JSON)?;
        favorites.toggle(symbol_name);
        favorites
            .to_json()
            .map(String::into_bytes)
            .map_err(|_| INTERNAL_ERROR)
    })) {
        Ok(Ok(json)) => unsafe { write_bytes(output, output_capacity, &json) },
        Ok(Err(code)) => code,
        Err(_) => INTERNAL_ERROR,
    }
}

/// Evaluate an Apple-style symbol effect and optional replace transition as UTF-8 JSON.
///
/// Input is an object with `symbol`, finite nonnegative `time`, optional `rtl`, `effect`,
/// optional `variableValue`, optional `transition`, and optional `magicReplace`.
/// Effect names use snake case (for example `variable_color` and `draw_on`);
/// layer results include Apple effect transforms and independent variable
/// rendering alpha/draw. `magicReplace` returns matched layer outlines.
/// Returns the required/written byte count, or -11 for invalid animation JSON
/// or options. Other errors use the common buffer contract; -6 means unknown
/// symbol. Use `output = NULL` and `output_capacity = 0` to query the size.
///
/// # Safety
/// For nonzero `input_len`, `input` must point to that many readable UTF-8
/// bytes. If `output_capacity` is nonzero, `output` must point to that many
/// writable bytes. The buffers may overlap because the result is built first.
#[no_mangle]
pub unsafe extern "C" fn sfstudio_animation_json(
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_capacity: usize,
) -> isize {
    let input = match unsafe { read_utf8(input, input_len) } {
        Ok(input) => input,
        Err(code) => return code,
    };
    if output_capacity > 0 && output.is_null() {
        return INVALID_OUTPUT;
    }
    let request: serde_json::Value = match serde_json::from_str(input) {
        Ok(request) => request,
        Err(_) => return INVALID_ANIMATION,
    };
    let Some(object) = request.as_object() else {
        return INVALID_ANIMATION;
    };
    let Some(name) = object.get("symbol").and_then(serde_json::Value::as_str) else {
        return INVALID_ANIMATION;
    };
    let Some(time) = object
        .get("time")
        .and_then(serde_json::Value::as_f64)
        .map(|v| v as f32)
        .filter(|v| v.is_finite() && *v >= 0.0)
    else {
        return INVALID_ANIMATION;
    };
    let rtl = match object.get("rtl") {
        None => false,
        Some(value) => match value.as_bool() {
            Some(value) => value,
            None => return INVALID_ANIMATION,
        },
    };
    let Some(effect_value) = object.get("effect") else {
        return INVALID_ANIMATION;
    };
    let effect = match parse_effect(effect_value) {
        Some(effect) => effect,
        None => return INVALID_ANIMATION,
    };
    let variable_value = match object.get("variableValue") {
        None => None,
        Some(value) => match value.as_f64().map(|v| v as f32).filter(|v| v.is_finite()) {
            Some(value) => Some(value),
            None => return INVALID_ANIMATION,
        },
    };
    let transition = match object.get("transition") {
        None | Some(serde_json::Value::Null) => None,
        Some(value) => match parse_transition(value) {
            Some(value) => Some(value),
            None => return INVALID_ANIMATION,
        },
    };
    let magic_request = match object.get("magicReplace") {
        None | Some(serde_json::Value::Null) => None,
        Some(value) => match parse_magic_replace(value) {
            Some(value) => Some(value),
            None => return INVALID_ANIMATION,
        },
    };

    match catch_unwind(AssertUnwindSafe(|| {
        let catalog = sfstudio_core::vector::vector_catalog().map_err(|_| INTERNAL_ERROR)?;
        let symbol = catalog.find(name).ok_or(UNKNOWN_SYMBOL)?;
        let glyph = catalog
            .glyph_for(symbol)
            .map_err(|_| INTERNAL_ERROR)?
            .ok_or(UNKNOWN_SYMBOL)?;
        let geometry = catalog
            .geometry_at_scale(symbol, &glyph, sfstudio_core::vector::OpticalScale::Medium)
            .map_err(|_| INTERNAL_ERROR)?;
        let animation = sfstudio_core::animation::AnimationSymbol::from_glyph(&glyph, &geometry);
        let (variable_alpha, variable_draw) = variable_value
            .map(|value| animation.variable_levels(value))
            .unwrap_or_else(|| {
                (
                    vec![1.0; animation.layers.len()],
                    vec![1.0; animation.layers.len()],
                )
            });
        let layers: Vec<_> = animation
            .layers
            .iter()
            .enumerate()
            .map(|(index, _)| {
                let frame = animation
                    .apply(effect, time, index, rtl)
                    .unwrap_or_default();
                serde_json::json!({
                    "scale": frame.scale,
                    "dx": frame.dx,
                    "dy": frame.dy,
                    "rotation": frame.rotation,
                    "alpha": frame.alpha,
                    "draw": frame.draw,
                    "pivotX": frame.pivot_x.is_finite().then_some(frame.pivot_x),
                    "pivotY": frame.pivot_y.is_finite().then_some(frame.pivot_y),
                    "variableAlpha": variable_alpha[index],
                    "variableDraw": variable_draw[index],
                })
            })
            .collect();
        let transition = transition.map(|(direction, t)| {
            let out = sfstudio_core::animation::replace_out(t, direction);
            let incoming = sfstudio_core::animation::replace_in(t, direction);
            serde_json::json!({
                "duration": sfstudio_core::animation::replace_seconds(direction),
                "out": { "scale": out.scale, "alpha": out.alpha },
                "in": { "scale": incoming.scale, "alpha": incoming.alpha },
            })
        });
        let magic_replace =
            if let Some((from, to, progress, from_weight, to_weight, from_scale, to_scale)) =
                &magic_request
            {
                let old_info = catalog.find(from).ok_or(UNKNOWN_SYMBOL)?;
                let new_info = catalog.find(to).ok_or(UNKNOWN_SYMBOL)?;
                let old_glyph = catalog
                    .glyph_for(old_info)
                    .map_err(|_| INTERNAL_ERROR)?
                    .ok_or(UNKNOWN_SYMBOL)?;
                let new_glyph = catalog
                    .glyph_for(new_info)
                    .map_err(|_| INTERNAL_ERROR)?
                    .ok_or(UNKNOWN_SYMBOL)?;
                let medium = sfstudio_core::vector::OpticalScale::Medium;
                let old_medium = catalog
                    .geometry_at_scale(old_info, &old_glyph, medium)
                    .map_err(|_| INTERNAL_ERROR)?;
                let new_medium = catalog
                    .geometry_at_scale(new_info, &new_glyph, medium)
                    .map_err(|_| INTERNAL_ERROR)?;
                let old_geometry = catalog
                    .geometry_at_scale(old_info, &old_glyph, *from_scale)
                    .map_err(|_| INTERNAL_ERROR)?;
                let new_geometry = catalog
                    .geometry_at_scale(new_info, &new_glyph, *to_scale)
                    .map_err(|_| INTERNAL_ERROR)?;
                let old_bounds =
                    sfstudio_core::vector::visible_bounds(&old_glyph, &old_geometry, *from_weight)
                        .map_err(|_| INTERNAL_ERROR)?;
                let new_bounds =
                    sfstudio_core::vector::visible_bounds(&new_glyph, &new_geometry, *to_weight)
                        .map_err(|_| INTERNAL_ERROR)?;
                let old_fit =
                    sfstudio_core::vector::visible_bounds(&old_glyph, &old_medium, *from_weight)
                        .map_err(|_| INTERNAL_ERROR)?;
                let new_fit =
                    sfstudio_core::vector::visible_bounds(&new_glyph, &new_medium, *to_weight)
                        .map_err(|_| INTERNAL_ERROR)?;
                let old_model =
                    sfstudio_core::animation::AnimationSymbol::from_glyph(&old_glyph, &old_medium);
                let new_model =
                    sfstudio_core::animation::AnimationSymbol::from_glyph(&new_glyph, &new_medium);
                let old_side = sfstudio_core::animation::MagicReplaceSide {
                    symbol: &old_model,
                    geometry: &old_geometry,
                    weight: *from_weight,
                    bounds: old_bounds,
                    fit: old_fit,
                };
                let new_side = sfstudio_core::animation::MagicReplaceSide {
                    symbol: &new_model,
                    geometry: &new_geometry,
                    weight: *to_weight,
                    bounds: new_bounds,
                    fit: new_fit,
                };
                let matched =
                    sfstudio_core::animation::magic_replace(old_side, new_side, *progress)
                        .map_err(|_| INTERNAL_ERROR)?;
                Some(serde_json::json!({
                    "from": old_info.name,
                    "to": new_info.name,
                    "progress": progress,
                    "matchedLayers": matched.iter().map(|m| serde_json::json!({
                        "oldLayer": m.old_layer,
                        "newLayer": m.new_layer,
                        "ops": m.ops,
                        "points": m.points,
                    })).collect::<Vec<_>>(),
                }))
            } else {
                None
            };
        serde_json::to_vec(&serde_json::json!({
            "symbol": symbol.name,
            "time": time,
            "duration": animation.duration(effect),
            "held": sfstudio_core::animation::AnimationSymbol::is_held(effect),
            "defaultRepeat": if sfstudio_core::animation::AnimationSymbol::continuous_by_default(effect) { "continuous" } else { "once" },
            "layers": layers,
            "transition": transition,
            "magicReplace": magic_replace,
        })).map_err(|_| INTERNAL_ERROR)
    })) {
        Ok(Ok(json)) => unsafe { write_bytes(output, output_capacity, &json) },
        Ok(Err(code)) => code,
        Err(_) => INTERNAL_ERROR,
    }
}

fn parse_scope(
    object: &serde_json::Map<String, serde_json::Value>,
) -> Option<sfstudio_core::animation::Scope> {
    use sfstudio_core::animation::Scope;
    match object
        .get("scope")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("whole")
    {
        "whole" | "whole_symbol" | "wholeSymbol" => Some(Scope::WholeSymbol),
        "by_layer" | "byLayer" => Some(Scope::ByLayer),
        _ => None,
    }
}

fn parse_effect(value: &serde_json::Value) -> Option<sfstudio_core::animation::Effect> {
    use sfstudio_core::animation::{Effect, WiggleDirection as W};
    let object = value.as_object()?;
    let kind = object.get("type")?.as_str()?;
    let scope = || parse_scope(object);
    let boolean = |key: &str, default: bool| {
        object
            .get(key)
            .map_or(Some(default), serde_json::Value::as_bool)
    };
    Some(match kind {
        "appear" => Effect::Appear { scope: scope()? },
        "disappear" => Effect::Disappear { scope: scope()? },
        "bounce" => Effect::Bounce {
            up: match object
                .get("direction")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("up")
            {
                "up" => true,
                "down" => false,
                _ => return None,
            },
            scope: scope()?,
        },
        "pulse" => Effect::Pulse { scope: scope()? },
        "variable_color" => Effect::VariableColor {
            cumulative: match object
                .get("style")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("iterative")
            {
                "iterative" => false,
                "cumulative" => true,
                _ => return None,
            },
            reversing: boolean("reversing", false)?,
            hide_inactive: boolean("hideInactiveLayers", false)?,
        },
        "scale" => Effect::Scale {
            up: boolean("up", true)?,
            returns: boolean("returns", false)?,
        },
        "wiggle" => {
            let direction = match object.get("direction") {
                None | Some(serde_json::Value::Null) => None,
                Some(serde_json::Value::String(name)) => Some(match name.as_str() {
                    "left" => W::Left,
                    "right" => W::Right,
                    "up" => W::Up,
                    "down" => W::Down,
                    "forward" => W::Forward,
                    "backward" => W::Backward,
                    "clockwise" => W::Clockwise,
                    "counter_clockwise" => W::CounterClockwise,
                    _ => return None,
                }),
                Some(value) => {
                    let degrees = value.as_object()?.get("custom")?.as_f64()? as f32;
                    if !degrees.is_finite() {
                        return None;
                    }
                    Some(W::Custom(degrees))
                }
            };
            Effect::Wiggle {
                direction,
                continuous: boolean("continuous", false)?,
            }
        }
        "rotate" => {
            let clockwise = match object.get("direction") {
                None | Some(serde_json::Value::Null) => None,
                Some(serde_json::Value::String(value)) => match value.as_str() {
                    "clockwise" => Some(true),
                    "counter_clockwise" => Some(false),
                    _ => return None,
                },
                _ => return None,
            };
            Effect::Rotate {
                clockwise,
                scope: scope()?,
                continuous: boolean("continuous", false)?,
            }
        }
        "breathe" => Effect::Breathe {
            pulsing: boolean("pulsing", false)?,
        },
        "draw_on" => Effect::DrawOn { scope: scope()? },
        "draw_off" => Effect::DrawOff { scope: scope()? },
        _ => return None,
    })
}

fn parse_transition(
    value: &serde_json::Value,
) -> Option<(sfstudio_core::animation::ReplaceDirection, f32)> {
    use sfstudio_core::animation::ReplaceDirection;
    let object = value.as_object()?;
    let direction = match object.get("direction")?.as_str()? {
        "down_up" => ReplaceDirection::DownUp,
        "off_up" => ReplaceDirection::OffUp,
        "up_up" => ReplaceDirection::UpUp,
        _ => return None,
    };
    let time = object.get("time")?.as_f64()? as f32;
    (time.is_finite() && time >= 0.0).then_some((direction, time))
}

type MagicReplaceRequest = (
    String,
    String,
    f32,
    f32,
    f32,
    sfstudio_core::vector::OpticalScale,
    sfstudio_core::vector::OpticalScale,
);

fn parse_magic_replace(value: &serde_json::Value) -> Option<MagicReplaceRequest> {
    let object = value.as_object()?;
    let from = object.get("from")?.as_str()?.to_owned();
    let to = object.get("to")?.as_str()?.to_owned();
    let progress = object.get("progress")?.as_f64()? as f32;
    if !progress.is_finite() || !(0.0..=1.0).contains(&progress) {
        return None;
    }
    let weight = |key: &str| -> Option<f32> {
        let value = object
            .get(key)
            .map_or(Some(400.0), serde_json::Value::as_f64)? as f32;
        value.is_finite().then_some(value)
    };
    let scale = |key: &str| -> Option<sfstudio_core::vector::OpticalScale> {
        use sfstudio_core::vector::OpticalScale;
        match object
            .get(key)
            .and_then(serde_json::Value::as_str)
            .unwrap_or("medium")
        {
            "small" => Some(OpticalScale::Small),
            "medium" => Some(OpticalScale::Medium),
            "large" => Some(OpticalScale::Large),
            _ => None,
        }
    };
    Some((
        from,
        to,
        progress,
        weight("fromWeight")?,
        weight("toWeight")?,
        scale("fromScale")?,
        scale("toScale")?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn glyph_json(name: &[u8], scale: i32, weight: f32) -> Result<serde_json::Value, isize> {
        // SAFETY: Input remains alive for both calls; output has the queried size.
        unsafe {
            let required = sfstudio_glyph_json(
                name.as_ptr(),
                name.len(),
                scale,
                weight,
                std::ptr::null_mut(),
                0,
            );
            if required < 0 {
                return Err(required);
            }
            let mut output = vec![0; required as usize];
            let written = sfstudio_glyph_json(
                name.as_ptr(),
                name.len(),
                scale,
                weight,
                output.as_mut_ptr(),
                output.len(),
            );
            if written != required {
                return Err(written);
            }
            serde_json::from_slice(&output).map_err(|_| INTERNAL_ERROR)
        }
    }

    fn search(query: &[u8], limit: usize) -> Result<String, isize> {
        // SAFETY: Both slices remain alive for the call and output capacity is exact.
        unsafe {
            let required =
                sfstudio_search_json(query.as_ptr(), query.len(), limit, std::ptr::null_mut(), 0);
            if required < 0 {
                return Err(required);
            }
            let mut output = vec![0; required as usize];
            let written = sfstudio_search_json(
                query.as_ptr(),
                query.len(),
                limit,
                output.as_mut_ptr(),
                output.len(),
            );
            if written != required {
                return Err(written);
            }
            Ok(String::from_utf8(output).expect("JSON is UTF-8"))
        }
    }

    #[test]
    fn returns_ranked_names_and_supports_sizing_calls() {
        assert_eq!(search(b"hosue", 1).unwrap(), r#"["house"]"#);
        assert_eq!(
            search(b"", 3).unwrap(),
            r#"["0.circle.fill","0.circle","0.square.fill"]"#
        );
        assert_eq!(search("🐈".as_bytes(), 5).unwrap(), "[]");
    }

    #[test]
    fn symbol_lookup_returns_kotlin_public_metadata() {
        let name = b"SFHeartFill";
        let mut output = [0; 256];
        // SAFETY: Input and output slices are valid for their lengths.
        let written = unsafe {
            sfstudio_symbol_json(name.as_ptr(), name.len(), output.as_mut_ptr(), output.len())
        };
        assert!(written > 0);
        let value: serde_json::Value = serde_json::from_slice(&output[..written as usize]).unwrap();
        assert_eq!(value["appleName"], "heart.fill");
        assert_eq!(value["pascalName"], "SFHeartFill");
        assert_eq!(value["isRestricted"], false);
        assert!(value["categories"]
            .as_array()
            .is_some_and(|values| !values.is_empty()));
        // SAFETY: Input is valid; output is a size query.
        assert_eq!(
            unsafe { sfstudio_symbol_json(b"unknown".as_ptr(), 7, std::ptr::null_mut(), 0) },
            UNKNOWN_SYMBOL
        );
    }

    #[test]
    fn glyph_abi_returns_weight_masters_layers_and_optical_scales() {
        let medium = glyph_json(b"SFHeartFill", 0, 400.0).unwrap();
        assert_eq!(medium["name"], "heart.fill");
        assert_eq!(medium["scale"], "medium");
        assert_eq!(medium["usedOpticalVariant"], false);
        assert!(!medium["geometry"]["ops"].as_array().unwrap().is_empty());
        assert_eq!(
            medium["geometry"]["regular"].as_array().unwrap().len(),
            medium["geometry"]["heavy"].as_array().unwrap().len()
        );
        assert_eq!(medium["weight"], 400.0);
        assert!(
            medium["bounds"]["left"].as_f64().unwrap()
                < medium["bounds"]["right"].as_f64().unwrap()
        );
        assert!(
            medium["bounds"]["top"].as_f64().unwrap()
                < medium["bounds"]["bottom"].as_f64().unwrap()
        );
        assert_eq!(
            medium["geometry"]["points"].as_array().unwrap().len(),
            medium["geometry"]["regular"].as_array().unwrap().len()
        );
        assert!(!medium["tree"].as_array().unwrap().is_empty());

        let black = glyph_json(b"heart.fill", 0, 1000.0).unwrap();
        let points = black["geometry"]["points"].as_array().unwrap();
        let heavy = black["geometry"]["heavy"].as_array().unwrap();
        for (index, (point, master)) in points.iter().zip(heavy).enumerate() {
            let master = master.as_f64().unwrap();
            let expected = if index % 2 == 1 { -master } else { master };
            assert!((point.as_f64().unwrap() - expected).abs() < 1e-3);
        }

        let small = glyph_json(b"heart.fill", 1, 400.0).unwrap();
        let large = glyph_json(b"heart.fill", 2, 400.0).unwrap();
        assert_eq!(small["usedOpticalVariant"], true);
        assert_eq!(large["usedOpticalVariant"], true);
        assert_ne!(small["geometry"]["regular"], medium["geometry"]["regular"]);
        assert_ne!(large["geometry"]["regular"], medium["geometry"]["regular"]);
        assert_eq!(glyph_json(b"missing", 0, 400.0), Err(UNKNOWN_SYMBOL));
        assert_eq!(glyph_json(b"heart.fill", 3, 400.0), Err(INVALID_SCALE));
        assert_eq!(glyph_json(b"heart.fill", 0, f32::NAN), Err(INVALID_WEIGHT));
    }

    #[test]
    fn filtered_search_returns_only_matching_favorites_and_category() {
        let symbol = sfstudio_core::catalog().find("heart.fill").unwrap();
        let category = symbol.categories.first().unwrap();
        let favorites = br#"["heart.fill","star"]"#;
        let expected: Vec<_> = sfstudio_core::catalog()
            .search_filtered(
                "",
                Some(category),
                Some(&HashSet::from(["heart.fill".to_owned(), "star".to_owned()])),
                10,
            )
            .iter()
            .map(|symbol| symbol.apple_name.clone())
            .collect();
        let mut output = [0; 64];
        // SAFETY: Input slices are valid and output has enough capacity.
        let written = unsafe {
            sfstudio_search_filtered_json(
                std::ptr::null(),
                0,
                category.as_bytes().as_ptr(),
                category.len(),
                favorites.as_ptr(),
                favorites.len(),
                1,
                10,
                output.as_mut_ptr(),
                output.len(),
            )
        };
        assert!(written > 0);
        let actual: Vec<String> = serde_json::from_slice(&output[..written as usize]).unwrap();
        assert_eq!(actual, expected);
        // SAFETY: Inputs are valid; output is a size query.
        assert_eq!(
            unsafe {
                sfstudio_search_filtered_json(
                    std::ptr::null(),
                    0,
                    std::ptr::null(),
                    0,
                    b"{broken".as_ptr(),
                    7,
                    1,
                    10,
                    std::ptr::null_mut(),
                    0,
                )
            },
            INVALID_FAVORITES_JSON
        );
    }

    #[test]
    fn reports_bad_inputs_and_never_partially_writes_short_buffers() {
        // SAFETY: The non-null test buffers are valid for their declared lengths.
        unsafe {
            assert_eq!(
                sfstudio_search_json(std::ptr::null(), 1, 1, std::ptr::null_mut(), 0),
                INVALID_QUERY
            );
            assert_eq!(
                sfstudio_search_json([0xff].as_ptr(), 1, 1, std::ptr::null_mut(), 0),
                INVALID_UTF8
            );
            assert_eq!(
                sfstudio_search_json(b"a".as_ptr(), 1, 1, std::ptr::null_mut(), 1),
                INVALID_OUTPUT
            );
            let mut short = [0xa5; 1];
            let required = sfstudio_search_json(b"home".as_ptr(), 4, 1, short.as_mut_ptr(), 1);
            assert!(required > 1);
            assert_eq!(short, [0xa5]);
        }
    }

    fn compose_snippet(
        mode: i32,
        tint_enabled: bool,
        background: Option<i32>,
    ) -> Result<String, isize> {
        let name = b"heart.fill";
        // SAFETY: Input and output slices remain alive and satisfy their lengths.
        unsafe {
            let background_argb = background.unwrap_or_default();
            let background_enabled = i32::from(background.is_some());
            let required = sfstudio_compose_snippet(
                name.as_ptr(),
                name.len(),
                mode,
                0x80e0_245e_u32 as i32,
                i32::from(tint_enabled),
                background_argb,
                background_enabled,
                std::ptr::null_mut(),
                0,
            );
            if required < 0 {
                return Err(required);
            }
            let mut output = vec![0; required as usize];
            let written = sfstudio_compose_snippet(
                name.as_ptr(),
                name.len(),
                mode,
                0x80e0_245e_u32 as i32,
                i32::from(tint_enabled),
                background_argb,
                background_enabled,
                output.as_mut_ptr(),
                output.len(),
            );
            if written != required {
                return Err(written);
            }
            Ok(String::from_utf8(output).expect("snippet is UTF-8"))
        }
    }

    #[test]
    fn returns_the_existing_plain_and_background_snippets() {
        assert_eq!(
            compose_snippet(1, true, Some(0xffdc_e9d3_u32 as i32)).unwrap(),
            include_str!(
                "../../../app/src/jvmTest/resources/compose-snippet-monochrome-background.kt.txt"
            )
            .replace("\r\n", "\n")
        );
        assert_eq!(
            compose_snippet(0, false, None).unwrap(),
            include_str!(
                "../../../app/src/jvmTest/resources/compose-snippet-dualtone-plain.kt.txt"
            )
            .replace("\r\n", "\n")
        );
    }

    #[test]
    fn reports_unknown_symbols_and_invalid_modes() {
        // SAFETY: The input is a valid static byte slice and output is a size query.
        unsafe {
            assert_eq!(
                sfstudio_compose_snippet(
                    b"missing".as_ptr(),
                    7,
                    0,
                    0,
                    0,
                    0,
                    0,
                    std::ptr::null_mut(),
                    0
                ),
                UNKNOWN_SYMBOL
            );
            assert_eq!(
                sfstudio_compose_snippet(
                    b"heart.fill".as_ptr(),
                    10,
                    2,
                    0,
                    0,
                    0,
                    0,
                    std::ptr::null_mut(),
                    0
                ),
                INVALID_MODE
            );
        }
    }

    #[test]
    fn toggles_favorites_without_changing_order_or_json_contract() {
        let input = br#"["heart.fill","star"]"#;
        let name = b"gear";
        // SAFETY: Inputs and output satisfy their declared lengths.
        unsafe {
            let required = sfstudio_favorites_toggle_json(
                input.as_ptr(),
                input.len(),
                name.as_ptr(),
                name.len(),
                std::ptr::null_mut(),
                0,
            );
            assert!(required > 0);
            let mut output = vec![0; required as usize];
            assert_eq!(
                sfstudio_favorites_toggle_json(
                    input.as_ptr(),
                    input.len(),
                    name.as_ptr(),
                    name.len(),
                    output.as_mut_ptr(),
                    output.len(),
                ),
                required
            );
            assert_eq!(
                String::from_utf8(output).unwrap(),
                "[\n  \"heart.fill\",\n  \"star\",\n  \"gear\"\n]"
            );
        }
    }

    #[test]
    fn rejects_malformed_favorites_json() {
        // SAFETY: Both inputs are valid static byte slices; output is a size query.
        unsafe {
            assert_eq!(
                sfstudio_favorites_toggle_json(
                    b"{broken".as_ptr(),
                    7,
                    b"gear".as_ptr(),
                    4,
                    std::ptr::null_mut(),
                    0,
                ),
                INVALID_FAVORITES_JSON
            );
        }
    }
}
