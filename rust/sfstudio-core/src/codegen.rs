//! Portable generation of the Compose snippets copied by the desktop app.

use crate::Symbol;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SymbolMode {
    Dualtone,
    Monochrome,
}

pub fn compose_snippet(
    symbol: &Symbol,
    mode: SymbolMode,
    tint_argb: i32,
    tint_enabled: bool,
    background_argb: Option<i32>,
) -> String {
    let (variant, package) = match mode {
        SymbolMode::Dualtone => ("Dualtone", "dualtone"),
        SymbolMode::Monochrome => ("Monochrome", "monochrome"),
    };
    let color = |argb: i32| format!("Color(0x{:08X})", argb as u32);
    let tint = if tint_enabled {
        color(tint_argb)
    } else {
        "Color.Unspecified".to_owned()
    };

    let mut imports = vec![
        "androidx.compose.foundation.layout.size".to_owned(),
        "androidx.compose.material3.Icon".to_owned(),
        "androidx.compose.runtime.Composable".to_owned(),
        "androidx.compose.ui.Modifier".to_owned(),
        "androidx.compose.ui.graphics.Color".to_owned(),
        "androidx.compose.ui.unit.dp".to_owned(),
        "com.composables.sfsymbols.SfSymbols".to_owned(),
        format!("com.composables.sfsymbols.{package}.{}", symbol.pascal_name),
    ];
    if background_argb.is_some() {
        imports.extend([
            "androidx.compose.foundation.background".to_owned(),
            "androidx.compose.foundation.layout.Box".to_owned(),
            "androidx.compose.foundation.shape.RoundedCornerShape".to_owned(),
            "androidx.compose.ui.Alignment".to_owned(),
            "androidx.compose.ui.draw.clip".to_owned(),
        ]);
    }
    imports.sort_unstable();

    let icon = format!(
        "Icon(\n    imageVector = SfSymbols.{variant}.{},\n    contentDescription = \"{}\",\n    tint = {tint},\n    modifier = {}.size(110.dp),\n)",
        symbol.pascal_name,
        symbol.apple_name,
        if background_argb.is_some() {
            "Modifier"
        } else {
            "modifier"
        },
    );
    let body = match background_argb {
        Some(background) => format!(
            "Box(\n    modifier = modifier\n        .size(240.dp)\n        .clip(RoundedCornerShape(10.dp))\n        .background({}),\n    contentAlignment = Alignment.Center,\n) {{\n{}\n}}",
            color(background),
            indent(&icon, 4),
        ),
        None => icon,
    };

    format!(
        "// Add the icon library to your module dependencies:\n\
         // implementation(\"com.github.lylicht.sfstudio:sfsymbols:TAG\")\n\
         // Add maven(\"https://jitpack.io\") to your dependency repositories.\n\n\
         {}\n\n\
         @Composable\nfun {}Icon(modifier: Modifier = Modifier) {{\n{}\n}}\n",
        imports
            .iter()
            .map(|import| format!("import {import}"))
            .collect::<Vec<_>>()
            .join("\n"),
        symbol.pascal_name,
        indent(&body, 4),
    )
}

fn indent(text: &str, spaces: usize) -> String {
    let prefix = " ".repeat(spaces);
    text.lines()
        .map(|line| format!("{prefix}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(path: &str) -> String {
        let text = match path {
            "background" => include_str!(
                "../../../app/src/jvmTest/resources/compose-snippet-monochrome-background.kt.txt"
            ),
            "plain" => include_str!(
                "../../../app/src/jvmTest/resources/compose-snippet-dualtone-plain.kt.txt"
            ),
            _ => unreachable!(),
        };
        text.replace("\r\n", "\n")
    }

    #[test]
    fn matches_kotlin_snippet_with_background() {
        let symbol = crate::catalog().find("heart.fill").unwrap();
        assert_eq!(
            compose_snippet(
                symbol,
                SymbolMode::Monochrome,
                0x80E0_245E_u32 as i32,
                true,
                Some(0xFFDC_E9D3_u32 as i32),
            ),
            fixture("background"),
        );
    }

    #[test]
    fn matches_kotlin_snippet_without_background_or_tint() {
        let symbol = crate::catalog().find("heart.fill").unwrap();
        assert_eq!(
            compose_snippet(symbol, SymbolMode::Dualtone, 0, false, None),
            fixture("plain"),
        );
    }
}
