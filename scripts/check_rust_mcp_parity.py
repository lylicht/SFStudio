#!/usr/bin/env python3
"""Differentially replay MCP calls against Kotlin and Rust SFStudio servers.

Requirements: JDK 21+, Rust toolchain; assemble both servers first:
    ./gradlew :app:assembleMcp
    cargo build --manifest-path rust/Cargo.toml -p sfstudio-mcp
    python scripts/check_rust_mcp_parity.py

Does not modify favorites, user data or the installed SFStudio application.
"""
import argparse
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def requests():
    yield {"jsonrpc": "2.0", "id": 1, "method": "initialize",
           "params": {"protocolVersion": "2025-11-25", "capabilities": {}}}
    yield {"jsonrpc": "2.0", "method": "notifications/initialized"}
    yield {"jsonrpc": "2.0", "id": 2, "method": "tools/list"}
    yield {"jsonrpc": "2.0", "id": 3, "method": "ping"}

    queries = [
        "home", "house", "phone", "garbage", "logout", "ai", "cog",
        "money", "chekmark", "weather", "hsf", "heart", "tradh",
        "trsh", "hosue", "gearshap", "garbgae", "settigns",
        "msg", "pw", "pics", "bunny", "zap", "doggy paw",
        "four boxes", "4 cubes stacked on top of each other",
        "a yucky can", "4 circle", "arrow.right",
        "arrow.right.circle", "rectangle.portrait.and.arrow.right",
        "zzzzzz", "🐈", "naïve", "   ", "a", "house fill",
    ]
    index = 4
    for query in queries:
        for limit in (1, 12):
            yield {"jsonrpc": "2.0", "id": index, "method": "tools/call",
                   "params": {"name": "search_symbols",
                              "arguments": {"query": query, "limit": limit}}}
            index += 1

    for query, category in [("a", "Weather"), ("heart", "Shapes"),
                            ("garbage", "Objects & Tools"), ("a", "not-category")]:
        yield {"jsonrpc": "2.0", "id": index, "method": "tools/call",
               "params": {"name": "search_symbols",
                          "arguments": {"query": query, "category": category, "limit": 5}}}
        index += 1

    for code in ("SFHeartFill", "heart.fill", "HOUSE", "xserve.raid", "unknown-code"):
        yield {"jsonrpc": "2.0", "id": index, "method": "tools/call",
               "params": {"name": "get_symbol", "arguments": {"code": code}}}
        index += 1

    for args in ({"query": "heart", "limit": 0},
                 {"query": "heart", "limit": 301},
                 {"query": 123},
                 {"query": ""},
                 {"query": "heart", "category": 10}):
        yield {"jsonrpc": "2.0", "id": index, "method": "tools/call",
               "params": {"name": "search_symbols", "arguments": args}}
        index += 1

    yield {"jsonrpc": "2.0", "id": index, "method": "not_a_method"}


def run_server(command, input_text):
    done = subprocess.run(command, input=input_text, text=True,
                          stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                          encoding="utf-8", cwd=ROOT, timeout=150, check=False)
    if done.returncode:
        raise RuntimeError(f"{command[0]} exited {done.returncode}: {done.stderr[-3000:]}")
    return [json.loads(line) for line in done.stdout.splitlines() if line.strip()]


def normalize(value):
    if isinstance(value, dict):
        return {key: normalize(item) if key != "text" else parse_text(item)
                for key, item in value.items()}
    if isinstance(value, list):
        return [normalize(item) for item in value]
    return value


def parse_text(text):
    if not isinstance(text, str):
        return text
    try:
        return normalize(json.loads(text))
    except json.JSONDecodeError:
        return text


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--java", default="java")
    parser.add_argument("--rust-binary",
                        default=str(ROOT / "rust/target/debug" /
                                    ("sfstudio-mcp.exe" if sys.platform == "win32" else "sfstudio-mcp")))
    args = parser.parse_args()

    kotlin_lib = ROOT / "app/build/mcp/lib"
    if not kotlin_lib.is_dir():
        parser.error("Run ./gradlew :app:assembleMcp first")
    rust_bin = Path(args.rust_binary)
    if not rust_bin.is_file():
        parser.error("Run cargo build --manifest-path rust/Cargo.toml -p sfstudio-mcp first")

    messages = list(requests())
    payload = "".join(json.dumps(m, ensure_ascii=False) + "\n" for m in messages)
    original = run_server(
        [args.java, "-Djava.awt.headless=true", "-cp",
         str(kotlin_lib / "*"), "com.sfsymbols.MainKt", "--mcp"], payload)
    port = run_server([str(rust_bin)], payload)

    if len(original) != len(port):
        raise AssertionError(f"Different response count: Kotlin={len(original)}, Rust={len(port)}")
    failures = []
    for number, (left, right) in enumerate(zip(original, port), 1):
        if normalize(left) != normalize(right):
            failures.append((number, left.get("id"), left, right))

    if failures:
        for ordinal, identifier, original_case, port_case in failures[:8]:
            print(f"Mismatch {ordinal} (request id={identifier}):")
            print("  Kotlin:", json.dumps(normalize(original_case), ensure_ascii=False)[:1200])
            print("  Rust:  ", json.dumps(normalize(port_case), ensure_ascii=False)[:1200])
        raise SystemExit(f"{len(failures)} / {len(original)} MCP responses differ")

    print(f"PASS: {len(original)} Kotlin and Rust MCP responses match (parsed JSON contents).")


if __name__ == "__main__":
    main()
