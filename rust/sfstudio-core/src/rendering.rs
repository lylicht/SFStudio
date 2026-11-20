//! SF Symbol layer color resolution and Apple's automatic two-stop gradient.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderingMode {
    Monochrome,
    Hierarchical,
    Palette,
    Multicolor,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgba {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl Rgba {
    pub const fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    fn with_alpha(self, alpha: f32) -> Self {
        Self { alpha, ..self }
    }
}

const HIERARCHICAL_ALPHA: [f32; 3] = [1.0, 0.5, 0.25];

pub fn layer_color(
    color_name: Option<&str>,
    mode: RenderingMode,
    tint: Rgba,
    palette: &[Rgba],
    hierarchical: i32,
    erases: bool,
    opacity: f32,
) -> Option<Rgba> {
    if color_name == Some("hidden") {
        return None;
    }
    match mode {
        RenderingMode::Monochrome => (!erases).then_some(tint),
        RenderingMode::Hierarchical => hierarchical_alpha(tint, hierarchical),
        RenderingMode::Palette => {
            if hierarchical < 0 {
                None
            } else {
                Some(
                    palette
                        .get(hierarchical as usize)
                        .or_else(|| palette.last())
                        .copied()
                        .unwrap_or_else(|| hierarchical_tint(tint, hierarchical)),
                )
            }
        }
        RenderingMode::Multicolor => {
            named_color(color_name, tint).map(|color| color.with_alpha(color.alpha * opacity))
        }
    }
}

pub fn named_color(name: Option<&str>, tint: Rgba) -> Option<Rgba> {
    let color = match name {
        None
        | Some("tintColor" | "labelColor" | "controlAccentColor" | "_gradient.highlegibility") => {
            tint
        }
        Some("erase" | "hidden") => return None,
        Some("white") => Rgba::new(1.0, 1.0, 1.0, 1.0),
        Some("black") => Rgba::new(0.0, 0.0, 0.0, 1.0),
        Some("secondaryLabelColor") => tint.with_alpha(tint.alpha * 0.5),
        Some("tertiaryLabelColor") => tint.with_alpha(tint.alpha * 0.3),
        Some("systemRedColor") => rgb(0xFF3B30),
        Some("systemOrangeColor") => rgb(0xFF9500),
        Some("systemYellowColor") => rgb(0xFFCC00),
        Some("systemGreenColor") => rgb(0x34C759),
        Some("systemMintColor") => rgb(0x00C7BE),
        Some("systemTealColor") => rgb(0x30B0C7),
        Some("systemCyanColor") => rgb(0x32ADE6),
        Some("systemBlueColor") => rgb(0x007AFF),
        Some("systemIndigoColor") => rgb(0x5856D6),
        Some("systemPurpleColor") => rgb(0xAF52DE),
        Some("systemPinkColor") => rgb(0xFF2D55),
        Some("systemBrownColor") => rgb(0xA2845E),
        Some("systemGrayColor") => rgb(0x8E8E93),
        Some(value) if value.len() == 7 && value.starts_with('#') => {
            match u32::from_str_radix(&value[1..], 16) {
                Ok(value) => rgb(value),
                Err(_) => tint,
            }
        }
        Some(_) => tint,
    };
    Some(color)
}

pub fn automatic_gradient(color: Rgba) -> (Rgba, Rgba) {
    let lightness = lightness(color);
    let amount = if lightness <= 0.25 {
        0.2
    } else if lightness <= 0.7 {
        0.25
    } else if lightness <= 0.82 {
        0.34
    } else {
        -0.11
    };
    let minimum = color.red.min(color.green).min(color.blue);
    let maximum = color.red.max(color.green).max(color.blue);
    let scale = headroom(minimum, lightness).min(headroom(maximum, lightness));
    let adjust = |value: f32| {
        let boosted = (value - lightness) * scale + value;
        (boosted + (if amount < 0.0 { boosted } else { 1.0 - boosted }) * amount).clamp(0.0, 1.0)
    };
    let other = Rgba::new(
        adjust(color.red),
        adjust(color.green),
        adjust(color.blue),
        color.alpha,
    );
    if lightness > 0.82 {
        (color, other)
    } else {
        (other, color)
    }
}

fn hierarchical_alpha(tint: Rgba, level: i32) -> Option<Rgba> {
    (level >= 0).then(|| hierarchical_tint(tint, level))
}

fn hierarchical_tint(tint: Rgba, level: i32) -> Rgba {
    tint.with_alpha(tint.alpha * HIERARCHICAL_ALPHA[(level as usize).min(2)])
}

fn rgb(value: u32) -> Rgba {
    Rgba::new(
        ((value >> 16) & 0xFF) as f32 / 255.0,
        ((value >> 8) & 0xFF) as f32 / 255.0,
        (value & 0xFF) as f32 / 255.0,
        1.0,
    )
}

fn headroom(value: f32, lightness: f32) -> f32 {
    let difference = value - lightness;
    if difference == 0.0 {
        return 0.2;
    }
    let target = if difference > 0.0 {
        1.0 - value
    } else {
        -value
    };
    (target / difference).clamp(0.0, 0.2)
}

fn lightness(color: Rgba) -> f32 {
    let linear = |value: f32| {
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    let luminance =
        0.2126 * linear(color.red) + 0.7152 * linear(color.green) + 0.0722 * linear(color.blue);
    if luminance > 0.008856 {
        (116.0 * luminance.cbrt() - 16.0) / 100.0
    } else {
        903.3 * luminance / 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rendering_modes_and_named_colors_match_kotlin_rules() {
        let tint = Rgba::new(0.2, 0.4, 0.6, 0.8);
        assert_eq!(
            layer_color(None, RenderingMode::Hierarchical, tint, &[], 2, false, 0.25),
            Some(Rgba::new(0.2, 0.4, 0.6, 0.2))
        );
        assert_eq!(
            layer_color(None, RenderingMode::Palette, tint, &[], 9, false, 1.0),
            Some(Rgba::new(0.2, 0.4, 0.6, 0.2))
        );
        let palette = [Rgba::new(1.0, 0.0, 0.0, 1.0), Rgba::new(0.0, 0.0, 1.0, 0.5)];
        assert_eq!(
            layer_color(None, RenderingMode::Palette, tint, &palette, 1, false, 1.0),
            Some(palette[1])
        );
        assert_eq!(
            layer_color(
                Some("#12aBcD"),
                RenderingMode::Multicolor,
                tint,
                &[],
                0,
                false,
                0.5
            ),
            Some(Rgba::new(18.0 / 255.0, 171.0 / 255.0, 205.0 / 255.0, 0.5))
        );
        assert_eq!(
            layer_color(None, RenderingMode::Monochrome, tint, &[], 0, true, 1.0),
            None
        );
        assert_eq!(
            named_color(Some("systemRedColor"), tint),
            Some(Rgba::new(1.0, 59.0 / 255.0, 48.0 / 255.0, 1.0))
        );
        assert_eq!(named_color(Some("erase"), tint), None);
        assert_eq!(
            layer_color(
                Some("hidden"),
                RenderingMode::Monochrome,
                tint,
                &[],
                0,
                false,
                1.0
            ),
            None
        );
    }

    #[test]
    fn automatic_gradient_preserves_alpha_and_order() {
        let black = Rgba::new(0.0, 0.0, 0.0, 0.4);
        let (top, bottom) = automatic_gradient(black);
        assert_eq!(top, Rgba::new(0.2, 0.2, 0.2, 0.4));
        assert_eq!(bottom, black);

        let white = Rgba::new(1.0, 1.0, 1.0, 0.7);
        let (top, bottom) = automatic_gradient(white);
        assert_eq!(top, white);
        assert_eq!(bottom, Rgba::new(0.89, 0.89, 0.89, 0.7));
    }
}
