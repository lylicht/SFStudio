use base64::{engine::general_purpose::STANDARD, Engine as _};
use std::{env, fs, io::Write, path::PathBuf};

fn kotlin_string_list(source: &str, marker: &str) -> Result<Vec<String>, String> {
    let start = source
        .find(marker)
        .ok_or_else(|| format!("missing Kotlin list marker {marker}"))?
        + marker.len();
    let bytes = source.as_bytes();
    let mut values = Vec::new();
    let mut i = start;
    let mut depth = 1usize;

    while i < bytes.len() {
        match bytes[i] {
            b'(' => {
                depth += 1;
                i += 1;
            }
            b')' => {
                depth -= 1;
                i += 1;
                if depth == 0 {
                    return Ok(values);
                }
            }
            b'"' => {
                let (value, next) = kotlin_string(source, i + 1)?;
                values.push(value);
                i = next;
            }
            _ => i += 1,
        }
    }

    Err(format!("unterminated Kotlin list for {marker}"))
}

fn kotlin_string(source: &str, mut i: usize) -> Result<(String, usize), String> {
    let bytes = source.as_bytes();
    let mut value = String::new();
    while i < bytes.len() {
        match bytes[i] {
            b'"' => return Ok((value, i + 1)),
            b'\\' => {
                i += 1;
                let escaped = *bytes
                    .get(i)
                    .ok_or_else(|| "unterminated Kotlin string escape".to_owned())?;
                match escaped {
                    b'n' => value.push('\n'),
                    b'r' => value.push('\r'),
                    b't' => value.push('\t'),
                    b'b' => value.push('\u{0008}'),
                    b'"' => value.push('"'),
                    b'\'' => value.push('\''),
                    b'\\' => value.push('\\'),
                    b'$' => value.push('$'),
                    b'u' => {
                        let end = i + 5;
                        let code = u32::from_str_radix(
                            source
                                .get(i + 1..end)
                                .ok_or_else(|| "truncated Kotlin Unicode escape".to_owned())?,
                            16,
                        )
                        .map_err(|error| format!("invalid Kotlin Unicode escape: {error}"))?;
                        let scalar = char::from_u32(code)
                            .ok_or_else(|| format!("invalid Unicode scalar U+{code:04X}"))?;
                        value.push(scalar);
                        i = end - 1;
                    }
                    _ => return Err(format!("unsupported Kotlin escape \\{}", escaped as char)),
                }
                i += 1;
            }
            _ => {
                let ch = source[i..]
                    .chars()
                    .next()
                    .ok_or_else(|| "invalid UTF-8 in Kotlin string".to_owned())?;
                value.push(ch);
                i += ch.len_utf8();
            }
        }
    }
    Err("unterminated Kotlin string".to_owned())
}

fn int_array(source: &str, marker: &str) -> Result<Vec<usize>, String> {
    let start = source
        .find(marker)
        .ok_or_else(|| format!("missing Kotlin array marker {marker}"))?
        + marker.len();
    let end = source[start..]
        .find(')')
        .map(|offset| start + offset)
        .ok_or_else(|| format!("unterminated Kotlin array {marker}"))?;
    source[start..end]
        .split(',')
        .map(|part| {
            part.trim()
                .parse::<usize>()
                .map_err(|error| format!("invalid integer in {marker}: {error}"))
        })
        .collect()
}

fn named_list(source: &str, name: &str) -> Result<Vec<String>, String> {
    let marker = format!("{name}: List<String> = listOf(");
    kotlin_string_list(source, &marker)
}

fn write_lines(output: &PathBuf, chunks: Vec<String>) -> Result<(), String> {
    let mut file = fs::File::create(output).map_err(|error| error.to_string())?;
    for chunk in chunks {
        file.write_all(chunk.as_bytes())
            .map_err(|error| error.to_string())?;
        if !chunk.ends_with('\n') {
            file.write_all(b"\n").map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn run() -> Result<(), String> {
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").ok_or("missing manifest dir")?);
    let data_dir =
        manifest.join("../../sfsymbols/src/commonMain/kotlin/com/composables/sfsymbols/core/data");
    let index_path = data_dir.join("SfBlobIndex.kt");
    let index = fs::read_to_string(&index_path).map_err(|error| error.to_string())?;
    println!("cargo:rerun-if-changed={}", index_path.display());

    let pieces_per_file = index
        .split_once("SF_PIECES_PER_FILE = ")
        .and_then(|(_, rest)| rest.split_whitespace().next())
        .ok_or("missing SF_PIECES_PER_FILE")?
        .parse::<usize>()
        .map_err(|error| error.to_string())?;
    let block_count = index
        .split_once("SF_BLOCK_COUNT = ")
        .and_then(|(_, rest)| rest.split_whitespace().next())
        .ok_or("missing SF_BLOCK_COUNT")?
        .parse::<usize>()
        .map_err(|error| error.to_string())?;
    let block_starts = int_array(&index, "SF_BLOCK_START = intArrayOf(")?;
    if block_starts.len() != block_count + 1 {
        return Err(format!(
            "expected {} block boundaries, found {}",
            block_count + 1,
            block_starts.len()
        ));
    }

    let piece_count = block_starts.last().copied().ok_or("empty block index")?;
    let mut pieces = Vec::with_capacity(piece_count);
    for file_index in 0..piece_count.div_ceil(pieces_per_file) {
        let path = data_dir.join(format!("SfBlobs{file_index}.kt"));
        println!("cargo:rerun-if-changed={}", path.display());
        let source = fs::read_to_string(path).map_err(|error| error.to_string())?;
        let values = kotlin_string_list(&source, "arrayOf(")?;
        if values.len() > pieces_per_file {
            return Err(format!("too many pieces in SfBlobs{file_index}.kt"));
        }
        pieces.extend(values);
    }
    if pieces.len() != *block_starts.last().ok_or("empty block index")? {
        return Err(format!(
            "block index references {} pieces, found {}",
            block_starts.last().unwrap(),
            pieces.len()
        ));
    }

    let output = PathBuf::from(env::var_os("OUT_DIR").ok_or("missing OUT_DIR")?);
    let mut blocks =
        fs::File::create(output.join("sf_vector_blocks.bin")).map_err(|error| error.to_string())?;
    for block in 0..block_count {
        let start = block_starts[block];
        let end = block_starts[block + 1];
        if start > end || end > pieces.len() {
            return Err(format!("invalid block piece range for block {block}"));
        }
        let encoded = pieces[start..end].concat();
        let compressed = STANDARD
            .decode(encoded)
            .map_err(|error| format!("invalid base64 in block {block}: {error}"))?;
        let length = u32::try_from(compressed.len())
            .map_err(|_| format!("compressed block {block} exceeds u32"))?;
        blocks
            .write_all(&length.to_le_bytes())
            .and_then(|_| blocks.write_all(&compressed))
            .map_err(|error| error.to_string())?;
    }

    let catalog_path = data_dir.join("SfCatalogData.kt");
    println!("cargo:rerun-if-changed={}", catalog_path.display());
    let catalog = fs::read_to_string(&catalog_path).map_err(|error| error.to_string())?;
    write_lines(
        &output.join("sf_vector_catalog.txt"),
        named_list(&catalog, "SF_SYMBOL_CHUNKS")?,
    )?;
    write_lines(
        &output.join("sf_vector_aliases.txt"),
        named_list(&catalog, "SF_ALIAS_CHUNKS")?,
    )?;
    write_lines(
        &output.join("sf_vector_tags.txt"),
        named_list(&catalog, "SF_TAG_CHUNKS")?,
    )?;
    write_lines(
        &output.join("sf_vector_colors.txt"),
        kotlin_string_list(&catalog, "SF_LAYER_COLORS = listOf(")?,
    )?;
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        panic!("failed to import Kotlin SF Symbols data: {error}");
    }
}
