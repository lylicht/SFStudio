//! Apple-style SF Symbol effect timing and per-layer transforms.

use crate::vector::{Bounds, Geometry, Glyph, LayerNode};
use std::cmp::min;
use std::f32::consts::PI;

const BOUNCE_UP_SEG: f32 = 0.21667;
const BOUNCE_TOTAL: f32 = 0.21667 + 0.21667 + 0.26667;
const WIGGLE_SEG: f32 = 0.167;
const WIGGLE_MOVE: [f32; 5] = [0.06, -0.04, 0.03, -0.025, 0.0];
const WIGGLE_TURN: [f32; 5] = [10.0, -8.0, 6.0, -5.0, 0.0];
const WIGGLE_CONT_MOVE: f32 = 0.025;
const WIGGLE_CONT_TURN: f32 = 5.0;
const VC_SLOT: f32 = 0.3333;
const WIGGLE_LOOP: f32 = 2.0 * (0.0835 + 0.167 + 0.0835);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    WholeSymbol,
    ByLayer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplaceDirection {
    DownUp,
    OffUp,
    UpUp,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Effect {
    Appear {
        scope: Scope,
    },
    Disappear {
        scope: Scope,
    },
    Bounce {
        up: bool,
        scope: Scope,
    },
    Pulse {
        scope: Scope,
    },
    VariableColor {
        cumulative: bool,
        reversing: bool,
        hide_inactive: bool,
    },
    Scale {
        up: bool,
        returns: bool,
    },
    Wiggle {
        direction: Option<WiggleDirection>,
        continuous: bool,
    },
    Rotate {
        clockwise: Option<bool>,
        scope: Scope,
        continuous: bool,
    },
    Breathe {
        pulsing: bool,
    },
    DrawOn {
        scope: Scope,
    },
    DrawOff {
        scope: Scope,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WiggleDirection {
    Left,
    Right,
    Up,
    Down,
    Forward,
    Backward,
    Clockwise,
    CounterClockwise,
    Custom(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayerEffect {
    pub scale: f32,
    pub dx: f32,
    pub dy: f32,
    pub rotation: f32,
    pub alpha: f32,
    pub draw: f32,
    pub pivot_x: f32,
    pub pivot_y: f32,
}

impl Default for LayerEffect {
    fn default() -> Self {
        Self {
            scale: 1.0,
            dx: 0.0,
            dy: 0.0,
            rotation: 0.0,
            alpha: 1.0,
            draw: 1.0,
            pivot_x: f32::NAN,
            pivot_y: f32::NAN,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnimationLayer {
    pub can_pulse: bool,
    pub can_rotate: bool,
    pub auto_variable: bool,
    pub variable_draw: bool,
    pub hierarchical: i32,
    pub erases: bool,
    pub root_u: f32,
    pub stroke_length: f32,
    pub group: usize,
    pub contours: Vec<usize>,
    pub tags: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MagicLayerMorph {
    pub old_layer: usize,
    pub new_layer: usize,
    pub ops: Vec<u8>,
    /// Interpolated screen-coordinate points in the new symbol's frame.
    pub points: Vec<f32>,
}

#[derive(Clone, Copy)]
pub struct MagicReplaceSide<'a> {
    pub symbol: &'a AnimationSymbol,
    pub geometry: &'a Geometry,
    pub weight: f32,
    pub bounds: Bounds,
    pub fit: Bounds,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnimationSymbol {
    pub layers: Vec<AnimationLayer>,
    pub back_to_front: bool,
    pub wiggle: u8,
    pub wiggle_angle: u32,
    pub rotation: u8,
    pub anchors: Vec<f32>,
}

impl AnimationSymbol {
    /// Build the same flat animation layers as the Kotlin `SfSymbol.flatten` path.
    pub fn from_glyph(glyph: &Glyph, geometry: &Geometry) -> Self {
        let lengths = contour_lengths(geometry);
        let mut layers = Vec::new();
        let mut group = 0;
        for node in &glyph.tree {
            let root_group = group;
            group += 1;
            flatten(node, root_group, &mut group, &lengths, &mut layers);
        }
        Self {
            layers,
            back_to_front: glyph.back_to_front,
            wiggle: glyph.wiggle,
            wiggle_angle: glyph.wiggle_angle,
            rotation: glyph.rotation,
            anchors: glyph.anchors.clone(),
        }
    }

    pub fn variable_order(&self) -> Vec<usize> {
        let mut order: Vec<usize> = self
            .layers
            .iter()
            .enumerate()
            .filter_map(|(i, layer)| layer.auto_variable.then_some(i))
            .collect();
        if order.is_empty() {
            order = self
                .layers
                .iter()
                .enumerate()
                .filter_map(|(i, layer)| (layer.hierarchical >= 0 && !layer.erases).then_some(i))
                .collect();
        }
        if !self.back_to_front {
            order.reverse();
        }
        order
    }

    pub fn variable_levels(&self, value: f32) -> (Vec<f32>, Vec<f32>) {
        let mut alpha = vec![1.0; self.layers.len()];
        let mut draw = vec![1.0; self.layers.len()];
        let order = self.variable_order();
        let count = order.len().max(1) as f32;
        let value = value.clamp(0.0, 1.0);
        for (position, index) in order.into_iter().enumerate() {
            let layer = &self.layers[index];
            if layer.variable_draw {
                draw[index] = (value * count - position as f32).clamp(0.0, 1.0);
            } else {
                alpha[index] = if value > position as f32 / count {
                    1.0
                } else {
                    0.3
                };
            }
        }
        (alpha, draw)
    }

    pub fn duration(&self, effect: Effect) -> f32 {
        match effect {
            Effect::Appear { .. } => 0.25,
            Effect::Disappear { .. } => 0.1667,
            Effect::Bounce { .. } => BOUNCE_TOTAL,
            Effect::Pulse { .. } => 2.0,
            Effect::VariableColor { reversing, .. } => {
                (self.variable_count().max(1) as f32 + 1.0)
                    * VC_SLOT
                    * if reversing { 2.0 } else { 1.0 }
            }
            Effect::Scale { returns, .. } => {
                if returns {
                    1.0
                } else {
                    0.25
                }
            }
            Effect::Wiggle { continuous, .. } => {
                if continuous {
                    WIGGLE_LOOP
                } else {
                    5.0 * WIGGLE_SEG
                }
            }
            Effect::Rotate { .. } => 2.0,
            Effect::Breathe { .. } => 3.0,
            Effect::DrawOn { scope } => self.draw_duration(scope, false),
            Effect::DrawOff { scope } => self.draw_duration(scope, true),
        }
    }

    pub fn is_held(effect: Effect) -> bool {
        matches!(
            effect,
            Effect::Appear { .. }
                | Effect::Disappear { .. }
                | Effect::DrawOn { .. }
                | Effect::DrawOff { .. }
        ) || matches!(effect, Effect::Scale { returns: false, .. })
    }

    pub fn continuous_by_default(effect: Effect) -> bool {
        matches!(effect, Effect::Breathe { .. })
            || matches!(
                effect,
                Effect::Rotate {
                    continuous: true,
                    ..
                } | Effect::Wiggle {
                    continuous: true,
                    ..
                }
            )
    }

    pub fn apply(&self, effect: Effect, time: f32, index: usize, rtl: bool) -> Option<LayerEffect> {
        let layer = self.layers.get(index)?;
        let groups = self
            .layers
            .iter()
            .map(|l| l.group)
            .max()
            .map_or(1, |g| g + 1);
        let group = layer.group as f32;
        let mut fx = LayerEffect::default();
        match effect {
            Effect::Appear { scope } => {
                let delay = if scope == Scope::ByLayer {
                    group * layer_step(0.05, 0.15, groups)
                } else {
                    0.0
                };
                let p = EASE.ease((time - delay) / (0.25 - delay));
                fx.alpha = p;
                fx.scale = 0.75 + 0.25 * p;
            }
            Effect::Disappear { scope } => {
                let delay = if scope == Scope::ByLayer {
                    group * layer_step(0.0333, 0.1, groups)
                } else {
                    0.0
                };
                let p = EASE.ease((time - delay) / (0.1667 - delay));
                fx.alpha = 1.0 - p;
                fx.scale = 1.0 - 0.25 * p;
            }
            Effect::Bounce { up, scope } => {
                let delay = if scope == Scope::ByLayer {
                    group * layer_step(0.05, 0.15, groups)
                } else {
                    0.0
                };
                let t = time - delay;
                let first = if up { 1.25 } else { 0.75 };
                let second = if up { 0.9094 } else { 1.1088 };
                let v = if t <= 0.0 {
                    1.0
                } else if t < BOUNCE_UP_SEG {
                    segment(BOUNCE_A, 1.0, first, 0.0, BOUNCE_UP_SEG, t)
                } else if t < 2.0 * BOUNCE_UP_SEG {
                    segment(BOUNCE_B, first, second, BOUNCE_UP_SEG, BOUNCE_UP_SEG, t)
                } else {
                    segment(
                        BOUNCE_C,
                        second,
                        1.0,
                        2.0 * BOUNCE_UP_SEG,
                        0.26667 - delay,
                        t,
                    )
                };
                fx.dy = -(v - 1.0);
            }
            Effect::Pulse { scope } => {
                if !self.layers.iter().any(|l| l.can_pulse) || layer.can_pulse {
                    let delay = if scope == Scope::ByLayer {
                        group * 0.05
                    } else {
                        0.0
                    };
                    let t = time - delay;
                    fx.alpha = if t <= 0.0 {
                        1.0
                    } else if t < 1.0 {
                        segment(EASE, 1.0, 0.3, 0.0, 1.0, t)
                    } else {
                        segment(EASE, 0.3, 1.0, 1.0, 1.0, t)
                    };
                }
            }
            Effect::VariableColor {
                cumulative,
                reversing,
                hide_inactive,
            } => {
                let sequence = self.variable_order();
                if let Some(position) = sequence.iter().position(|&i| i == index) {
                    let total = (sequence.len() as f32 + 1.0) * VC_SLOT;
                    let t = if reversing && time > total {
                        2.0 * total - time
                    } else {
                        time
                    };
                    let inactive = if hide_inactive { 0.0 } else { 0.3 };
                    let up = EASE.ease((t - position as f32 * VC_SLOT) / VC_SLOT);
                    let level = if cumulative {
                        up
                    } else {
                        up * (1.0 - EASE.ease((t - (position + 1) as f32 * VC_SLOT) / VC_SLOT))
                    };
                    fx.alpha = inactive + (1.0 - inactive) * level;
                }
            }
            Effect::Scale { up, returns } => {
                let peak = if up { 1.25 } else { 0.5 };
                let p = if !returns || time < 0.25 {
                    (if up { EASE } else { SCALE_DOWN }).ease(time / 0.25)
                } else if time < 0.75 {
                    1.0
                } else {
                    1.0 - (if up { EASE } else { SCALE_BACK }).ease((time - 0.75) / 0.25)
                };
                fx.scale = 1.0 + (peak - 1.0) * p;
            }
            Effect::Wiggle {
                direction,
                continuous,
            } => {
                let direction = direction.unwrap_or_else(|| self.preferred_wiggle());
                let movement =
                    || self.wiggle_wave(time, continuous, &WIGGLE_MOVE, WIGGLE_CONT_MOVE);
                let turn = || self.wiggle_wave(time, continuous, &WIGGLE_TURN, WIGGLE_CONT_TURN);
                match direction {
                    WiggleDirection::Left => fx.dx = -movement(),
                    WiggleDirection::Right => fx.dx = movement(),
                    WiggleDirection::Forward => fx.dx = if rtl { -movement() } else { movement() },
                    WiggleDirection::Backward => fx.dx = if rtl { movement() } else { -movement() },
                    WiggleDirection::Up => fx.dy = -movement(),
                    WiggleDirection::Down => fx.dy = movement(),
                    WiggleDirection::Clockwise => fx.rotation = turn(),
                    WiggleDirection::CounterClockwise => fx.rotation = -turn(),
                    WiggleDirection::Custom(degrees) => {
                        let angle = degrees * PI / 180.0;
                        let amount = movement();
                        fx.dx = angle.cos() * amount;
                        fx.dy = angle.sin() * amount;
                    }
                }
            }
            Effect::Rotate {
                clockwise,
                scope,
                continuous,
            } => {
                if !self.layers.iter().any(|l| l.can_rotate) || layer.can_rotate {
                    let clockwise = clockwise.unwrap_or(self.rotation != 2);
                    let delay = if scope == Scope::ByLayer {
                        group * layer_step(0.05, 0.2, groups + 1)
                    } else {
                        0.0
                    };
                    let angle = if continuous {
                        360.0 * ((time / 2.0) % 1.0)
                    } else {
                        segment(ROTATE_ONCE, 0.0, 360.0, delay, 2.0 - delay, time)
                    };
                    fx.rotation = if clockwise { angle } else { -angle };
                    if self.anchors.len() >= 2 {
                        let count = self.anchors.len() / 2;
                        fx.pivot_x =
                            (0..count).map(|i| self.anchors[2 * i]).sum::<f32>() / count as f32;
                        fx.pivot_y = 1.0
                            - (0..count).map(|i| self.anchors[2 * i + 1]).sum::<f32>()
                                / count as f32;
                    }
                }
            }
            Effect::Breathe { pulsing } => {
                let t = time % 3.0;
                fx.scale = if t < 1.5 {
                    segment(BREATHE_SCALE, 1.0, 1.2, 0.0, 1.5, t)
                } else {
                    segment(BREATHE_SCALE, 1.2, 1.0, 1.5, 1.5, t)
                };
                if pulsing {
                    fx.alpha = if t < 1.5 {
                        segment(EASE, 1.0, 0.3, 0.0, 1.5, t)
                    } else {
                        segment(EASE, 0.3, 1.0, 1.5, 1.5, t)
                    };
                }
            }
            Effect::DrawOn { scope } => {
                let delay = if scope == Scope::ByLayer {
                    self.draw_start(layer.group, false)
                } else {
                    0.0
                };
                if layer.root_u.is_nan() {
                    let p = EASE.ease((time - delay) / 0.25);
                    fx.alpha = p;
                    fx.scale = 0.75 + 0.25 * p;
                } else {
                    fx.draw = DRAW_ON.ease((time - delay) / draw_seconds(layer.stroke_length));
                }
            }
            Effect::DrawOff { scope } => {
                let delay = if scope == Scope::ByLayer {
                    self.draw_start(layer.group, true)
                } else {
                    0.0
                };
                if layer.root_u.is_nan() {
                    let p = EASE.ease((time - delay) / 0.1667);
                    fx.alpha = 1.0 - p;
                    fx.scale = 1.0 - 0.25 * p;
                } else {
                    fx.draw =
                        1.0 - DRAW_OFF.ease((time - delay) / draw_seconds(layer.stroke_length));
                }
            }
        }
        Some(fx)
    }

    fn variable_count(&self) -> usize {
        self.variable_order().len()
    }

    fn preferred_wiggle(&self) -> WiggleDirection {
        match self.wiggle {
            1 => WiggleDirection::Right,
            2 => WiggleDirection::Left,
            3 => WiggleDirection::Up,
            4 => WiggleDirection::Down,
            5 => WiggleDirection::Clockwise,
            6 => WiggleDirection::CounterClockwise,
            7 => WiggleDirection::Backward,
            8 => WiggleDirection::Forward,
            9 => WiggleDirection::Custom(self.wiggle_angle as f32),
            _ => WiggleDirection::Right,
        }
    }

    fn wiggle_wave(&self, time: f32, continuous: bool, values: &[f32; 5], amplitude: f32) -> f32 {
        if continuous {
            return continuous_wiggle(time) * amplitude;
        }
        if time >= 5.0 * WIGGLE_SEG {
            return 0.0;
        }
        let k = min(4, (time / WIGGLE_SEG) as i32).max(0) as usize;
        let previous = if k == 0 { 0.0 } else { values[k - 1] };
        let curve = if k == 0 { WIGGLE_1 } else { EASE };
        previous + (values[k] - previous) * curve.ease((time - k as f32 * WIGGLE_SEG) / WIGGLE_SEG)
    }

    fn draw_start(&self, group: usize, off: bool) -> f32 {
        (0..group).map(|g| self.group_draw_seconds(g, off)).sum()
    }

    fn group_draw_seconds(&self, group: usize, off: bool) -> f32 {
        self.layers
            .iter()
            .filter(|layer| layer.group == group)
            .map(|layer| {
                if layer.root_u.is_nan() {
                    if off {
                        0.1667
                    } else {
                        0.25
                    }
                } else {
                    draw_seconds(layer.stroke_length)
                }
            })
            .fold(0.0, f32::max)
    }

    fn draw_duration(&self, scope: Scope, off: bool) -> f32 {
        if scope == Scope::ByLayer {
            let count = self
                .layers
                .iter()
                .map(|l| l.group)
                .max()
                .map_or(0, |g| g + 1);
            self.draw_start(count, off)
        } else {
            (self
                .layers
                .iter()
                .filter(|layer| !layer.root_u.is_nan())
                .map(|layer| draw_seconds(layer.stroke_length))
                .fold(0.0, f32::max))
            .max(0.25)
        }
    }
}

pub fn draw_seconds(length: f32) -> f32 {
    let x = length / 581.79;
    let (mut lo, mut hi) = (0.0, 4.0);
    for _ in 0..30 {
        let u = (lo + hi) / 2.0;
        if (0.7 * u - 0.27) * u * u + 0.57 * u < x {
            lo = u;
        } else {
            hi = u;
        }
    }
    let u = (lo + hi) / 2.0;
    (52.0 * (((-0.77 * u + 0.96) * u + 0.81) * u) + 8.0) / 60.0
}

/// Match Apple-tagged, structurally identical layers and morph their outlines.
/// Result pairs are `(new layer index, old layer index)`, matching the Compose renderer.
pub fn magic_replace(
    old: MagicReplaceSide<'_>,
    new: MagicReplaceSide<'_>,
    progress: f32,
) -> Result<Vec<MagicLayerMorph>, String> {
    if !old.weight.is_finite() || !new.weight.is_finite() || !progress.is_finite() {
        return Err("morph weights and progress must be finite".to_owned());
    }
    let old_contours = contour_layout(old.geometry);
    let new_contours = contour_layout(new.geometry);
    let mut used_old = vec![false; old.symbol.layers.len()];
    let mut matches = Vec::new();
    for (new_index, new_layer) in new.symbol.layers.iter().enumerate() {
        if new_layer.erases {
            continue;
        }
        for (old_index, old_layer) in old.symbol.layers.iter().enumerate() {
            if used_old[old_index]
                || old_layer.erases
                || !old_layer
                    .tags
                    .iter()
                    .any(|tag| new_layer.tags.contains(tag))
                || !same_layer_signature(old_layer, new_layer, &old_contours, &new_contours)
            {
                continue;
            }
            used_old[old_index] = true;
            matches.push((new_index, old_index));
            break;
        }
    }

    let old_points = crate::vector::interpolate_points(old.geometry, old.weight);
    let new_points = crate::vector::interpolate_points(new.geometry, new.weight);
    let old_fit_size = (old.fit.right - old.fit.left).max(old.fit.bottom - old.fit.top);
    let new_fit_size = (new.fit.right - new.fit.left).max(new.fit.bottom - new.fit.top);
    if old_fit_size <= 0.0
        || new_fit_size <= 0.0
        || !old_fit_size.is_finite()
        || !new_fit_size.is_finite()
    {
        return Err("morph fit bounds must have positive dimensions".to_owned());
    }
    let ratio = old_fit_size / new_fit_size;
    let old_center = (
        (old.bounds.left + old.bounds.right) / 2.0,
        (old.bounds.top + old.bounds.bottom) / 2.0,
    );
    let new_center = (
        (new.bounds.left + new.bounds.right) / 2.0,
        (new.bounds.top + new.bounds.bottom) / 2.0,
    );
    let mut output = Vec::with_capacity(matches.len());
    for (new_index, old_index) in matches {
        let old_layer = &old.symbol.layers[old_index];
        let new_layer = &new.symbol.layers[new_index];
        let mut ops = Vec::new();
        let mut points = Vec::new();
        for (old_contour, new_contour) in old_layer.contours.iter().zip(&new_layer.contours) {
            let (old_ops, mut old_offset) = old_contours
                .get(*old_contour)
                .ok_or_else(|| "old layer references a missing contour".to_owned())?;
            let (new_ops, mut new_offset) = new_contours
                .get(*new_contour)
                .ok_or_else(|| "new layer references a missing contour".to_owned())?;
            if old_ops != new_ops {
                return Err("matched contour structures differ".to_owned());
            }
            ops.extend_from_slice(new_ops);
            for &op in new_ops {
                let coordinate_count = match op {
                    0 | 1 => 2,
                    2 => 6,
                    3 => 0,
                    _ => return Err("invalid path operation".to_owned()),
                };
                for _ in 0..(coordinate_count / 2) {
                    let x = (old_points[old_offset] - old_center.0) / ratio + new_center.0;
                    let y = (old_points[old_offset + 1] - old_center.1) / ratio + new_center.1;
                    points.push(x + (new_points[new_offset] - x) * progress);
                    points.push(y + (new_points[new_offset + 1] - y) * progress);
                    old_offset += 2;
                    new_offset += 2;
                }
            }
        }
        output.push(MagicLayerMorph {
            old_layer: old_index,
            new_layer: new_index,
            ops,
            points,
        });
    }
    Ok(output)
}

fn contour_layout(geometry: &Geometry) -> Vec<(Vec<u8>, usize)> {
    let mut contours = Vec::new();
    let mut start = 0;
    let mut offset = 0;
    let mut ops = Vec::new();
    for &op in &geometry.ops {
        ops.push(op);
        offset += match op {
            0 | 1 => 2,
            2 => 6,
            _ => 0,
        };
        if op == 3 {
            contours.push((std::mem::take(&mut ops), start));
            start = offset;
        }
    }
    if !ops.is_empty() {
        contours.push((ops, start));
    }
    contours
}

fn same_layer_signature(
    old: &AnimationLayer,
    new: &AnimationLayer,
    old_contours: &[(Vec<u8>, usize)],
    new_contours: &[(Vec<u8>, usize)],
) -> bool {
    old.contours.len() == new.contours.len()
        && old
            .contours
            .iter()
            .zip(&new.contours)
            .all(|(&old_id, &new_id)| {
                old_contours.get(old_id).map(|(ops, _)| ops)
                    == new_contours.get(new_id).map(|(ops, _)| ops)
            })
}

pub fn replace_seconds(direction: ReplaceDirection) -> f32 {
    if direction == ReplaceDirection::DownUp {
        0.5
    } else {
        0.25
    }
}

pub fn replace_out(time: f32, direction: ReplaceDirection) -> LayerEffect {
    let mut fx = LayerEffect::default();
    match direction {
        ReplaceDirection::OffUp => fx.alpha = 0.0,
        ReplaceDirection::UpUp => {
            fx.scale = 1.0 + 0.25 * REPLACE_UP.ease(time / 0.25);
            fx.alpha = 1.0 - EASE.ease(time / 0.25);
        }
        ReplaceDirection::DownUp => {
            fx.scale = 1.0 - 0.5 * REPLACE_DOWN.ease(time / 0.25);
            fx.alpha = if time >= 0.25 {
                0.0
            } else {
                1.0 - REPLACE_FADE.ease((time - 0.0833) / 0.1667)
            };
        }
    }
    fx
}

pub fn replace_in(time: f32, direction: ReplaceDirection) -> LayerEffect {
    let start = if direction == ReplaceDirection::DownUp {
        0.25
    } else {
        0.0
    };
    let local = time - start;
    LayerEffect {
        scale: 0.5 + 0.5 * REPLACE_GROW.ease(local / 0.25),
        alpha: if direction == ReplaceDirection::DownUp {
            REPLACE_FADE.ease(local / 0.1667)
        } else {
            EASE.ease(local / 0.25)
        },
        ..LayerEffect::default()
    }
}

#[derive(Clone, Copy)]
struct Curve(f32, f32, f32, f32);

impl Curve {
    fn ease(self, progress: f32) -> f32 {
        let x = progress.clamp(0.0, 1.0);
        if x == 0.0 || x == 1.0 {
            return x;
        }
        let (mut lo, mut hi, mut t) = (0.0, 1.0, x);
        for _ in 0..24 {
            let u = 1.0 - t;
            let bx = 3.0 * u * u * t * self.0 + 3.0 * u * t * t * self.2 + t * t * t;
            if bx < x {
                lo = t;
            } else {
                hi = t;
            }
            t = (lo + hi) / 2.0;
        }
        let u = 1.0 - t;
        3.0 * u * u * t * self.1 + 3.0 * u * t * t * self.3 + t * t * t
    }
}

const EASE: Curve = Curve(0.33, 0.0, 0.67, 1.0);
const BOUNCE_A: Curve = Curve(0.55, 0.0, 0.67, 1.0);
const BOUNCE_B: Curve = Curve(0.45, 0.0, 0.67, 1.0);
const BOUNCE_C: Curve = Curve(0.25, 0.0, 0.55, 1.0);
const WIGGLE_1: Curve = Curve(0.55, 0.0, 0.67, 1.0);
const BREATHE_SCALE: Curve = Curve(0.4, 0.0, 0.6, 1.0);
const SCALE_DOWN: Curve = Curve(0.75, 0.0, 0.65, 1.0);
const SCALE_BACK: Curve = Curve(0.35, 0.0, 0.25, 1.0);
const ROTATE_ONCE: Curve = Curve(0.25, 0.25, 0.25, 1.0);
const DRAW_ON: Curve = Curve(0.2, 0.0, 0.25, 1.0);
const DRAW_OFF: Curve = Curve(0.4, 0.0, 0.8, 1.0);
const REPLACE_DOWN: Curve = Curve(0.75, 0.0, 0.8, 1.0);
const REPLACE_GROW: Curve = Curve(0.2, 0.0, 0.25, 1.0);
const REPLACE_FADE: Curve = Curve(0.17, 0.17, 0.67, 1.0);
const REPLACE_UP: Curve = Curve(0.33, 0.0, 0.83, 0.83);

fn segment(curve: Curve, from: f32, to: f32, start: f32, duration: f32, time: f32) -> f32 {
    from + (to - from) * curve.ease((time - start) / duration)
}

fn continuous_wiggle(time: f32) -> f32 {
    let t = time % WIGGLE_LOOP;
    let half = WIGGLE_LOOP / 2.0;
    let local = t % half;
    let up_curve = if t < half { WIGGLE_UP } else { WIGGLE_UP2 };
    if local < 0.0835 {
        segment(up_curve, 0.0, 1.0, 0.0, 0.0835, local)
    } else if local < 0.2505 {
        segment(WIGGLE_UP, 1.0, -1.0, 0.0835, 0.167, local)
    } else {
        segment(WIGGLE_BACK, -1.0, 0.0, 0.2505, 0.0835, local)
    }
}

const WIGGLE_UP: Curve = Curve(0.33, 0.0, 0.67, 1.0);
const WIGGLE_BACK: Curve = Curve(0.33, 0.0, 0.88, 0.83);
const WIGGLE_UP2: Curve = Curve(0.12, 0.17, 0.67, 1.0);

fn layer_step(max_step: f32, budget: f32, groups: usize) -> f32 {
    max_step.min(budget / groups as f32)
}

fn contour_lengths(geometry: &Geometry) -> Vec<f32> {
    let points = &geometry.regular;
    let mut lengths = Vec::new();
    let (mut offset, mut x, mut y, mut accumulated) = (0, 0.0, 0.0, 0.0);
    for &op in &geometry.ops {
        match op {
            0 => {
                x = points[offset];
                y = points[offset + 1];
                offset += 2;
            }
            1 => {
                let nx = points[offset];
                let ny = points[offset + 1];
                accumulated += (nx - x).hypot(ny - y);
                x = nx;
                y = ny;
                offset += 2;
            }
            2 => {
                let (mut px, mut py) = (x, y);
                for step in 1..=8 {
                    let t = step as f32 / 8.0;
                    let u = 1.0 - t;
                    let nx = u * u * u * x
                        + 3.0 * u * u * t * points[offset]
                        + 3.0 * u * t * t * points[offset + 2]
                        + t * t * t * points[offset + 4];
                    let ny = u * u * u * y
                        + 3.0 * u * u * t * points[offset + 1]
                        + 3.0 * u * t * t * points[offset + 3]
                        + t * t * t * points[offset + 5];
                    accumulated += (nx - px).hypot(ny - py);
                    px = nx;
                    py = ny;
                }
                x = points[offset + 4];
                y = points[offset + 5];
                offset += 6;
            }
            _ => {
                lengths.push(accumulated);
                accumulated = 0.0;
            }
        }
    }
    lengths
}

fn flatten(
    node: &LayerNode,
    assigned_group: usize,
    next_group: &mut usize,
    lengths: &[f32],
    out: &mut Vec<AnimationLayer>,
) {
    if node.children.is_empty() {
        let length = node
            .contours
            .iter()
            .filter_map(|&c| lengths.get(c))
            .sum::<f32>()
            / 2.0
            / 2048.0
            * 100.0;
        out.push(AnimationLayer {
            can_pulse: node.can_pulse,
            can_rotate: node.can_rotate,
            auto_variable: node.auto_variable || node.variable_draw,
            variable_draw: node.variable_draw,
            hierarchical: node.hierarchical,
            erases: node.mono_erase,
            root_u: node.ties.first().copied().unwrap_or(f32::NAN),
            stroke_length: length,
            group: assigned_group,
            contours: node.contours.clone(),
            tags: node.tags.clone(),
        });
        return;
    }
    let together = node.animate_together.then(|| {
        let group = *next_group;
        *next_group += 1;
        group
    });
    for child in &node.children {
        let group = if let Some(group) = together {
            group
        } else {
            let group = *next_group;
            *next_group += 1;
            group
        };
        flatten(child, group, next_group, lengths, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apple_animation_timing_and_replacements() {
        assert!((draw_seconds(0.0) - 8.0 / 60.0).abs() < 0.002);
        assert!((draw_seconds(581.79) - 1.0).abs() < 0.01);
        let down = replace_out(0.25, ReplaceDirection::DownUp);
        assert!((down.scale - 0.5).abs() < 0.001);
        assert_eq!(down.alpha, 0.0);
        assert!((replace_in(0.5, ReplaceDirection::DownUp).scale - 1.0).abs() < 0.001);
        assert_eq!(replace_seconds(ReplaceDirection::DownUp), 0.5);
    }

    #[test]
    fn effect_frames_match_the_kotlin_reference_fixture() {
        let catalog = crate::vector::vector_catalog().expect("embedded SF Symbols decode");
        for row in
            include_str!("../../../sfsymbols/src/jvmTest/resources/sf-animation-fingerprints.txt")
                .lines()
        {
            let fields: Vec<_> = row.split('|').collect();
            assert_eq!(fields.len(), 18, "bad Kotlin animation fixture row: {row}");
            let info = catalog.find(fields[0]).expect("fixture symbol exists");
            let glyph = catalog
                .glyph_for(info)
                .expect("glyph decodes")
                .expect("fixture glyph exists");
            let geometry = catalog
                .geometry_at_scale(info, &glyph, crate::vector::OpticalScale::Medium)
                .expect("medium geometry decodes");
            let symbol = AnimationSymbol::from_glyph(&glyph, &geometry);
            let time = fields[2].parse::<f32>().unwrap();
            let rtl = fields[3].parse::<bool>().unwrap();
            let index = fields[4].parse::<usize>().unwrap();
            let effect = match fields[1] {
                "appear_layer" => Effect::Appear {
                    scope: Scope::ByLayer,
                },
                "disappear" => Effect::Disappear {
                    scope: Scope::WholeSymbol,
                },
                "bounce_down_layer" => Effect::Bounce {
                    up: false,
                    scope: Scope::ByLayer,
                },
                "pulse_layer" => Effect::Pulse {
                    scope: Scope::ByLayer,
                },
                "variable_iterative_reverse" => Effect::VariableColor {
                    cumulative: false,
                    reversing: true,
                    hide_inactive: false,
                },
                "variable_cumulative_hide" => Effect::VariableColor {
                    cumulative: true,
                    reversing: false,
                    hide_inactive: true,
                },
                "scale_up_return" => Effect::Scale {
                    up: true,
                    returns: true,
                },
                "scale_down" => Effect::Scale {
                    up: false,
                    returns: false,
                },
                "wiggle_left" => Effect::Wiggle {
                    direction: Some(WiggleDirection::Left),
                    continuous: false,
                },
                "wiggle_continuous_rtl" => Effect::Wiggle {
                    direction: Some(WiggleDirection::Forward),
                    continuous: true,
                },
                "rotate_clockwise_layer" => Effect::Rotate {
                    clockwise: Some(true),
                    scope: Scope::ByLayer,
                    continuous: false,
                },
                "rotate_preferred" => Effect::Rotate {
                    clockwise: None,
                    scope: Scope::WholeSymbol,
                    continuous: false,
                },
                "breathe" => Effect::Breathe { pulsing: false },
                "breathe_pulsing" => Effect::Breathe { pulsing: true },
                "draw_on_layer" => Effect::DrawOn {
                    scope: Scope::ByLayer,
                },
                "draw_off" => Effect::DrawOff {
                    scope: Scope::WholeSymbol,
                },
                id => panic!("unknown Kotlin animation fixture effect: {id}"),
            };
            assert_close(symbol.duration(effect), fields[5].parse().unwrap(), row);
            assert_eq!(
                AnimationSymbol::is_held(effect),
                fields[6].parse::<bool>().unwrap(),
                "{row}"
            );
            assert_eq!(
                AnimationSymbol::continuous_by_default(effect),
                fields[7].parse::<bool>().unwrap(),
                "{row}"
            );
            let actual = symbol
                .apply(effect, time, index, rtl)
                .expect("fixture layer exists");
            let values = [
                actual.scale,
                actual.dx,
                actual.dy,
                actual.rotation,
                actual.alpha,
                actual.draw,
                actual.pivot_x,
                actual.pivot_y,
            ];
            for (actual, expected) in values.into_iter().zip(&fields[8..16]) {
                let expected: f32 = expected.parse().unwrap();
                if expected.is_nan() {
                    assert!(actual.is_nan(), "{row}");
                } else {
                    assert_close(actual, expected, row);
                }
            }
            let (alpha, draw) = symbol.variable_levels(0.63);
            assert_close(alpha[index], fields[16].parse().unwrap(), row);
            assert_close(draw[index], fields[17].parse().unwrap(), row);
        }
    }

    #[test]
    fn magic_replace_morphs_tagged_layers_in_the_new_symbol_frame() {
        let catalog = crate::vector::vector_catalog().unwrap();
        let info = catalog.find("wifi").unwrap();
        let glyph = catalog.glyph_for(info).unwrap().unwrap();
        let geometry = catalog
            .geometry_at_scale(info, &glyph, crate::vector::OpticalScale::Medium)
            .unwrap();
        let symbol = AnimationSymbol::from_glyph(&glyph, &geometry);
        let bounds = crate::vector::visible_bounds(&glyph, &geometry, 400.0).unwrap();
        let side = MagicReplaceSide {
            symbol: &symbol,
            geometry: &geometry,
            weight: 400.0,
            bounds,
            fit: bounds,
        };
        let morphs = magic_replace(side, side, 0.5).unwrap();
        assert!(!morphs.is_empty());
        assert!(morphs
            .iter()
            .all(|m| !m.ops.is_empty() && m.points.iter().all(|p| p.is_finite())));
        let source_points = crate::vector::interpolate_points(&geometry, 400.0);
        let contours = contour_layout(&geometry);
        for morph in morphs {
            assert_eq!(morph.old_layer, morph.new_layer);
            let mut expected_ops = Vec::new();
            let mut expected_points = Vec::new();
            for &contour in &symbol.layers[morph.new_layer].contours {
                let (ops, start) = &contours[contour];
                expected_ops.extend_from_slice(ops);
                let count = ops
                    .iter()
                    .map(|op| match op {
                        0 | 1 => 2,
                        2 => 6,
                        _ => 0,
                    })
                    .sum::<usize>();
                expected_points.extend_from_slice(&source_points[*start..*start + count]);
            }
            assert_eq!(morph.ops, expected_ops);
            assert_eq!(morph.points.len(), expected_points.len());
            assert!(morph
                .points
                .iter()
                .zip(expected_points)
                .all(|(a, b)| (a - b).abs() < 0.0001));
        }
    }

    fn assert_close(actual: f32, expected: f32, context: &str) {
        assert!(
            (actual - expected).abs() <= 0.00005,
            "{context}: {actual} != {expected}"
        );
    }
}
