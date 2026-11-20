"""Exercise the assembled MCP server through real stdin/stdout pipes (no third-party packages)."""
import argparse
import json
import os
from pathlib import Path
import queue
import subprocess
import threading
import zipfile


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--java", default="java", help="Java 21+ executable")
    launch = parser.add_mutually_exclusive_group()
    launch.add_argument("--rust-binary", help="Run the Rust server through the existing --mcp launcher")
    launch.add_argument("--launcher", help="Run a packaged SFStudio launcher with --mcp")
    args = parser.parse_args()
    lib = Path(__file__).resolve().parents[1] / "app/build/mcp/lib"
    if args.launcher:
        launcher = Path(args.launcher).resolve()
        if not launcher.is_file():
            parser.error(f"SFStudio launcher not found: {launcher}")
        resource = "native/sfstudio-mcp.exe" if launcher.suffix == ".exe" else "native/sfstudio-mcp"
        app_jars = [
            jar
            for directory in (launcher.parent / "app", launcher.parent.parent / "app")
            if directory.is_dir()
            for jar in directory.glob("app-jvm*.jar")
        ]
        rust_bundled = False
        for jar in app_jars:
            if zipfile.is_zipfile(jar):
                with zipfile.ZipFile(jar) as archive:
                    rust_bundled |= resource in archive.namelist()
        if not rust_bundled:
            parser.error(f"Packaged Rust MCP resource missing near launcher: {resource}")
        command = [str(launcher), "--mcp"]
    else:
        if not lib.is_dir():
            parser.error("Run ./gradlew assembleMcp first")
        command = [args.java, "-Djava.awt.headless=true"]
    if args.rust_binary:
        rust_binary = Path(args.rust_binary).resolve()
        if not rust_binary.is_file():
            parser.error(f"Rust MCP binary not found: {rust_binary}")
        rust_resource = "native/sfstudio-mcp.exe" if rust_binary.suffix == ".exe" else "native/sfstudio-mcp"
        app_jar = lib / "app-jvm.jar"
        if not app_jar.is_file() or not zipfile.is_zipfile(app_jar):
            parser.error("Assemble the app JAR with :app:assembleMcp first")
        with zipfile.ZipFile(app_jar) as archive:
            if rust_resource not in archive.namelist():
                parser.error(f"Rust MCP resource missing from app JAR: {rust_resource}")
        command.append(f"-Dcompose.application.resources.dir={rust_binary.parent}")
    if not args.launcher:
        command.extend(["-cp", str(lib / "*"), "com.sfsymbols.MainKt", "--mcp"])
    env = os.environ.copy()
    if args.rust_binary or args.launcher:
        env["SFSTUDIO_REQUIRE_RUST_MCP"] = "1"
    else:
        env.pop("SFSTUDIO_REQUIRE_RUST_MCP", None)
    process = subprocess.Popen(
        command,
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        text=True, encoding="utf-8", bufsize=1, env=env,
    )
    replies = queue.Queue()

    def read_replies():
        for line in process.stdout:
            replies.put(line)

    threading.Thread(target=read_replies, daemon=True).start()

    def send(message):
        process.stdin.write(json.dumps(message) + "\n")
        process.stdin.flush()

    def receive(expected_id):
        response = json.loads(replies.get(timeout=20))
        assert response["jsonrpc"] == "2.0", response
        assert response["id"] == expected_id, response
        return response

    try:
        send({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
            "protocolVersion": "2025-11-25", "capabilities": {},
            "clientInfo": {"name": "catalog-smoke-test", "version": "1"}}})
        assert receive(1)["result"]["protocolVersion"] == "2025-11-25"
        send({"jsonrpc": "2.0", "method": "notifications/initialized"})
        # One write deliberately pipelines requests to detect lost buffered input.
        requests = [
            {"id": 2, "method": "tools/list"},
            {"id": 3, "method": "tools/call", "params": {"name": "search_symbols", "arguments": {"query": "ai", "limit": 50}}},
            {"id": 4, "method": "tools/call", "params": {"name": "get_symbol", "arguments": {"code": "SFHeartFill"}}},
            {"id": 5, "method": "tools/call", "params": {"name": "search_symbols", "arguments": {"query": "a", "category": "weather", "limit": 1}}},
            {"id": 6, "method": "tools/call", "params": {"name": "get_symbol", "arguments": {"code": "missing-symbol"}}},
            {"id": 7, "method": "not_a_method"},
            {"id": 8, "method": "tools/call", "params": {"name": "search_symbols", "arguments": {"query": "heart", "limit": "wrong"}}},
            {"id": 9, "method": "ping"},
        ]
        process.stdin.write("".join(json.dumps({"jsonrpc": "2.0", **r}) + "\n" for r in requests))
        process.stdin.flush()
        responses = {i: receive(i) for i in range(2, 10)}
        assert {t["name"] for t in responses[2]["result"]["tools"]} == {"search_symbols", "get_symbol"}

        def content(request_id):
            return json.loads(responses[request_id]["result"]["content"][0]["text"])

        assert "sparkles" in {s["appleName"] for s in content(3)["symbols"]}
        assert content(4)["appleName"] == "heart.fill"
        assert content(5)["count"] == 1
        assert "weather" in [c.lower() for c in content(5)["symbols"][0]["categories"]]
        assert responses[6]["result"]["isError"] is True
        assert responses[7]["error"]["code"] == -32601
        assert responses[8]["error"]["code"] == -32602
        assert responses[9]["result"] == {}
        send({"jsonrpc": "2.0", "id": True, "method": "ping"})
        assert receive(None)["error"]["code"] == -32600
        send({"jsonrpc": "2.0", "id": 10, "method": "ping", "params": []})
        assert receive(10)["error"]["code"] == -32602
        process.stdin.write("{broken\n")
        process.stdin.flush()
        assert receive(None)["error"]["code"] == -32700
        send({"jsonrpc": "2.0", "id": 11, "method": "tools/call", "params": {
            "name": "search_symbols", "arguments": {"query": "naïve 🐈", "limit": 1}}})
        assert receive(11)["result"]["content"]
        process.stdin.close()
        assert process.wait(timeout=20) == 0, process.stderr.read()
        assert "falling back to Kotlin" not in process.stderr.read(), "Packaged launcher used Kotlin fallback"
        assert replies.empty(), "Unexpected stdout or notification response"
        print("MCP smoke test passed: handshake, tools, pipelining, Unicode, invalid requests, errors, stdout purity, and clean EOF.")
    finally:
        if process.poll() is None:
            process.kill()
            process.wait()


if __name__ == "__main__":
    main()
