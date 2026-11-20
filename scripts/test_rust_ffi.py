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
    assert toggle_favorites(broken_json, 7, ctypes.create_string_buffer(b"gear"), 4,
                            None, 0) == -8

    print("Rust C ABI search, filtering, codegen, and favorites smoke test passed (Python ctypes)")


if __name__ == "__main__":
    main()
