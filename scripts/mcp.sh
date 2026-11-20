#!/bin/sh
set -eu
mcp_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
if [ -x "$mcp_dir/bin/sfstudio-mcp" ]; then
    exec "$mcp_dir/bin/sfstudio-mcp" "$@"
fi
mcp_java=java
if [ -n "${JAVA_HOME:-}" ]; then
    mcp_java="$JAVA_HOME/bin/java"
fi
exec "$mcp_java" -Djava.awt.headless=true -cp "$mcp_dir/lib/*" com.sfsymbols.MainKt --mcp
