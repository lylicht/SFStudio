//! SFStudio's platform-neutral symbol search, ported from FuzzySearch.kt.
//!
//! The catalog and aliases are embedded from their existing Kotlin source files.
//! That keeps the 7,007 symbols in one authoritative location during migration.
//! Run scripts/check_rust_mcp_parity.py to compare against the original JVM server.

pub mod codegen;

use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

const CATALOG_SOURCE: &str = include_str!(
    "../../../sfsymbols/src/commonMain/kotlin/com/composables/sfsymbols/SfSymbolsCatalog.kt"
);
const TAG_SOURCE: &str = include_str!("../../../app/src/jvmMain/resources/symbol_tags.txt");

#[derive(Debug)]
pub struct Symbol {
    pub apple_name: String,
    pub pascal_name: String,
    pub categories: Vec<String>,
    pub is_restricted: bool,
    segments: Vec<String>,
    pascal_words: Vec<String>,
    normalized: String,
    tags: Vec<String>,
}

#[derive(Debug)]
pub struct Catalog {
    symbols: Vec<Symbol>,
    vocabulary: HashSet<String>,
}

pub fn catalog() -> &'static Catalog {
    static INSTANCE: OnceLock<Catalog> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        Catalog::from_sources(CATALOG_SOURCE, TAG_SOURCE)
            .expect("SFStudio catalog data must be valid")
    })
}

impl Catalog {
    pub fn from_sources(catalog_kotlin: &str, tag_text: &str) -> Result<Self, String> {
        let raw = catalog_kotlin
            .split_once("private val raw: String = \"\"\"")
            .and_then(|(_, following)| following.split_once("\"\"\""))
            .map(|(data, _)| data)
            .ok_or("cannot locate SfSymbolsCatalog.raw")?;
        let tags = parse_tags(tag_text);
        let mut symbols = Vec::new();
        let mut vocabulary = HashSet::new();

        for (i, raw_line) in raw.lines().enumerate() {
            let line = raw_line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<_> = line.split('|').collect();
            if parts.len() != 4 {
                return Err(format!("catalog row {} needs four columns", i + 1));
            }
            let apple_name = parts[0].to_owned();
            let pascal_name = parts[1].to_owned();
            let categories: Vec<String> = if parts[2].is_empty() {
                Vec::new()
            } else {
                parts[2].split(',').map(str::to_owned).collect()
            };
            let segments = split_words(&apple_name.to_lowercase());
            let pascal_words = camel_words(&pascal_name);
            let normalized = apple_name.to_lowercase().replace(['.', '-', '_'], " ");
            let symbol_tags = tags_for(&apple_name, &tags);
            vocabulary.extend(
                segments
                    .iter()
                    .chain(&pascal_words)
                    .chain(&symbol_tags)
                    .cloned(),
            );
            symbols.push(Symbol {
                apple_name,
                pascal_name,
                categories,
                is_restricted: parts[3] == "1",
                segments,
                pascal_words,
                normalized,
                tags: symbol_tags,
            });
        }

        if symbols.is_empty() {
            return Err("catalog contains no symbols".to_owned());
        }
        Ok(Self {
            symbols,
            vocabulary,
        })
    }

    pub fn all(&self) -> &[Symbol] {
        &self.symbols
    }

    pub fn find(&self, code: &str) -> Option<&Symbol> {
        self.symbols.iter().find(|s| {
            s.apple_name.eq_ignore_ascii_case(code) || s.pascal_name.eq_ignore_ascii_case(code)
        })
    }

    /// Search results are ordered identically to the original Kotlin algorithm:
    /// descending score, fewest name segments, shortest name, alphabetical name.
    pub fn search(&self, query: &str, limit: usize) -> Vec<&Symbol> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return self.symbols.iter().take(limit).collect();
        }

        let mut tokens = split_words(&q);
        let meaningful: Vec<String> = tokens.iter().filter(|t| !is_filler(t)).cloned().collect();
        if !meaningful.is_empty() {
            tokens = meaningful;
        }
        if tokens.len() >= 3 {
            tokens.retain(|t| !t.chars().all(|c| c.is_numeric()));
        }

        let mut results: Vec<(i32, usize)> = self
            .symbols
            .iter()
            .enumerate()
            .filter_map(|(i, symbol)| {
                let value = self.score(symbol, &tokens, &q);
                (value > 0).then_some((value, i))
            })
            .collect();
        results.sort_by(|a, b| {
            let left = &self.symbols[a.1];
            let right = &self.symbols[b.1];
            b.0.cmp(&a.0)
                .then_with(|| left.segments.len().cmp(&right.segments.len()))
                .then_with(|| left.apple_name.len().cmp(&right.apple_name.len()))
                .then_with(|| left.apple_name.cmp(&right.apple_name))
        });
        results
            .into_iter()
            .take(limit)
            .map(|(_, i)| &self.symbols[i])
            .collect()
    }

    fn score(&self, symbol: &Symbol, tokens: &[String], q: &str) -> i32 {
        let apple = symbol.apple_name.to_lowercase();
        let mut total = if apple == q {
            1000
        } else if symbol.tags.iter().any(|t| t == q) || apple.starts_with(q) {
            700
        } else if apple.contains(q) {
            350
        } else if symbol.normalized.contains(q) {
            300
        } else {
            0
        };

        let mut strong = false;
        for token in tokens {
            let best = self.token_score(symbol, token);
            if best >= 120 {
                strong = true;
            }
            total += best;
        }
        if !strong {
            return 0;
        }

        let dot_count = q.chars().filter(|c| *c == '.').count();
        if dot_count >= 2 && tokens.len() == dot_count + 1 {
            let matches = tokens
                .iter()
                .filter(|t| {
                    symbol
                        .segments
                        .iter()
                        .any(|s| s == *t || s.starts_with(t.as_str()))
                        || self.edit_match(symbol, t) > 0
                })
                .count();
            if matches < tokens.len() {
                total /= 2;
            }
        }

        let q_no_dots = q.replace('.', "");
        if !q_no_dots.is_empty()
            && !symbol.pascal_words.is_empty()
            && symbol.pascal_words.concat().contains(&q_no_dots)
        {
            total = max(total, 220);
        }
        total
    }

    fn token_score(&self, symbol: &Symbol, t: &str) -> i32 {
        let mut best = 0;
        for seg in &symbol.segments {
            best = max(
                best,
                if seg == t {
                    600
                } else if seg.starts_with(t) {
                    450
                } else {
                    0
                },
            );
        }
        for word in &symbol.pascal_words {
            best = max(
                best,
                if word == t {
                    550
                } else if word.starts_with(t) {
                    380
                } else {
                    0
                },
            );
        }
        if best == 0 && symbol.normalized.contains(t) {
            best = 250;
        }
        let n = t.chars().count();
        if best < 250 && (3..=14).contains(&n) && !self.vocabulary.contains(t) {
            best = max(best, self.edit_match(symbol, t));
        }
        if best == 0 && n >= 3 && subsequence(t, &symbol.normalized) {
            best = 150;
        }
        for category in &symbol.categories {
            let cat = category.to_lowercase();
            let s = if cat == t {
                500
            } else if cat.starts_with(t) {
                320
            } else if n >= 3 && cat.contains(t) {
                200
            } else {
                0
            };
            best = max(best, s);
        }
        for tag in &symbol.tags {
            let s = if tag == t {
                560
            } else if n >= 3 && tag.starts_with(t) {
                300
            } else {
                0
            };
            best = max(best, s);
        }
        best
    }

    fn edit_match(&self, symbol: &Symbol, token: &str) -> i32 {
        let token_len = token.chars().count();
        let budget = edit_budget(token_len);
        let mut best = budget + 1;
        for word in symbol
            .segments
            .iter()
            .chain(&symbol.pascal_words)
            .chain(&symbol.tags)
        {
            let word_len = word.chars().count();
            if (2..=14).contains(&word_len) && word_len.abs_diff(token_len) <= budget {
                best = min(best, bounded_distance(word, token, budget));
            }
        }
        if (1..=budget).contains(&best) {
            300 - 40 * best as i32
        } else {
            0
        }
    }
}

fn split_words(s: &str) -> Vec<String> {
    s.split(['.', '-', '_', ' '])
        .filter(|x| !x.is_empty())
        .map(str::to_owned)
        .collect()
}

fn camel_words(pascal: &str) -> Vec<String> {
    let mut separated = String::new();
    for (i, ch) in pascal.chars().enumerate() {
        if i != 0 && ch.is_uppercase() {
            separated.push(' ');
        }
        separated.extend(ch.to_lowercase());
    }
    separated
        .split(' ')
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

fn is_filler(w: &str) -> bool {
    matches!(
        w,
        "a" | "an"
            | "the"
            | "of"
            | "on"
            | "in"
            | "at"
            | "to"
            | "for"
            | "with"
            | "and"
            | "or"
            | "each"
            | "other"
            | "top"
            | "some"
            | "like"
            | "that"
            | "is"
            | "are"
            | "my"
            | "me"
            | "icon"
            | "symbol"
            | "something"
            | "thing"
    )
}

fn parse_tags(data: &str) -> HashMap<String, Vec<String>> {
    let mut tags: HashMap<String, Vec<String>> = HashMap::new();
    for line in data
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
    {
        if let Some((key, values)) = line.split_once(':') {
            let key = key.trim().to_lowercase();
            if key.is_empty() {
                continue;
            }
            let entry = tags.entry(key).or_default();
            for word in values.split_whitespace() {
                let tag = word.to_lowercase();
                if !entry.contains(&tag) {
                    entry.push(tag);
                }
            }
        }
    }
    tags
}

fn tags_for(name: &str, table: &HashMap<String, Vec<String>>) -> Vec<String> {
    let name = name.to_lowercase();
    let mut result = Vec::new();
    for key in std::iter::once(name.as_str()).chain(name.split('.')) {
        if let Some(values) = table.get(key) {
            for tag in values {
                if !result.contains(tag) {
                    result.push(tag.clone());
                }
            }
        }
    }
    let segments: HashSet<&str> = name.split('.').collect();
    result.retain(|t| !segments.contains(t.as_str()));
    result
}

fn edit_budget(len: usize) -> usize {
    if len <= 4 {
        1
    } else if len <= 7 {
        2
    } else {
        3
    }
}

fn subsequence(needle: &str, haystack: &str) -> bool {
    let mut required = needle.chars();
    let mut next = required.next();
    for ch in haystack.chars() {
        if next == Some(ch) {
            next = required.next();
            if next.is_none() {
                return true;
            }
        }
    }
    next.is_none()
}

/// Optimal string alignment distance, including adjacent transpositions.
/// Matches the Kotlin early-exit rule: two consecutive rows over the budget.
pub fn bounded_distance(a: &str, b: &str, budget: usize) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (la, lb) = (a.len(), b.len());
    if la.abs_diff(lb) > budget {
        return budget + 1;
    }
    if la == 0 {
        return lb;
    }
    if lb == 0 {
        return la;
    }
    let mut prev2 = vec![0; lb + 1];
    let mut prev: Vec<usize> = (0..=lb).collect();
    let mut curr = vec![0; lb + 1];
    let mut prev_min = 0;
    for i in 1..=la {
        curr[0] = i;
        let mut row_min = i;
        for j in 1..=lb {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            let mut d = min(min(curr[j - 1] + 1, prev[j] + 1), prev[j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                d = min(d, prev2[j - 2] + 1);
            }
            curr[j] = d;
            row_min = min(row_min, d);
        }
        if row_min > budget && prev_min > budget {
            return budget + 1;
        }
        prev_min = row_min;
        std::mem::swap(&mut prev2, &mut prev);
        std::mem::swap(&mut prev, &mut curr);
    }
    min(prev[lb], budget + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_the_same_catalog_and_finds_symbol_by_both_names() {
        let index = catalog();
        assert_eq!(index.all().len(), 7007);
        assert_eq!(index.find("SFHeartFill").unwrap().apple_name, "heart.fill");
        assert_eq!(index.find("HEART.FILL").unwrap().pascal_name, "SFHeartFill");
        assert!(index.find("not-a-symbol").is_none());
    }

    #[test]
    fn preserves_existing_search_regressions() {
        let index = catalog();
        for (query, expected) in [
            ("garbage", "trash"),
            ("cog", "gear"),
            ("home", "house"),
            ("tradh", "trash"),
            ("trsh", "trash"),
            ("hosue", "house"),
            ("gearshap", "gearshape"),
            ("garbgae", "trash"),
            ("settigns", "gear"),
            ("msg", "message"),
            ("pw", "lock"),
            ("pics", "photo"),
            ("bunny", "hare"),
            ("zap", "bolt"),
            ("doggy paw", "pawprint"),
            ("four boxes", "square.grid.2x2"),
            ("4 cubes stacked on top of each other", "square.stack.3d.up"),
            ("a yucky can", "trash"),
            ("4 circle", "4.circle"),
        ] {
            assert_eq!(
                index.search(query, 1)[0].apple_name,
                expected,
                "query={query}"
            );
        }
    }

    #[test]
    fn swaps_cost_one_edit_and_filler_query_is_safe() {
        assert_eq!(bounded_distance("house", "hosue", 2), 1);
        assert_eq!(bounded_distance("trash", "trsh", 2), 1);
        assert_eq!(catalog().search("  ", 2).len(), 2);
    }
}
