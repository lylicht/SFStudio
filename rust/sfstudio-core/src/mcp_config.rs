//! MCP client configuration copied from the desktop Settings screen.

/// Builds the client configuration for either an installed launcher or the dev classpath.
pub fn client_config(
    app_path: Option<&str>,
    development_lib_path: &str,
) -> Result<String, serde_json::Error> {
    let development_classpath = format!("{development_lib_path}/*");
    let (command, args) = match app_path {
        Some(path) => (path, vec!["--mcp"]),
        None => (
            "java",
            vec![
                "-Djava.awt.headless=true",
                "-cp",
                development_classpath.as_str(),
                "com.sfsymbols.MainKt",
                "--mcp",
            ],
        ),
    };
    let command = serde_json::to_string(command)?;
    let args = args
        .iter()
        .map(serde_json::to_string)
        .collect::<Result<Vec<_>, _>>()?
        .join(",\n                ");
    Ok(format!(
        "{{\n    \"mcpServers\": {{\n        \"sfstudio\": {{\n            \"command\": {command},\n            \"args\": [\n                {args}\n            ]\n        }}\n    }}\n}}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn config_preserves_installed_and_development_launch_contracts() {
        let installed_text =
            client_config(Some(r#"C:/Apps/SFStudio/"Pilot" café.exe"#), "/unused").unwrap();
        assert_eq!(
            installed_text,
            r#"{
    "mcpServers": {
        "sfstudio": {
            "command": "C:/Apps/SFStudio/\"Pilot\" café.exe",
            "args": [
                "--mcp"
            ]
        }
    }
}"#
        );
        let installed: Value = serde_json::from_str(&installed_text).unwrap();
        assert_eq!(
            installed["mcpServers"]["sfstudio"]["command"],
            r#"C:/Apps/SFStudio/"Pilot" café.exe"#
        );
        assert_eq!(
            installed["mcpServers"]["sfstudio"]["args"],
            serde_json::json!(["--mcp"])
        );

        let development_text = client_config(None, "C:/build folder/app/lib").unwrap();
        assert_eq!(
            development_text,
            r#"{
    "mcpServers": {
        "sfstudio": {
            "command": "java",
            "args": [
                "-Djava.awt.headless=true",
                "-cp",
                "C:/build folder/app/lib/*",
                "com.sfsymbols.MainKt",
                "--mcp"
            ]
        }
    }
}"#
        );
        let development: Value = serde_json::from_str(&development_text).unwrap();
        assert_eq!(development["mcpServers"]["sfstudio"]["command"], "java");
        assert_eq!(
            development["mcpServers"]["sfstudio"]["args"],
            serde_json::json!([
                "-Djava.awt.headless=true",
                "-cp",
                "C:/build folder/app/lib/*",
                "com.sfsymbols.MainKt",
                "--mcp"
            ])
        );
    }
}
