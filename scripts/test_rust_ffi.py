#!/usr/bin/env python3
"""Smoke-test the Rust C ABI from Python through ctypes."""
import argparse
import ctypes
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]


def default_library():
    if sys.platform == "win32":
        return ROOT / "rust/target/debug/sfstudio_ffi.dll"
    if sys.platform == "darwin":
        return ROOT / "rust/target/debug/libsfstudio_ffi.dylib"
    return ROOT / "rust/target/debug/libsfstudio_ffi.so"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--library", type=Path, default=default_library())
    library_path = parser.parse_args().library
    if not library_path.is_file():
        parser.error(f"Build the Rust FFI library first: {library_path}")

    library = ctypes.CDLL(str(library_path))
    search_json = library.sfstudio_search_json
    search_json.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_size_t,
                            ctypes.c_void_p, ctypes.c_size_t]
    search_json.restype = ctypes.c_ssize_t
    symbol_json = library.sfstudio_symbol_json
    symbol_json.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p, ctypes.c_size_t]
    symbol_json.restype = ctypes.c_ssize_t
    glyph_json = library.sfstudio_glyph_json
    glyph_json.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_int, ctypes.c_float,
                           ctypes.c_void_p, ctypes.c_size_t]
    glyph_json.restype = ctypes.c_ssize_t
    animation_json = library.sfstudio_animation_json
    animation_json.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p, ctypes.c_size_t]
    animation_json.restype = ctypes.c_ssize_t
    draw_field_json = library.sfstudio_draw_field_json
    draw_field_json.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p, ctypes.c_size_t]
    draw_field_json.restype = ctypes.c_ssize_t
    mcp_client_config_json = library.sfstudio_mcp_client_config_json
    mcp_client_config_json.argtypes = [
        ctypes.c_void_p, ctypes.c_size_t, ctypes.c_int32,
        ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p, ctypes.c_size_t,
    ]
    mcp_client_config_json.restype = ctypes.c_ssize_t
    filter_selection_json = library.sfstudio_filter_selection_json
    filter_selection_json.argtypes = [
        ctypes.c_int32, ctypes.c_void_p, ctypes.c_size_t, ctypes.c_int32,
        ctypes.c_void_p, ctypes.c_size_t,
    ]
    filter_selection_json.restype = ctypes.c_ssize_t
    search_filtered_json = library.sfstudio_search_filtered_json
    search_filtered_json.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p,
                                     ctypes.c_size_t, ctypes.c_void_p, ctypes.c_size_t,
                                     ctypes.c_int, ctypes.c_size_t, ctypes.c_void_p,
                                     ctypes.c_size_t]
    search_filtered_json.restype = ctypes.c_ssize_t
    compose_snippet = library.sfstudio_compose_snippet
    compose_snippet.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_int,
                                ctypes.c_int, ctypes.c_int, ctypes.c_int, ctypes.c_int,
                                ctypes.c_void_p, ctypes.c_size_t]
    compose_snippet.restype = ctypes.c_ssize_t
    toggle_favorites = library.sfstudio_favorites_toggle_json
    toggle_favorites.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p,
                                 ctypes.c_size_t, ctypes.c_void_p, ctypes.c_size_t]
    toggle_favorites.restype = ctypes.c_ssize_t

    def search(query, limit):
        encoded = query.encode("utf-8")
        query_buffer = ctypes.create_string_buffer(encoded) if encoded else None
        query_pointer = ctypes.cast(query_buffer, ctypes.c_void_p) if query_buffer else None
        required = search_json(query_pointer, len(encoded), limit, None, 0)
        assert required >= 0, f"sizing call failed: {required}"
        output = ctypes.create_string_buffer(required)
        written = search_json(query_pointer, len(encoded), limit, output, required)
        assert written == required, f"write returned {written}, expected {required}"
        return json.loads(output.raw[:written])

    assert search("hosue", 1) == ["house"]
    assert search("🐈", 5) == []
    assert search("", 3) == ["0.circle.fill", "0.circle", "0.square.fill"]

    symbol_name = ctypes.create_string_buffer(b"SFHeartFill")
    symbol_length = len(b"SFHeartFill")
    symbol_required = symbol_json(symbol_name, symbol_length, None, 0)
    assert symbol_required > 0
    symbol_output = ctypes.create_string_buffer(symbol_required)
    symbol_written = symbol_json(symbol_name, symbol_length, symbol_output, symbol_required)
    assert symbol_written == symbol_required
    metadata = json.loads(symbol_output.raw[:symbol_written])
    assert metadata["appleName"] == "heart.fill"
    assert metadata["pascalName"] == "SFHeartFill"
    assert metadata["isRestricted"] is False
    assert metadata["categories"]
    assert symbol_json(ctypes.create_string_buffer(b"missing"), 7, None, 0) == -6

    def glyph(name, scale, weight=400.0):
        encoded = name.encode("utf-8")
        name_buffer = ctypes.create_string_buffer(encoded)
        required = glyph_json(name_buffer, len(encoded), scale, weight, None, 0)
        assert required > 0, f"glyph sizing call failed: {required}"
        output = ctypes.create_string_buffer(required)
        written = glyph_json(name_buffer, len(encoded), scale, weight, output, required)
        assert written == required
        return json.loads(output.raw[:written])

    medium_glyph = glyph("SFHeartFill", 0)
    black_glyph = glyph("heart.fill", 0, 1000.0)
    small_glyph = glyph("heart.fill", 1)
    large_glyph = glyph("heart.fill", 2)
    assert medium_glyph["name"] == "heart.fill"
    assert medium_glyph["scale"] == "medium"
    assert medium_glyph["weight"] == 400.0
    assert medium_glyph["geometry"]["ops"]
    assert len(medium_glyph["geometry"]["points"]) == len(medium_glyph["geometry"]["regular"])
    assert medium_glyph["bounds"]["left"] < medium_glyph["bounds"]["right"]
    assert medium_glyph["bounds"]["top"] < medium_glyph["bounds"]["bottom"]
    assert black_glyph["weight"] == 1000.0
    assert all(
        abs(point - master * (-1 if index % 2 else 1)) < 1e-3
        for index, (point, master) in enumerate(zip(
            black_glyph["geometry"]["points"], black_glyph["geometry"]["heavy"]
        ))
    )
    assert medium_glyph["tree"]
    assert small_glyph["usedOpticalVariant"] is True
    assert large_glyph["usedOpticalVariant"] is True
    assert small_glyph["geometry"]["regular"] != medium_glyph["geometry"]["regular"]
    assert large_glyph["geometry"]["regular"] != medium_glyph["geometry"]["regular"]
    assert glyph_json(ctypes.create_string_buffer(b"missing"), 7, 0, 400.0, None, 0) == -6
    assert glyph_json(ctypes.create_string_buffer(b"heart.fill"), 10, 3, 400.0, None, 0) == -9
    assert glyph_json(ctypes.create_string_buffer(b"heart.fill"), 10, 0, float("nan"), None, 0) == -10

    def animation(request):
        encoded = json.dumps(request, separators=(",", ":")).encode("utf-8")
        input_buffer = ctypes.create_string_buffer(encoded)
        required = animation_json(input_buffer, len(encoded), None, 0)
        assert required > 0, f"animation sizing call failed: {required}"
        output = ctypes.create_string_buffer(required)
        written = animation_json(input_buffer, len(encoded), output, required)
        assert written == required
        return json.loads(output.raw[:written])

    appear = animation({"symbol": "heart.fill", "time": 0.125,
                        "effect": {"type": "appear"},
                        "transition": {"direction": "down_up", "time": 0.25}})
    assert appear["held"] is True and appear["defaultRepeat"] == "once"
    assert abs(appear["layers"][0]["alpha"] - 0.5) < 0.01
    assert abs(appear["layers"][0]["scale"] - 0.875) < 0.01
    assert "appearance" not in appear["layers"][0]
    assert appear["transition"]["out"]["alpha"] == 0.0
    assert abs(appear["transition"]["out"]["scale"] - 0.5) < 0.01
    assert animation({"symbol": "wifi", "time": 0.5,
                      "effect": {"type": "variable_color"}, "variableValue": 0.5})["layers"]
    for effect in (
        {"type": "disappear"},
        {"type": "bounce", "direction": "down", "scope": "by_layer"},
        {"type": "pulse", "scope": "by_layer"},
        {"type": "variable_color", "style": "cumulative", "reversing": True,
         "hideInactiveLayers": True},
        {"type": "scale", "up": False, "returns": True},
        {"type": "wiggle", "direction": {"custom": 37.0}, "continuous": True},
        {"type": "rotate", "direction": "counter_clockwise", "scope": "by_layer",
         "continuous": True},
        {"type": "breathe", "pulsing": True},
        {"type": "draw_on", "scope": "by_layer"},
        {"type": "draw_on", "scope": "by_layer", "individually": True},
        {"type": "draw_off"},
    ):
        assert animation({"symbol": "wifi", "time": 0.5, "rtl": True,
                          "effect": effect})["layers"]
    magic = animation({"symbol": "wifi", "time": 0.5, "effect": {"type": "pulse"},
                       "magicReplace": {"from": "wifi", "to": "wifi", "progress": 0.5}})
    assert magic["magicReplace"]["matchedLayers"]
    assert all(layer["ops"] and len(layer["points"]) for layer in magic["magicReplace"]["matchedLayers"])
    paired_magic = animation({
        "symbol": "xmark.circle.fill", "time": 0.5, "effect": {"type": "pulse"},
        "magicReplace": {"from": "checkmark.circle.fill", "to": "xmark.circle.fill", "progress": 0.5},
    })["magicReplace"]
    assert paired_magic["from"] == "checkmark.circle.fill"
    assert paired_magic["to"] == "xmark.circle.fill"
    assert paired_magic["matchedLayers"]

    multicolor = animation({
        "symbol": "cloud.sun.fill", "time": 0.5, "effect": {"type": "pulse"},
        "rendering": {"mode": "multicolor", "tint": [0.2, 0.4, 0.6, 0.8], "gradient": True},
    })
    appearances = [layer["appearance"] for layer in multicolor["layers"]]
    assert appearances and all("color" in appearance and "gradient" in appearance for appearance in appearances)
    assert any(appearance["gradient"] is not None for appearance in appearances)
    palette = animation({
        "symbol": "wifi", "time": 0.5, "effect": {"type": "pulse"},
        "rendering": {
            "mode": "palette", "tint": [0.1, 0.2, 0.3, 1.0],
            "palette": [[1.0, 0.0, 0.0, 1.0], [0.0, 0.0, 1.0, 0.5]],
        },
    })
    assert all("appearance" in layer for layer in palette["layers"])
    invalid_rendering = ctypes.create_string_buffer(
        b'{"symbol":"wifi","time":0,"effect":{"type":"appear"},'
        b'"rendering":{"mode":"palette","tint":[2,0,0,1]}}'
    )
    assert animation_json(invalid_rendering, len(invalid_rendering.value), None, 0) == -11

    def draw_field(request):
        encoded = json.dumps(request, separators=(",", ":")).encode("utf-8")
        input_buffer = ctypes.create_string_buffer(encoded)
        required = draw_field_json(input_buffer, len(encoded), None, 0)
        assert required > 0, f"draw-field sizing call failed: {required}"
        output = ctypes.create_string_buffer(required)
        written = draw_field_json(input_buffer, len(encoded), output, required)
        assert written == required
        return json.loads(output.raw[:written])

    draw_mask = [0] * (112 * 112)
    draw_row = 55 * 112
    draw_mask[draw_row + 55:draw_row + 60] = [1] * 5
    draw_request = {
        "mask": draw_mask, "origin": [0.0, 0.0], "cell": 1.0,
        "symbolBounds": [0.0, 0.0, 112.0, 112.0],
        "ties": [55.5 / 112, 55.5 / 112],
    }
    draw_levels = draw_field(draw_request)
    assert (draw_levels["width"], draw_levels["height"]) == (112, 112)
    assert draw_levels["levels"][draw_row + 55:draw_row + 60] == [0, 12, 24, 36, 47]
    assert draw_levels["levels"][draw_row + 54] == 255
    invalid_draw_mask = ctypes.create_string_buffer(b'{"mask":[]}')
    assert draw_field_json(invalid_draw_mask, len(invalid_draw_mask.value), None, 0) == -12

    def mcp_client_config(app_path, development_lib_path):
        app_bytes = app_path.encode("utf-8") if app_path is not None else b""
        app_buffer = ctypes.create_string_buffer(app_bytes) if app_path is not None else None
        development_bytes = development_lib_path.encode("utf-8")
        development_buffer = ctypes.create_string_buffer(development_bytes)

        def call(output, capacity):
            return mcp_client_config_json(
                app_buffer,
                len(app_bytes),
                0 if app_path is None else 1,
                development_buffer,
                len(development_bytes),
                output,
                capacity,
            )

        required = call(None, 0)
        assert required > 0, f"MCP client config sizing call failed: {required}"
        output = ctypes.create_string_buffer(required)
        written = call(output, required)
        assert written == required
        return json.loads(output.raw[:written])

    installed_config = mcp_client_config('C:/Apps/SFStudio/"Pilot" café.exe', "/unused")
    assert installed_config["mcpServers"]["sfstudio"] == {
        "command": 'C:/Apps/SFStudio/"Pilot" café.exe', "args": ["--mcp"]
    }
    development_config = mcp_client_config(None, "C:/build folder/app/lib")
    assert development_config["mcpServers"]["sfstudio"] == {
        "command": "java",
        "args": [
            "-Djava.awt.headless=true", "-cp", "C:/build folder/app/lib/*",
            "com.sfsymbols.MainKt", "--mcp",
        ],
    }
    assert mcp_client_config_json(None, 0, 2, None, 0, None, 0) == -13

    def filter_selection(action, category=None):
        encoded = category.encode("utf-8") if category is not None else b""
        category_buffer = ctypes.create_string_buffer(encoded) if category is not None else None

        def call(output, capacity):
            return filter_selection_json(
                action,
                category_buffer,
                len(encoded),
                0 if category is None else 1,
                output,
                capacity,
            )

        required = call(None, 0)
        assert required > 0, f"filter selection sizing call failed: {required}"
        output = ctypes.create_string_buffer(required)
        written = call(output, required)
        assert written == required
        return json.loads(output.raw[:written])

    assert filter_selection(0, "天气") == ["天气", False]
    assert filter_selection(0, "") == ["", False]
    assert filter_selection(1) == [None, True]
    assert filter_selection(2) == [None, False]
    assert filter_selection_json(3, None, 0, 0, None, 0) == -14

    def filtered_search(query, category, favorites, limit):
        encoded_query = query.encode("utf-8")
        encoded_category = category.encode("utf-8") if category is not None else b""
        encoded_favorites = json.dumps(favorites).encode("utf-8") if favorites is not None else b""
        query_buffer = ctypes.create_string_buffer(encoded_query) if encoded_query else None
        category_buffer = ctypes.create_string_buffer(encoded_category) if encoded_category else None
        favorites_buffer = ctypes.create_string_buffer(encoded_favorites) if encoded_favorites else None
        args = (
            query_buffer, len(encoded_query), category_buffer, len(encoded_category),
            favorites_buffer, len(encoded_favorites), int(favorites is not None), limit,
        )
        required = search_filtered_json(*args, None, 0)
        assert required >= 0, f"filtered search sizing call failed: {required}"
        output = ctypes.create_string_buffer(required)
        written = search_filtered_json(*args, output, required)
        assert written == required
        return json.loads(output.raw[:written])

    assert set(filtered_search("", None, ["heart.fill", "star"], 10)) == {"heart.fill", "star"}

    query = ctypes.create_string_buffer(b"home")
    short = ctypes.create_string_buffer(b"\xa5", 1)
    required = search_json(query, 4, 1, short, 1)
    assert required > 1 and short.raw == b"\xa5", "short buffer was partially modified"
    assert search_json(None, 1, 1, None, 0) == -1
    invalid_utf8 = ctypes.create_string_buffer(b"\xff")
    assert search_json(invalid_utf8, 1, 1, None, 0) == -2
    assert search_json(query, 4, 1, None, 1) == -3

    def snippet(mode, tint, tint_enabled, background, background_enabled):
        name = b"heart.fill"
        name_buffer = ctypes.create_string_buffer(name)
        required = compose_snippet(
            name_buffer, len(name), mode, ctypes.c_int32(tint), tint_enabled,
            ctypes.c_int32(background), background_enabled, None, 0,
        )
        assert required >= 0, f"snippet sizing call failed: {required}"
        output = ctypes.create_string_buffer(required)
        written = compose_snippet(
            name_buffer, len(name), mode, ctypes.c_int32(tint), tint_enabled,
            ctypes.c_int32(background), background_enabled, output, required,
        )
        assert written == required
        return output.raw[:written].decode("utf-8")

    golden_dir = ROOT / "app/src/jvmTest/resources"
    assert snippet(1, 0x80E0245E, 1, 0xFFDCE9D3, 1) == (
        golden_dir / "compose-snippet-monochrome-background.kt.txt"
    ).read_text(encoding="utf-8")
    assert snippet(0, 0, 0, 0, 0) == (
        golden_dir / "compose-snippet-dualtone-plain.kt.txt"
    ).read_text(encoding="utf-8")

    def toggle(json_text, name):
        encoded_json = json_text.encode("utf-8")
        encoded_name = name.encode("utf-8")
        json_buffer = ctypes.create_string_buffer(encoded_json)
        name_buffer = ctypes.create_string_buffer(encoded_name)
        required = toggle_favorites(
            json_buffer, len(encoded_json), name_buffer, len(encoded_name), None, 0,
        )
        assert required >= 0, f"favorites sizing call failed: {required}"
        output = ctypes.create_string_buffer(required)
        written = toggle_favorites(
            json_buffer, len(encoded_json), name_buffer, len(encoded_name), output, required,
        )
        assert written == required
        return json.loads(output.raw[:written])

    assert toggle('["heart.fill","star"]', "gear") == ["heart.fill", "star", "gear"]
    assert toggle('["heart.fill","star"]', "heart.fill") == ["star"]
    broken_json = ctypes.create_string_buffer(b"{broken")
    assert animation_json(broken_json, 7, None, 0) == -11
    negative_time = ctypes.create_string_buffer(
        b'{"symbol":"wifi","time":-0.1,"effect":{"type":"appear"}}'
    )
    assert animation_json(negative_time, len(negative_time.value), None, 0) == -11
    invalid_morph = ctypes.create_string_buffer(
        b'{"symbol":"wifi","time":0,"effect":{"type":"appear"},'
        b'"magicReplace":{"from":"wifi","to":"wifi","progress":1.1}}'
    )
    assert animation_json(invalid_morph, len(invalid_morph.value), None, 0) == -11
    missing_animation = ctypes.create_string_buffer(
        b'{"symbol":"missing","time":0,"effect":{"type":"appear"}}'
    )
    assert animation_json(missing_animation, len(missing_animation.value), None, 0) == -6
    assert toggle_favorites(broken_json, 7, ctypes.create_string_buffer(b"gear"), 4,
                            None, 0) == -8

    print("Rust C ABI search, metadata, vectors, Apple animations, draw fields, MCP config, filtering, codegen, and favorites smoke test passed (Python ctypes)")


if __name__ == "__main__":
    main()
