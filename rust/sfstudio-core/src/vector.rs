//! Decoder for the Kotlin library's packed SF Symbol vector catalog.
//!
//! `build.rs` imports the authoritative generated Kotlin data into compact
//! build artifacts; no symbol data is checked in a second time.

use flate2::read::DeflateDecoder;
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::Read;
use std::ops::Range;
use std::sync::{Arc, Mutex, OnceLock};

const BLOCK_CACHE_CAPACITY: usize = 6;
const MAX_DECOMPRESSED_BLOCK: usize = 16 * 1024 * 1024;
const MAX_VECTOR_OPERATIONS: usize = 4 * 1024 * 1024;
const MAX_LAYER_NODES: usize = 100_000;
const MAX_LAYER_DEPTH: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpticalScale {
    Small,
    Medium,
    Large,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Geometry {
    /// 0 = move, 1 = line, 2 = cubic, 3 = close.
    pub ops: Vec<u8>,
    /// Point pairs for the regular, light, and heavy masters, in font units.
    pub regular: Vec<f32>,
    pub light: Vec<f32>,
    pub heavy: Vec<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Bounds {
    fn empty() -> Self {
        Self {
            left: f32::MAX,
            top: f32::MAX,
            right: -f32::MAX,
            bottom: -f32::MAX,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayerNode {
    pub is_group: bool,
    pub can_pulse: bool,
    pub can_rotate: bool,
    pub clear_behind: bool,
    pub animate_together: bool,
    pub variable_draw: bool,
    pub auto_variable: bool,
    pub mono_erase: bool,
    pub shape_erase: bool,
    /// -1 = omitted from hierarchical mode, 0..2 = primary/secondary/tertiary.
    pub hierarchical: i32,
    /// Index into the catalog color table, or `None` for no explicit color.
    pub color_id: Option<usize>,
    pub opacity: f32,
    /// Tie midpoints as normalized (x, y) pairs; y is measured from the top.
    pub ties: Vec<f32>,
    pub contours: Vec<usize>,
    pub tags: Vec<usize>,
    pub children: Vec<LayerNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Glyph {
    pub geometry: Geometry,
    pub layers_exact: bool,
    pub back_to_front: bool,
    pub gapless: bool,
    pub default_variable_rendering: u8,
    pub wiggle: u8,
    pub wiggle_angle: u32,
    pub rotation: u8,
    /// Anchor pairs, normalized to the glyph's bounds.
    pub anchors: Vec<f32>,
    pub tree: Vec<LayerNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlyphReference {
    pub block: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolInfo {
    pub name: String,
    pub block: usize,
    pub offset: usize,
    pub categories: Vec<usize>,
    pub since: String,
    pub flags: u8,
    pub keywords: Vec<String>,
    pub hierarchical_since: String,
    pub multicolor_since: String,
    pub small: Option<GlyphReference>,
    pub large: Option<GlyphReference>,
}

impl SymbolInfo {
    pub fn is_restricted(&self) -> bool {
        self.flags & 1 != 0
    }

    pub fn mirrors_in_rtl(&self) -> bool {
        self.flags & 2 != 0
    }

    pub fn supports_hierarchical(&self) -> bool {
        self.flags & 8 != 0
    }

    pub fn supports_multicolor(&self) -> bool {
        self.flags & 16 != 0
    }

    pub fn in_legacy_library(&self) -> bool {
        self.flags & 32 != 0
    }
}

struct BlockCache {
    entries: HashMap<usize, Arc<[u8]>>,
    insertion_order: VecDeque<usize>,
}

pub struct VectorCatalog {
    symbols: Vec<SymbolInfo>,
    by_name: HashMap<String, usize>,
    by_legacy_name: HashMap<String, usize>,
    aliases: HashMap<String, String>,
    tags: Vec<String>,
    colors: Vec<String>,
    compressed_blocks: &'static [u8],
    block_ranges: Vec<Range<usize>>,
    blocks: Mutex<BlockCache>,
}

impl VectorCatalog {
    fn from_embedded() -> Result<Self, String> {
        let compressed_blocks = include_bytes!(concat!(env!("OUT_DIR"), "/sf_vector_blocks.bin"));
        let catalog_source = include_str!(concat!(env!("OUT_DIR"), "/sf_vector_catalog.txt"));
        let alias_source = include_str!(concat!(env!("OUT_DIR"), "/sf_vector_aliases.txt"));
        let tag_source = include_str!(concat!(env!("OUT_DIR"), "/sf_vector_tags.txt"));
        let color_source = include_str!(concat!(env!("OUT_DIR"), "/sf_vector_colors.txt"));

        let mut block_ranges = Vec::new();
        let mut cursor = 0usize;
        while cursor < compressed_blocks.len() {
            let header_end = cursor
                .checked_add(4)
                .ok_or_else(|| "compressed block offset overflow".to_owned())?;
            let header = compressed_blocks
                .get(cursor..header_end)
                .ok_or_else(|| "truncated compressed block header".to_owned())?;
            let length = u32::from_le_bytes(header.try_into().expect("four-byte header")) as usize;
            let start = header_end;
            let end = start
                .checked_add(length)
                .ok_or_else(|| "compressed block length overflow".to_owned())?;
            if end > compressed_blocks.len() {
                return Err("truncated compressed block payload".to_owned());
            }
            block_ranges.push(start..end);
            cursor = end;
        }

        let mut symbols = Vec::new();
        let mut by_name = HashMap::new();
        for (line_number, line) in catalog_source.lines().enumerate() {
            if line.is_empty() {
                continue;
            }
            let fields: Vec<_> = line.split('|').collect();
            if fields.len() != 10 {
                return Err(format!(
                    "vector catalog row {} has {} columns, expected 10",
                    line_number + 1,
                    fields.len()
                ));
            }
            let (block, offset) = parse_reference(fields[1])?
                .ok_or_else(|| format!("symbol {} has no medium geometry", fields[0]))?;
            let categories = parse_indices(fields[2])?;
            let flags = fields[4]
                .parse::<u8>()
                .map_err(|error| format!("invalid flags for {}: {error}", fields[0]))?;
            let small =
                parse_reference(fields[8])?.map(|(block, offset)| GlyphReference { block, offset });
            let large =
                parse_reference(fields[9])?.map(|(block, offset)| GlyphReference { block, offset });
            let symbol = SymbolInfo {
                name: fields[0].to_owned(),
                block,
                offset,
                categories,
                since: fields[3].to_owned(),
                flags,
                keywords: parse_strings(fields[5]),
                hierarchical_since: fields[6].to_owned(),
                multicolor_since: fields[7].to_owned(),
                small,
                large,
            };
            if symbol.block >= block_ranges.len() {
                return Err(format!("symbol {} references a missing block", symbol.name));
            }
            let index = symbols.len();
            if by_name.insert(symbol.name.clone(), index).is_some() {
                return Err(format!("duplicate vector symbol {}", symbol.name));
            }
            symbols.push(symbol);
        }
        if symbols.is_empty() {
            return Err("vector catalog is empty".to_owned());
        }
        let by_legacy_name = symbols
            .iter()
            .enumerate()
            .filter(|(_, symbol)| symbol.in_legacy_library())
            .map(|(index, symbol)| (legacy_name(&symbol.name), index))
            .collect();

        let mut aliases = HashMap::new();
        for line in alias_source.lines().filter(|line| !line.is_empty()) {
            let (alias, canonical) = line
                .split_once('|')
                .ok_or_else(|| format!("malformed vector alias row: {line}"))?;
            aliases.insert(alias.to_owned(), canonical.to_owned());
        }

        Ok(Self {
            symbols,
            by_name,
            by_legacy_name,
            aliases,
            tags: tag_source.lines().map(str::to_owned).collect(),
            colors: color_source.lines().map(str::to_owned).collect(),
            compressed_blocks,
            block_ranges,
            blocks: Mutex::new(BlockCache {
                entries: HashMap::new(),
                insertion_order: VecDeque::new(),
            }),
        })
    }

    pub fn all(&self) -> &[SymbolInfo] {
        &self.symbols
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn colors(&self) -> &[String] {
        &self.colors
    }

    /// Looks up an Apple name, generated alias, or legacy `SF...` property name.
    pub fn find(&self, name: &str) -> Option<&SymbolInfo> {
        let index = self
            .by_name
            .get(name)
            .copied()
            .or_else(|| {
                self.aliases
                    .get(name)
                    .and_then(|canonical| self.by_name.get(canonical).copied())
            })
            .or_else(|| self.by_legacy_name.get(name).copied())?;
        self.symbols.get(index)
    }

    pub fn glyph(&self, name: &str) -> Result<Option<Glyph>, String> {
        let Some(symbol) = self.find(name) else {
            return Ok(None);
        };
        self.glyph_for(symbol)
    }

    pub fn glyph_for(&self, symbol: &SymbolInfo) -> Result<Option<Glyph>, String> {
        let block = self.block(symbol.block)?;
        let mut reader = Reader::new(&block, symbol.offset)?;
        read_glyph(&mut reader).map(Some)
    }

    pub fn geometry_at_scale(
        &self,
        symbol: &SymbolInfo,
        medium: &Glyph,
        scale: OpticalScale,
    ) -> Result<Geometry, String> {
        let reference = match scale {
            OpticalScale::Small => symbol.small.as_ref(),
            OpticalScale::Medium => None,
            OpticalScale::Large => symbol.large.as_ref(),
        };
        let Some(reference) = reference else {
            return Ok(medium.geometry.clone());
        };
        let block = self.block(reference.block)?;
        let mut reader = Reader::new(&block, reference.offset)?;
        read_scale_geometry(&mut reader, &medium.geometry)
    }

    fn block(&self, index: usize) -> Result<Arc<[u8]>, String> {
        let range = self
            .block_ranges
            .get(index)
            .ok_or_else(|| format!("unknown vector block {index}"))?;
        let mut cache = self
            .blocks
            .lock()
            .map_err(|_| "vector block cache is poisoned".to_owned())?;
        if let Some(block) = cache.entries.get(&index) {
            return Ok(Arc::clone(block));
        }

        let compressed = self
            .compressed_blocks
            .get(range.clone())
            .ok_or_else(|| format!("invalid compressed range for block {index}"))?;
        let decoder = DeflateDecoder::new(compressed);
        let mut output = Vec::new();
        decoder
            .take((MAX_DECOMPRESSED_BLOCK + 1) as u64)
            .read_to_end(&mut output)
            .map_err(|error| format!("could not inflate vector block {index}: {error}"))?;
        if output.len() > MAX_DECOMPRESSED_BLOCK {
            return Err(format!(
                "vector block {index} exceeds the decoded size limit"
            ));
        }

        let block: Arc<[u8]> = output.into();
        while cache.entries.len() >= BLOCK_CACHE_CAPACITY {
            if let Some(oldest) = cache.insertion_order.pop_front() {
                cache.entries.remove(&oldest);
            }
        }
        cache.insertion_order.push_back(index);
        cache.entries.insert(index, Arc::clone(&block));
        Ok(block)
    }
}

/// The embedded vector catalog, decoded once and shared across Rust callers.
pub fn vector_catalog() -> Result<&'static VectorCatalog, String> {
    static CATALOG: OnceLock<Result<VectorCatalog, String>> = OnceLock::new();
    match CATALOG.get_or_init(VectorCatalog::from_embedded) {
        Ok(catalog) => Ok(catalog),
        Err(error) => Err(error.clone()),
    }
}

/// Interpolates one of Apple's nine supported weights and flips y into screen coordinates.
pub fn interpolate_points(geometry: &Geometry, weight: f32) -> Vec<f32> {
    let (source, amount) = if weight >= 400.0 {
        (&geometry.heavy, (weight - 400.0) / 600.0)
    } else {
        (&geometry.light, (400.0 - weight) / 399.0)
    };
    geometry
        .regular
        .iter()
        .zip(source)
        .enumerate()
        .map(|(index, (regular, other))| {
            let value = regular + amount * (other - regular);
            if index % 2 == 1 {
                -value
            } else {
                value
            }
        })
        .collect()
}

/// Calculates tight visible bounds in screen coordinates, preserving Kotlin's
/// cubic-extrema and erase-layer rules.
pub fn visible_bounds(glyph: &Glyph, geometry: &Geometry, weight: f32) -> Result<Bounds, String> {
    if !weight.is_finite() {
        return Err("weight must be finite".to_owned());
    }
    if geometry.regular.len() != geometry.light.len()
        || geometry.regular.len() != geometry.heavy.len()
    {
        return Err("geometry weight masters have different lengths".to_owned());
    }
    let expected = geometry.ops.iter().try_fold(0usize, |count, op| {
        count.checked_add(match op {
            0 | 1 => 2,
            2 => 6,
            3 => 0,
            _ => return None,
        })
    });
    if expected != Some(geometry.regular.len()) {
        return Err("path operations do not match geometry coordinates".to_owned());
    }

    let points = interpolate_points(geometry, weight);
    if points.iter().any(|point| !point.is_finite()) {
        return Err("weight interpolation produced a non-finite coordinate".to_owned());
    }
    let mut drawn = HashSet::new();
    collect_drawn_contours(&glyph.tree, &mut drawn);
    let all = drawn.is_empty();
    let mut contour = 0usize;
    let mut skip = !all && !drawn.contains(&contour);
    let mut bounds = Bounds::empty();
    let mut offset = 0usize;
    let mut current = (0.0f32, 0.0f32);

    for op in &geometry.ops {
        match op {
            0 | 1 => {
                let x = points[offset];
                let y = points[offset + 1];
                include(&mut bounds, x, y, skip);
                current = (x, y);
                offset += 2;
            }
            2 => {
                let p1 = (points[offset], points[offset + 1]);
                let p2 = (points[offset + 2], points[offset + 3]);
                let p3 = (points[offset + 4], points[offset + 5]);
                include(&mut bounds, p3.0, p3.1, skip);
                cubic_axis(current.0, p1.0, p2.0, p3.0, |x| {
                    include(&mut bounds, x, current.1, skip)
                });
                cubic_axis(current.1, p1.1, p2.1, p3.1, |y| {
                    include(&mut bounds, current.0, y, skip)
                });
                current = p3;
                offset += 6;
            }
            _ => {
                contour += 1;
                skip = !all && !drawn.contains(&contour);
            }
        }
    }
    Ok(bounds)
}

fn collect_drawn_contours(nodes: &[LayerNode], drawn: &mut HashSet<usize>) {
    for node in nodes {
        if node.children.is_empty() {
            if !node.mono_erase {
                drawn.extend(node.contours.iter().copied());
            }
        } else {
            collect_drawn_contours(&node.children, drawn);
        }
    }
}

fn include(bounds: &mut Bounds, x: f32, y: f32, skip: bool) {
    if !skip {
        bounds.left = bounds.left.min(x);
        bounds.top = bounds.top.min(y);
        bounds.right = bounds.right.max(x);
        bounds.bottom = bounds.bottom.max(y);
    }
}

fn cubic_axis(p0: f32, p1: f32, p2: f32, p3: f32, mut include: impl FnMut(f32)) {
    let a = -p0 + 3.0 * p1 - 3.0 * p2 + p3;
    let b = 2.0 * (p0 - 2.0 * p1 + p2);
    let c = p1 - p0;
    let at = |t: f32| {
        let u = 1.0 - t;
        u * u * u * p0 + 3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t * p3
    };
    if a.abs() < 1e-6 {
        if b.abs() > 1e-9 {
            let t = -c / b;
            if t > 0.0 && t < 1.0 {
                include(at(t));
            }
        }
    } else {
        let discriminant = b * b - 4.0 * a * c;
        if discriminant >= 0.0 {
            let root = discriminant.sqrt();
            for t in [(-b + root) / (2.0 * a), (-b - root) / (2.0 * a)] {
                if t > 0.0 && t < 1.0 {
                    include(at(t));
                }
            }
        }
    }
}

fn legacy_name(name: &str) -> String {
    let mut result = String::from("SF");
    for part in name.split('.') {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            result.extend(first.to_uppercase());
            result.extend(chars);
        }
    }
    result
}

fn parse_reference(value: &str) -> Result<Option<(usize, usize)>, String> {
    if value.is_empty() {
        return Ok(None);
    }
    let (block, offset) = value
        .split_once(':')
        .ok_or_else(|| format!("malformed geometry reference {value}"))?;
    let block = block
        .parse()
        .map_err(|error| format!("invalid block reference {value}: {error}"))?;
    let offset = offset
        .parse()
        .map_err(|error| format!("invalid byte offset {value}: {error}"))?;
    Ok(Some((block, offset)))
}

fn parse_indices(value: &str) -> Result<Vec<usize>, String> {
    if value.is_empty() {
        return Ok(Vec::new());
    }
    value
        .split(',')
        .map(|item| {
            item.parse()
                .map_err(|error| format!("invalid category index {item}: {error}"))
        })
        .collect()
}

fn parse_strings(value: &str) -> Vec<String> {
    if value.is_empty() {
        Vec::new()
    } else {
        value.split(';').map(str::to_owned).collect()
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
    nodes_read: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8], position: usize) -> Result<Self, String> {
        if position > bytes.len() {
            return Err("glyph offset is past the end of its block".to_owned());
        }
        Ok(Self {
            bytes,
            position,
            nodes_read: 0,
        })
    }

    fn u8(&mut self) -> Result<u8, String> {
        let value = *self
            .bytes
            .get(self.position)
            .ok_or_else(|| "truncated glyph data".to_owned())?;
        self.position += 1;
        Ok(value)
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], String> {
        let end = self
            .position
            .checked_add(count)
            .ok_or_else(|| "glyph byte range overflow".to_owned())?;
        let data = self
            .bytes
            .get(self.position..end)
            .ok_or_else(|| "truncated glyph data".to_owned())?;
        self.position = end;
        Ok(data)
    }

    fn varint(&mut self) -> Result<u32, String> {
        let mut result = 0u32;
        for index in 0..5 {
            let byte = self.u8()?;
            if index == 4 && byte > 0x0f {
                return Err("variable-length integer overflows u32".to_owned());
            }
            result |= u32::from(byte & 0x7f) << (index * 7);
            if byte < 0x80 {
                return Ok(result);
            }
        }
        Err("unterminated variable-length integer".to_owned())
    }

    fn signed(&mut self) -> Result<i32, String> {
        let value = self.varint()?;
        Ok(((value >> 1) as i32) ^ -((value & 1) as i32))
    }

    fn count(&mut self, limit: usize, label: &str) -> Result<usize, String> {
        let count = usize::try_from(self.varint()?)
            .map_err(|error| format!("invalid {label} count: {error}"))?;
        if count > limit {
            return Err(format!("{label} count {count} exceeds the limit {limit}"));
        }
        Ok(count)
    }
}

fn read_geometry(reader: &mut Reader<'_>) -> Result<Geometry, String> {
    let operation_count = reader.count(MAX_VECTOR_OPERATIONS, "path operation")?;
    let ops = reader.take(operation_count)?.to_vec();
    let point_count = ops.iter().try_fold(0usize, |count, op| {
        count.checked_add(match op {
            0 | 1 => 1,
            2 => 3,
            _ => 0,
        })
    });
    let coordinate_count = point_count
        .and_then(|count| count.checked_mul(2))
        .ok_or_else(|| "geometry coordinate count overflow".to_owned())?;
    let mut regular = Vec::with_capacity(coordinate_count);
    let (mut x, mut y) = (0i32, 0i32);
    for _ in 0..coordinate_count / 2 {
        x = x
            .checked_add(reader.signed()?)
            .ok_or_else(|| "regular x coordinate overflow".to_owned())?;
        y = y
            .checked_add(reader.signed()?)
            .ok_or_else(|| "regular y coordinate overflow".to_owned())?;
        regular.extend([x as f32, y as f32]);
    }
    let mut light = Vec::with_capacity(coordinate_count);
    for value in &regular {
        light.push(*value + reader.signed()? as f32);
    }
    let mut heavy = Vec::with_capacity(coordinate_count);
    for value in &regular {
        heavy.push(*value + reader.signed()? as f32);
    }
    Ok(Geometry {
        ops,
        regular,
        light,
        heavy,
    })
}

fn read_glyph(reader: &mut Reader<'_>) -> Result<Glyph, String> {
    let geometry = read_geometry(reader)?;
    let flags = reader.u8()?;
    let wiggle = reader.u8()?;
    let wiggle_angle = if wiggle == 9 { reader.varint()? } else { 0 };
    let rotation = reader.u8()?;
    let anchor_count = reader.count(MAX_VECTOR_OPERATIONS, "anchor")?;
    let mut anchors = Vec::with_capacity(anchor_count * 2);
    for _ in 0..anchor_count * 2 {
        anchors.push(reader.signed()? as f32 / 4096.0);
    }
    let root_count = reader.count(MAX_LAYER_NODES, "root layer")?;
    let mut tree = Vec::with_capacity(root_count);
    for _ in 0..root_count {
        tree.push(read_node(reader, 0)?);
    }
    Ok(Glyph {
        geometry,
        layers_exact: flags & 1 != 0,
        back_to_front: flags & 2 != 0,
        gapless: flags & 4 != 0,
        default_variable_rendering: (flags >> 3) & 3,
        wiggle,
        wiggle_angle,
        rotation,
        anchors,
        tree,
    })
}

fn read_node(reader: &mut Reader<'_>, depth: usize) -> Result<LayerNode, String> {
    if depth >= MAX_LAYER_DEPTH {
        return Err("layer tree exceeds maximum depth".to_owned());
    }
    reader.nodes_read += 1;
    if reader.nodes_read > MAX_LAYER_NODES {
        return Err("layer tree exceeds maximum node count".to_owned());
    }
    let type_flags = reader.u8()?;
    let mode_flags = reader.u8()?;
    let color = reader.u8()?;
    let opacity = reader.u8()?;
    let tie_count = reader.count(MAX_VECTOR_OPERATIONS, "draw-rail tie")?;
    let mut ties = Vec::with_capacity(tie_count * 2);
    for _ in 0..tie_count * 2 {
        ties.push(reader.signed()? as f32 / 1024.0);
    }
    let contour_count = reader.count(MAX_VECTOR_OPERATIONS, "layer contour")?;
    let mut contours = Vec::with_capacity(contour_count);
    for _ in 0..contour_count {
        contours.push(reader.varint()? as usize);
    }
    let tag_count = reader.count(MAX_VECTOR_OPERATIONS, "layer tag")?;
    let mut tags = Vec::with_capacity(tag_count);
    for _ in 0..tag_count {
        tags.push(reader.varint()? as usize);
    }
    let child_count = reader.count(MAX_LAYER_NODES, "child layer")?;
    let mut children = Vec::with_capacity(child_count);
    for _ in 0..child_count {
        children.push(read_node(reader, depth + 1)?);
    }
    Ok(LayerNode {
        is_group: type_flags & 3 == 1,
        can_pulse: type_flags & 4 != 0,
        can_rotate: type_flags & 8 != 0,
        clear_behind: type_flags & 16 != 0,
        animate_together: type_flags & 32 != 0,
        variable_draw: type_flags & 64 != 0,
        auto_variable: type_flags & 128 != 0,
        mono_erase: mode_flags & 1 != 0,
        shape_erase: mode_flags & 2 != 0,
        hierarchical: i32::from((mode_flags >> 2) & 3) - 1,
        color_id: (color != u8::MAX).then_some(usize::from(color)),
        opacity: f32::from(opacity) / 100.0,
        ties,
        contours,
        tags,
        children,
    })
}

fn read_scale_geometry(reader: &mut Reader<'_>, medium: &Geometry) -> Result<Geometry, String> {
    let point_count = medium.regular.len() / 2;
    let transform = |reader: &mut Reader<'_>, master: &[f32]| -> Result<Vec<f32>, String> {
        let mut output = vec![0f32; point_count * 2];
        for axis in 0..2 {
            let scale = reader.signed()?;
            let translation = reader.signed()?;
            for point in 0..point_count {
                let index = point * 2 + axis;
                let base = master[index] as i32;
                let transformed = base
                    .wrapping_mul(scale)
                    .wrapping_add(2048)
                    .wrapping_shr(12)
                    .wrapping_add(translation)
                    .wrapping_add(reader.signed()?);
                output[index] = transformed as f32;
            }
        }
        Ok(output)
    };
    Ok(Geometry {
        ops: medium.ops.clone(),
        regular: transform(reader, &medium.regular)?,
        light: transform(reader, &medium.light)?,
        heavy: transform(reader, &medium.heavy)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fingerprint(u64);

    impl Default for Fingerprint {
        fn default() -> Self {
            Self(0xcbf29ce484222325)
        }
    }

    impl Fingerprint {
        fn byte(&mut self, value: u8) {
            self.0 = (self.0 ^ u64::from(value)).wrapping_mul(1_099_511_628_211);
        }

        fn int(&mut self, value: i32) {
            for byte in value.to_le_bytes() {
                self.byte(byte);
            }
        }

        fn boolean(&mut self, value: bool) {
            self.byte(u8::from(value));
        }

        fn floats(&mut self, values: &[f32]) {
            self.int(values.len() as i32);
            for value in values {
                self.int(value.to_bits() as i32);
            }
        }

        fn integers(&mut self, values: &[usize]) {
            self.int(values.len() as i32);
            for value in values {
                self.int(*value as i32);
            }
        }

        fn geometry(&mut self, geometry: &Geometry) {
            self.int(geometry.ops.len() as i32);
            for op in &geometry.ops {
                self.byte(*op);
            }
            self.floats(&geometry.regular);
            self.floats(&geometry.light);
            self.floats(&geometry.heavy);
        }

        fn glyph(&mut self, glyph: &Glyph) {
            self.geometry(&glyph.geometry);
            self.boolean(glyph.layers_exact);
            self.boolean(glyph.back_to_front);
            self.boolean(glyph.gapless);
            self.byte(glyph.default_variable_rendering);
            self.byte(glyph.wiggle);
            self.int(glyph.wiggle_angle as i32);
            self.byte(glyph.rotation);
            self.floats(&glyph.anchors);
            self.int(glyph.tree.len() as i32);
            for node in &glyph.tree {
                self.node(node);
            }
        }

        fn node(&mut self, node: &LayerNode) {
            self.boolean(node.is_group);
            self.boolean(node.can_pulse);
            self.boolean(node.can_rotate);
            self.boolean(node.clear_behind);
            self.boolean(node.animate_together);
            self.boolean(node.variable_draw);
            self.boolean(node.auto_variable);
            self.boolean(node.mono_erase);
            self.boolean(node.shape_erase);
            self.int(node.hierarchical);
            self.int(node.color_id.map_or(-1, |value| value as i32));
            self.int(node.opacity.to_bits() as i32);
            self.floats(&node.ties);
            self.integers(&node.contours);
            self.integers(&node.tags);
            self.int(node.children.len() as i32);
            for child in &node.children {
                self.node(child);
            }
        }

        fn hex(&self) -> String {
            format!("{:016x}", self.0)
        }
    }

    fn assert_layer_tags(nodes: &[LayerNode], tag_count: usize, name: &str) {
        for node in nodes {
            for tag in &node.tags {
                assert!(*tag < tag_count, "{name} has invalid tag {tag}");
            }
            assert_layer_tags(&node.children, tag_count, name);
        }
    }

    #[test]
    fn every_kotlin_glyph_and_optical_variant_matches_its_reference_fingerprint() {
        let catalog = vector_catalog().expect("embedded vector sources are valid");
        let rows =
            include_str!("../../../sfsymbols/src/jvmTest/resources/sf-vector-fingerprints.txt");
        let references: Vec<_> = rows.lines().collect();
        assert_eq!(catalog.all().len(), references.len());
        assert!(!catalog.tags().is_empty());
        assert!(!catalog.colors().is_empty());

        for (symbol, reference) in catalog.all().iter().zip(references) {
            let fields: Vec<_> = reference.split('|').collect();
            assert_eq!(fields.len(), 4, "malformed fingerprint for {}", symbol.name);
            assert_eq!(fields[0], symbol.name, "catalog order differs");
            let glyph = catalog
                .glyph_for(symbol)
                .unwrap_or_else(|error| panic!("{}: {error}", symbol.name))
                .unwrap_or_else(|| panic!("{} has no geometry", symbol.name));
            assert_eq!(
                glyph.geometry.regular.len(),
                glyph.geometry.light.len(),
                "{}",
                symbol.name
            );
            assert_eq!(
                glyph.geometry.regular.len(),
                glyph.geometry.heavy.len(),
                "{}",
                symbol.name
            );
            assert!(
                glyph.geometry.ops.iter().all(|op| *op <= 3),
                "{}",
                symbol.name
            );
            assert!(
                glyph.anchors.iter().all(|value| value.is_finite()),
                "{}",
                symbol.name
            );
            assert!(
                glyph.geometry.regular.iter().all(|value| value.is_finite()),
                "{}",
                symbol.name
            );
            assert_layer_tags(&glyph.tree, catalog.tags().len(), &symbol.name);

            let mut fingerprint = Fingerprint::default();
            fingerprint.glyph(&glyph);
            assert_eq!(
                fields[1],
                fingerprint.hex(),
                "{} medium glyph differs",
                symbol.name
            );

            for (scale, reference, name) in [
                (OpticalScale::Small, symbol.small.as_ref(), "small"),
                (OpticalScale::Large, symbol.large.as_ref(), "large"),
            ] {
                if reference.is_none() {
                    assert_eq!(fields[if name == "small" { 2 } else { 3 }], "-");
                    continue;
                }
                let geometry = catalog.geometry_at_scale(symbol, &glyph, scale).unwrap();
                let mut fingerprint = Fingerprint::default();
                fingerprint.geometry(&geometry);
                assert_eq!(
                    fields[if name == "small" { 2 } else { 3 }],
                    fingerprint.hex(),
                    "{} {name} optical geometry differs",
                    symbol.name
                );
            }
        }
    }

    #[test]
    fn visible_bounds_match_kotlin_across_weights_and_optical_scales() {
        let catalog = vector_catalog().expect("embedded vector sources are valid");
        let references =
            include_str!("../../../sfsymbols/src/jvmTest/resources/sf-vector-bounds.txt");
        for reference in references.lines() {
            let fields: Vec<_> = reference.split('|').collect();
            assert_eq!(fields.len(), 4, "malformed bounds fingerprint: {reference}");
            let symbol = catalog
                .find(fields[0])
                .unwrap_or_else(|| panic!("unknown bounds symbol {}", fields[0]));
            let glyph = catalog
                .glyph_for(symbol)
                .expect("Kotlin glyph data decodes")
                .expect("Kotlin symbol has a glyph");
            let scale = match fields[2] {
                "small" => OpticalScale::Small,
                "medium" => OpticalScale::Medium,
                "large" => OpticalScale::Large,
                scale => panic!("unknown optical scale {scale}"),
            };
            let geometry = catalog
                .geometry_at_scale(symbol, &glyph, scale)
                .expect("Kotlin optical geometry decodes");
            let weight: f32 = fields[1].parse().expect("valid Kotlin weight");
            let bounds = visible_bounds(&glyph, &geometry, weight)
                .unwrap_or_else(|error| panic!("{} bounds: {error}", symbol.name));
            let mut fingerprint = Fingerprint::default();
            for value in [bounds.left, bounds.top, bounds.right, bounds.bottom] {
                fingerprint.int(value.to_bits() as i32);
            }
            assert_eq!(
                fields[3],
                fingerprint.hex(),
                "{} {} {} bounds differ",
                symbol.name,
                weight,
                fields[2]
            );
        }
    }

    #[test]
    fn lookup_accepts_apple_names_aliases_and_legacy_names() {
        let catalog = vector_catalog().expect("embedded vector sources are valid");
        assert_eq!(catalog.find("heart.fill").unwrap().name, "heart.fill");
        assert_eq!(catalog.find("SFHeartFill").unwrap().name, "heart.fill");
        assert_eq!(catalog.find("airplayaudio").unwrap().name, "airplay.audio");
        assert_eq!(
            catalog.find("phone.arrow.up.right.fill").unwrap().name,
            "phone.arrow.up.right.fill"
        );
        assert_eq!(
            catalog.find("phone.fill.arrow.up.right").unwrap().name,
            "phone.arrow.up.right.fill"
        );
        assert_eq!(catalog.glyph("not-a-symbol").unwrap(), None);
    }

    #[test]
    fn weight_interpolation_uses_regular_at_four_hundred_and_flips_y() {
        let geometry = Geometry {
            ops: vec![0],
            regular: vec![10.0, 20.0],
            light: vec![2.0, 4.0],
            heavy: vec![22.0, 44.0],
        };
        assert_eq!(interpolate_points(&geometry, 400.0), vec![10.0, -20.0]);
        assert_eq!(interpolate_points(&geometry, 1000.0), vec![22.0, -44.0]);
        assert_eq!(interpolate_points(&geometry, 1.0), vec![2.0, -4.0]);
    }

    fn layer(contours: Vec<usize>, mono_erase: bool) -> LayerNode {
        LayerNode {
            is_group: false,
            can_pulse: false,
            can_rotate: false,
            clear_behind: false,
            animate_together: false,
            variable_draw: false,
            auto_variable: false,
            mono_erase,
            shape_erase: false,
            hierarchical: 0,
            color_id: None,
            opacity: 1.0,
            ties: Vec::new(),
            contours,
            tags: Vec::new(),
            children: Vec::new(),
        }
    }

    fn make_glyph(tree: Vec<LayerNode>) -> Glyph {
        Glyph {
            geometry: Geometry {
                ops: Vec::new(),
                regular: Vec::new(),
                light: Vec::new(),
                heavy: Vec::new(),
            },
            layers_exact: true,
            back_to_front: false,
            gapless: false,
            default_variable_rendering: 0,
            wiggle: 0,
            wiggle_angle: 0,
            rotation: 0,
            anchors: Vec::new(),
            tree,
        }
    }

    #[test]
    fn bounds_use_true_cubic_extrema_instead_of_control_points() {
        let glyph = make_glyph(vec![layer(vec![0], false)]);
        let geometry = Geometry {
            ops: vec![0, 2, 3],
            regular: vec![0.0, 0.0, 0.0, 100.0, 100.0, 100.0, 100.0, 0.0],
            light: vec![0.0, 0.0, 0.0, 100.0, 100.0, 100.0, 100.0, 0.0],
            heavy: vec![0.0, 0.0, 0.0, 100.0, 100.0, 100.0, 100.0, 0.0],
        };
        assert_eq!(
            visible_bounds(&glyph, &geometry, 400.0).unwrap(),
            Bounds {
                left: 0.0,
                top: -75.0,
                right: 100.0,
                bottom: 0.0,
            }
        );
        assert!(visible_bounds(&glyph, &geometry, f32::NAN).is_err());
        let explosive = Geometry {
            ops: vec![0],
            regular: vec![1e30, 1e30],
            light: vec![0.0, 0.0],
            heavy: vec![0.0, 0.0],
        };
        assert!(visible_bounds(&glyph, &explosive, 1e38).is_err());
    }

    #[test]
    fn bounds_ignore_erase_only_contours_and_fall_back_when_no_drawn_layer_exists() {
        let geometry = Geometry {
            ops: vec![0, 1, 1, 3, 0, 1, 1, 3],
            regular: vec![
                0.0, 0.0, 100.0, 100.0, 100.0, 0.0, 4.0, 5.0, 10.0, 12.0, 8.0, 7.0,
            ],
            light: vec![
                0.0, 0.0, 100.0, 100.0, 100.0, 0.0, 4.0, 5.0, 10.0, 12.0, 8.0, 7.0,
            ],
            heavy: vec![
                0.0, 0.0, 100.0, 100.0, 100.0, 0.0, 4.0, 5.0, 10.0, 12.0, 8.0, 7.0,
            ],
        };
        let glyph = make_glyph(vec![layer(vec![0], true), layer(vec![1], false)]);
        assert_eq!(
            visible_bounds(&glyph, &geometry, 400.0).unwrap(),
            Bounds {
                left: 4.0,
                top: -12.0,
                right: 10.0,
                bottom: -5.0,
            }
        );

        let erased = make_glyph(vec![layer(vec![0], true)]);
        assert_eq!(
            visible_bounds(&erased, &geometry, 400.0).unwrap(),
            Bounds {
                left: 0.0,
                top: -100.0,
                right: 100.0,
                bottom: 0.0,
            }
        );
    }
}
