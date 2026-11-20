//! Standalone Rust implementation of SFStudio's headless MCP server.
//! Packaged desktop `--mcp` launches use this server; development classpath
//! launches retain the Kotlin implementation.

use serde_json::{json, Value};
use sfstudio_core::{catalog, Symbol};
use std::io::{self, BufRead, Write};

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut writer = io::BufWriter::new(stdout.lock());
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Some(response) = handle_line(&line) {
            writeln!(writer, "{response}")?;
            writer.flush()?;
        }
    }
    Ok(())
}

fn error(id: Value, code: i64, message: impl Into<String>) -> Value {
    json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message.into()}})
}

fn success(id: Value, result: Value) -> Value {
    json!({"jsonrpc":"2.0","id":id,"result":result})
}

fn handle_line(line: &str) -> Option<Value> {
    let value: Value = match serde_json::from_str(line) {
        Ok(value) => value,
        Err(_) => return Some(error(Value::Null, -32700, "Parse error")),
    };
    let Some(message) = value.as_object() else {
        return Some(error(Value::Null, -32600, "Invalid request"));
    };
    let id = message.get("id");
    let valid_id = id.is_some_and(|value| value.is_string() || value.as_i64().is_some());
    let method = message.get("method").and_then(Value::as_str);
    if message.get("jsonrpc").and_then(Value::as_str) != Some("2.0")
        || method.is_none()
        || (id.is_some() && !valid_id)
    {
        return Some(error(
            if valid_id {
                id.cloned().unwrap_or(Value::Null)
            } else {
                Value::Null
            },
            -32600,
            "Invalid request",
        ));
    }

    // JSON-RPC notifications do not produce responses, including unknown ones.
    let id = id?.clone();
    let params = match message.get("params") {
        Some(Value::Object(map)) => map,
        Some(_) => return Some(error(id, -32602, "'params' must be an object")),
        None => {
            return Some(success_or_error(
                id,
                method.unwrap(),
                &serde_json::Map::new(),
            ))
        }
    };
    Some(success_or_error(id, method.unwrap(), params))
}

fn success_or_error(id: Value, method: &str, params: &serde_json::Map<String, Value>) -> Value {
    match handle_method(method, params) {
        Ok(result) => success(id, result),
        Err((code, message)) => error(id, code, message),
    }
}

type RpcResult = Result<Value, (i64, String)>;

fn invalid(message: impl Into<String>) -> (i64, String) {
    (-32602, message.into())
}

fn optional_string<'a>(
    args: &'a serde_json::Map<String, Value>,
    key: &str,
) -> Result<Option<&'a str>, (i64, String)> {
    match args.get(key) {
        Some(Value::String(value)) => Ok(Some(value)),
        Some(_) => Err(invalid(format!("'{key}' must be a string"))),
        None => Ok(None),
    }
}

fn required_string<'a>(
    args: &'a serde_json::Map<String, Value>,
    key: &str,
) -> Result<&'a str, (i64, String)> {
    optional_string(args, key)?
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| invalid(format!("'{key}' is required and must be a nonblank string")))
}

fn handle_method(method: &str, params: &serde_json::Map<String, Value>) -> RpcResult {
    match method {
        "initialize" => {
            let requested = required_string(params, "protocolVersion")?;
            let version = match requested {
                "2025-11-25" | "2025-06-18" | "2025-03-26" | "2024-11-05" => requested,
                _ => "2025-11-25",
            };
            Ok(json!({
                "protocolVersion":version,
                "capabilities":{"tools":{}},
                "serverInfo":{"name":"sfstudio","version":"2.0.0"}
            }))
        }
        "ping" => Ok(json!({})),
        "tools/list" => Ok(tools_list()),
        "tools/call" => {
            let name = required_string(params, "name")?;
            let empty = serde_json::Map::new();
            let args = match params.get("arguments") {
                Some(Value::Object(map)) => map,
                Some(_) => return Err(invalid("'arguments' must be an object")),
                None => &empty,
            };
            match name {
                "search_symbols" => search_symbols(args),
                "get_symbol" => get_symbol(args),
                _ => Err(invalid(format!("Unknown tool: {name}"))),
            }
        }
        _ => Err((-32601, format!("Method not found: {method}"))),
    }
}

fn tools_list() -> Value {
    json!({"tools":[
        {
            "name":"search_symbols",
            "description":"Fuzzy, typo-tolerant, ranked search of the SF Symbols catalog. Returns symbol metadata matching the query.",
            "inputSchema":{
                "type":"object",
                "properties":{
                    "query":{"type":"string","description":"Search text, e.g. 'heart' or 'arrow.right'"},
                    "category":{"type":"string","description":"Optional category to filter within, e.g. 'objects'"},
                    "limit":{"type":"integer","minimum":1,"maximum":300,
                        "description":"Max results to return (default 25)"}
                },
                "required":["query"]
            }
        },
        {
            "name":"get_symbol",
            "description":"Resolve a symbol's pascal code (e.g. 'SFHeartFill') or apple name (e.g. 'heart.fill') to its metadata: categories, the Compose expression for each variant and the import it needs.",
            "inputSchema":{
                "type":"object",
                "properties":{
                    "code":{"type":"string","description":"Pascal name like 'SFHeartFill' or apple name like 'heart.fill'"}
                },
                "required":["code"]
            }
        }
    ]})
}

fn search_symbols(args: &serde_json::Map<String, Value>) -> RpcResult {
    let query = required_string(args, "query")?;
    let category = optional_string(args, "category")?;
    let limit = match args.get("limit") {
        None => 25,
        Some(value) => match value.as_i64() {
            Some(value @ 1..=300) => value as usize,
            _ => return Err(invalid("'limit' must be an integer from 1 to 300")),
        },
    };
    // Preserve Kotlin behavior: scope AFTER scoring, BEFORE applying limit.
    let index = catalog();
    let symbols: Vec<Value> = index
        .search(query, index.all().len())
        .into_iter()
        .filter(|s| {
            category.map_or(true, |cat| {
                s.categories.iter().any(|c| c.eq_ignore_ascii_case(cat))
            })
        })
        .take(limit)
        .map(symbol_json)
        .collect();
    Ok(text_content(
        json!({"count":symbols.len(),"symbols":symbols}),
    ))
}

fn get_symbol(args: &serde_json::Map<String, Value>) -> RpcResult {
    let code = required_string(args, "code")?;
    match catalog().find(code) {
        Some(symbol) => Ok(text_content(symbol_json(symbol))),
        None => Ok(json!({
            "isError":true,
            "content":[{"type":"text","text":format!("No symbol found for '{code}'")}]
        })),
    }
}

fn symbol_json(symbol: &Symbol) -> Value {
    json!({
        "appleName":symbol.apple_name,
        "pascalName":symbol.pascal_name,
        "code":format!("SfSymbols.Dualtone.{}",symbol.pascal_name),
        "monochromeCode":format!("SfSymbols.Monochrome.{}",symbol.pascal_name),
        "import":format!("com.composables.sfsymbols.dualtone.{}",symbol.pascal_name),
        "monochromeImport":format!("com.composables.sfsymbols.monochrome.{}",symbol.pascal_name),
        "categories":symbol.categories,
        "isRestricted":symbol.is_restricted
    })
}

fn text_content(value: Value) -> Value {
    json!({"content":[{"type":"text","text":value.to_string()}]})
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response(line: &str) -> Value {
        handle_line(line).unwrap()
    }

    #[test]
    fn handles_handshake_and_notifications() {
        let r = response(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25"}}"#,
        );
        assert_eq!(r["result"]["protocolVersion"], "2025-11-25");
        assert_eq!(r["result"]["serverInfo"]["name"], "sfstudio");
        assert!(handle_line(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#).is_none());
    }

    #[test]
    fn exposes_tools_and_symbol_lookup() {
        let r = response(r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#);
        assert_eq!(r["result"]["tools"].as_array().unwrap().len(), 2);
        let r = response(
            r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_symbol","arguments":{"code":"SFHeartFill"}}}"#,
        );
        let json: Value =
            serde_json::from_str(r["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
        assert_eq!(json["appleName"], "heart.fill");
    }

    #[test]
    fn input_validation_matches_jvm() {
        assert_eq!(response("{broken")["error"]["code"], -32700);
        assert_eq!(response("[]")["error"]["code"], -32600);
        assert_eq!(
            response(r#"{"jsonrpc":"2.0","id":1,"method":"unknown"}"#)["error"]["code"],
            -32601
        );
        assert_eq!(
            response(
                r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"search_symbols","arguments":{"query":"a","limit":0}}}"#
            )["error"]["code"],
            -32602
        );
    }
}
