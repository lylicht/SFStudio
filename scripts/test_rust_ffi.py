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

    query = ctypes.create_string_buffer(b"home")
    short = ctypes.create_string_buffer(b"\xa5", 1)
    required = search_json(query, 4, 1, short, 1)
    assert required > 1 and short.raw == b"\xa5", "short buffer was partially modified"
    assert search_json(None, 1, 1, None, 0) == -1
    invalid_utf8 = ctypes.create_string_buffer(b"\xff")
    assert search_json(invalid_utf8, 1, 1, None, 0) == -2
    assert search_json(query, 4, 1, None, 1) == -3

    print("Rust C ABI smoke test passed (Python ctypes)")


if __name__ == "__main__":
    main()
