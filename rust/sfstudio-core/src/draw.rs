//! Distance fields for Apple's rail-guided Draw On/Off reveal masks.

pub const GRID: usize = 112;
pub const LEVELS: usize = 48;
pub const OUTSIDE: u8 = u8::MAX;
const SQRT2: f32 = std::f32::consts::SQRT_2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrawField {
    /// Row-major reveal level per pixel; 255 marks pixels outside the rasterized layer.
    pub levels: Vec<u8>,
    /// Horizontal row runs as `[level, y, x_start, x_end]`, with inclusive x bounds.
    pub runs: Vec<[u8; 4]>,
}

/// Builds reveal levels from the target renderer's binary raster mask.
///
/// Origin and cell describe the top-left and size of each raster pixel in glyph coordinates.
/// The target renderer supplies the mask so its antialiasing stays authoritative.
pub fn build_draw_field(
    mask: &[u8],
    origin: [f32; 2],
    cell: f32,
    symbol_bounds: [f32; 4],
    ties: &[f32],
) -> Result<Option<DrawField>, String> {
    if mask.len() != GRID * GRID || mask.iter().any(|pixel| *pixel > 1) {
        return Err("draw mask must contain exactly 112x112 zero/one pixels".to_owned());
    }
    if !origin.iter().all(|value| value.is_finite())
        || !cell.is_finite()
        || cell <= 0.0
        || !symbol_bounds.iter().all(|value| value.is_finite())
        || symbol_bounds[2] < symbol_bounds[0]
        || symbol_bounds[3] < symbol_bounds[1]
        || ties.len() % 2 != 0
        || !ties.iter().all(|value| value.is_finite())
    {
        return Err("draw field coordinates and ties must be finite and well-formed".to_owned());
    }
    if !mask.contains(&1) {
        return Ok(None);
    }

    let size = GRID * GRID;
    let mut distances = vec![f32::MAX; size];
    let mut components = vec![usize::MAX; size];
    let mut component_maxima = Vec::new();
    let mut heap = MinHeap::default();
    let mut next_tie = 0;
    let tie_count = ties.len() / 2;
    let mut component = 0;

    loop {
        let mut seed = None;
        while seed.is_none() && next_tie < tie_count {
            let tie = [ties[next_tie * 2], ties[next_tie * 2 + 1]];
            seed = nearest_unvisited(mask, &distances, origin, cell, symbol_bounds, tie);
            next_tie += 1;
        }
        if seed.is_none() {
            seed = mask.iter().enumerate().find_map(|(index, pixel)| {
                (*pixel == 1 && distances[index] == f32::MAX).then_some(index)
            });
        }
        let Some(seed) = seed else { break };
        component_maxima.push(flood(
            seed,
            component,
            mask,
            &mut distances,
            &mut components,
            &mut heap,
        ));
        component += 1;
    }

    let mut levels = vec![OUTSIDE; size];
    for index in 0..size {
        if mask[index] == 0 {
            continue;
        }
        let maximum = component_maxima[components[index]];
        let progress = if maximum > 0.0 {
            distances[index] / maximum
        } else {
            0.0
        };
        levels[index] = ((progress * LEVELS as f32) as usize).min(LEVELS - 1) as u8;
    }
    let runs = encode_runs(&levels);
    Ok(Some(DrawField { levels, runs }))
}

fn encode_runs(levels: &[u8]) -> Vec<[u8; 4]> {
    let mut runs = Vec::new();
    for y in 0..GRID {
        let row = y * GRID;
        let mut x = 0;
        while x < GRID {
            let level = levels[row + x];
            if level == OUTSIDE {
                x += 1;
                continue;
            }
            let start = x;
            while x + 1 < GRID && levels[row + x + 1] == level {
                x += 1;
            }
            runs.push([level, y as u8, start as u8, x as u8]);
            x += 1;
        }
    }
    runs
}

fn nearest_unvisited(
    mask: &[u8],
    distances: &[f32],
    origin: [f32; 2],
    cell: f32,
    bounds: [f32; 4],
    tie: [f32; 2],
) -> Option<usize> {
    let gx = bounds[0] + tie[0] * (bounds[2] - bounds[0]);
    let gy = bounds[1] + tie[1] * (bounds[3] - bounds[1]);
    let cx = (((gx - origin[0]) / cell) as i32).clamp(0, GRID as i32 - 1) as usize;
    let cy = (((gy - origin[1]) / cell) as i32).clamp(0, GRID as i32 - 1) as usize;
    let mut best = None;
    let mut best_distance = i32::MAX;
    for y in cy.saturating_sub(10)..=(cy + 10).min(GRID - 1) {
        for x in cx.saturating_sub(10)..=(cx + 10).min(GRID - 1) {
            let index = y * GRID + x;
            if mask[index] == 0 || distances[index] != f32::MAX {
                continue;
            }
            let dx = x as i32 - cx as i32;
            let dy = y as i32 - cy as i32;
            let distance = dx * dx + dy * dy;
            if distance < best_distance {
                best_distance = distance;
                best = Some(index);
            }
        }
    }
    best
}

fn flood(
    seed: usize,
    component: usize,
    mask: &[u8],
    distances: &mut [f32],
    components: &mut [usize],
    heap: &mut MinHeap,
) -> f32 {
    let mut maximum = 0.0f32;
    distances[seed] = 0.0;
    components[seed] = component;
    heap.push(0.0, seed);
    while let Some(index) = heap.pop() {
        let distance = distances[index];
        let x = index % GRID;
        let y = index / GRID;
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0 || ny < 0 || nx >= GRID as i32 || ny >= GRID as i32 {
                    continue;
                }
                let next_x = nx as usize;
                let next_y = ny as usize;
                let next = next_y * GRID + next_x;
                if mask[next] == 0
                    || (dx != 0
                        && dy != 0
                        && (mask[y * GRID + next_x] == 0 || mask[next_y * GRID + x] == 0))
                {
                    continue;
                }
                let next_distance = distance + if dx != 0 && dy != 0 { SQRT2 } else { 1.0 };
                if next_distance < distances[next] {
                    distances[next] = next_distance;
                    components[next] = component;
                    maximum = maximum.max(next_distance);
                    heap.push(next_distance, next);
                }
            }
        }
    }
    maximum
}

#[derive(Default)]
struct MinHeap {
    keys: Vec<f32>,
    values: Vec<usize>,
}

impl MinHeap {
    fn push(&mut self, key: f32, value: usize) {
        let mut index = self.keys.len();
        self.keys.push(key);
        self.values.push(value);
        while index > 0 {
            let parent = (index - 1) / 2;
            if self.keys[parent] <= key {
                break;
            }
            self.keys[index] = self.keys[parent];
            self.values[index] = self.values[parent];
            index = parent;
        }
        self.keys[index] = key;
        self.values[index] = value;
    }

    fn pop(&mut self) -> Option<usize> {
        if self.keys.is_empty() {
            return None;
        }
        let value = self.values[0];
        let key = self.keys.pop()?;
        let last_value = self.values.pop()?;
        if !self.keys.is_empty() {
            let mut index = 0;
            let len = self.keys.len();
            loop {
                let mut child = index * 2 + 1;
                if child >= len {
                    break;
                }
                if child + 1 < len && self.keys[child + 1] < self.keys[child] {
                    child += 1;
                }
                if self.keys[child] >= key {
                    break;
                }
                self.keys[index] = self.keys[child];
                self.values[index] = self.values[child];
                index = child;
            }
            self.keys[index] = key;
            self.values[index] = last_value;
        }
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn follows_tie_along_connected_stroke_and_clamps_last_level() {
        let mut mask = vec![0; GRID * GRID];
        let row = 55 * GRID;
        mask[row + 55..row + 60].fill(1);
        let field = build_draw_field(
            &mask,
            [0.0, 0.0],
            1.0,
            [0.0, 0.0, GRID as f32, GRID as f32],
            &[55.5 / GRID as f32, 55.5 / GRID as f32],
        )
        .unwrap()
        .unwrap();
        assert_eq!(&field.levels[row + 55..row + 60], &[0, 12, 24, 36, 47]);
        assert_eq!(field.levels[row + 54], OUTSIDE);
    }

    #[test]
    fn diagonal_components_do_not_cut_across_empty_corners() {
        let mut mask = vec![0; GRID * GRID];
        mask[0] = 1;
        mask[GRID + 1] = 1;
        let field = build_draw_field(&mask, [0.0, 0.0], 1.0, [0.0, 0.0, 112.0, 112.0], &[])
            .unwrap()
            .unwrap();
        assert_eq!(field.levels[0], 0);
        assert_eq!(field.levels[GRID + 1], 0);
    }

    #[test]
    fn each_disconnected_stroke_starts_at_its_own_rail_tie() {
        let mut mask = vec![0; GRID * GRID];
        let first = 20 * GRID + 10;
        let second = 40 * GRID + 20;
        mask[first..first + 2].fill(1);
        mask[second..second + 2].fill(1);
        let field = build_draw_field(
            &mask,
            [0.0, 0.0],
            1.0,
            [0.0, 0.0, GRID as f32, GRID as f32],
            &[
                10.5 / GRID as f32,
                20.5 / GRID as f32,
                20.5 / GRID as f32,
                40.5 / GRID as f32,
            ],
        )
        .unwrap()
        .unwrap();
        assert_eq!(&field.levels[first..first + 2], &[0, 47]);
        assert_eq!(&field.levels[second..second + 2], &[0, 47]);
    }

    #[test]
    fn rejects_malformed_masks_and_returns_none_for_empty_paths() {
        assert!(build_draw_field(&[0], [0.0, 0.0], 1.0, [0.0, 0.0, 1.0, 1.0], &[]).is_err());
        assert!(build_draw_field(
            &vec![0; GRID * GRID],
            [0.0, 0.0],
            1.0,
            [0.0, 0.0, 1.0, 1.0],
            &[],
        )
        .unwrap()
        .is_none());
    }

    #[test]
    fn groups_adjacent_pixels_with_the_same_level_into_inclusive_runs() {
        let mut levels = vec![OUTSIDE; GRID * GRID];
        levels[2 * GRID + 4..2 * GRID + 7].fill(3);
        levels[2 * GRID + 8] = 4;

        assert_eq!(encode_runs(&levels), vec![[3, 2, 4, 6], [4, 2, 8, 8]]);
    }

    #[test]
    fn draw_levels_match_kotlin_rasterized_symbol_fixtures() {
        const REFERENCE: &str =
            include_str!("../../../sfsymbols/src/jvmTest/resources/sf-draw-fingerprints.txt");
        const LEVEL_CODES: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijkl";

        for row in REFERENCE.lines() {
            let fields: Vec<_> = row.split('|').collect();
            assert_eq!(fields.len(), 11, "bad Kotlin draw-field fixture row");
            let parse = |index: usize| {
                fields[index]
                    .parse::<f32>()
                    .unwrap_or_else(|error| panic!("invalid float in {row}: {error}"))
            };
            let expected: Vec<u8> = fields[10]
                .bytes()
                .map(|level| {
                    if level == b'.' {
                        OUTSIDE
                    } else {
                        LEVEL_CODES
                            .iter()
                            .position(|code| *code == level)
                            .unwrap_or_else(|| panic!("invalid reveal-level code in {row}"))
                            as u8
                    }
                })
                .collect();
            assert_eq!(expected.len(), GRID * GRID, "bad pixel count in {row}");
            let mask: Vec<u8> = expected
                .iter()
                .map(|level| u8::from(*level != OUTSIDE))
                .collect();
            let ties: Vec<f32> = fields[9]
                .split(',')
                .map(|tie| {
                    tie.parse::<f32>()
                        .unwrap_or_else(|error| panic!("invalid tie in {row}: {error}"))
                })
                .collect();
            let field = build_draw_field(
                &mask,
                [parse(2), parse(3)],
                parse(4),
                [parse(5), parse(6), parse(7), parse(8)],
                &ties,
            )
            .unwrap()
            .unwrap_or_else(|| panic!("empty Rust field for {} layer {}", fields[0], fields[1]));
            assert_eq!(
                field.levels, expected,
                "Rust reveal levels differ from Kotlin for {} layer {}",
                fields[0], fields[1]
            );
            let mut reconstructed = vec![OUTSIDE; GRID * GRID];
            for [level, y, start, end] in field.runs {
                for x in start..=end {
                    let index = y as usize * GRID + x as usize;
                    assert_eq!(reconstructed[index], OUTSIDE, "overlapping Rust draw runs");
                    reconstructed[index] = level;
                }
            }
            assert_eq!(
                reconstructed, expected,
                "Rust draw runs differ for {} layer {}",
                fields[0], fields[1]
            );
        }
    }
}
