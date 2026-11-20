//! Ordered favorites and the JSON array format used by the Kotlin app.

use std::collections::HashSet;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Favorites(Vec<String>);

impl Favorites {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let names: Vec<String> = serde_json::from_str(json)?;
        let mut seen = HashSet::with_capacity(names.len());
        Ok(Self(
            names
                .into_iter()
                .filter(|name| seen.insert(name.clone()))
                .collect(),
        ))
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.0)
    }

    pub fn contains(&self, apple_name: &str) -> bool {
        self.0.iter().any(|name| name == apple_name)
    }

    /// Toggle a symbol and return its new favorite state.
    pub fn toggle(&mut self, apple_name: &str) -> bool {
        if let Some(index) = self.0.iter().position(|name| name == apple_name) {
            self.0.remove(index);
            false
        } else {
            self.0.push(apple_name.to_owned());
            true
        }
    }

    pub fn as_slice(&self) -> &[String] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_kotlin_format_and_deduplicates_without_reordering() {
        let favorites = Favorites::from_json(include_str!(
            "../../../app/src/jvmTest/resources/pinned-symbols-kotlin.json"
        ))
        .unwrap();
        assert_eq!(favorites.as_slice(), ["heart.fill", "star"]);
    }

    #[test]
    fn writes_json_the_kotlin_api_can_read_and_toggles_in_order() {
        let mut favorites = Favorites::from_json(r#"["heart.fill","star"]"#).unwrap();
        assert!(favorites.toggle("gear"));
        assert!(!favorites.toggle("heart.fill"));
        assert_eq!(favorites.as_slice(), ["star", "gear"]);

        let serialized = favorites.to_json().unwrap();
        let fixture = include_str!("../../../app/src/jvmTest/resources/pinned-symbols-rust.json")
            .replace("\r\n", "\n");
        assert_eq!(serialized + "\n", fixture);
    }

    #[test]
    fn malformed_saved_data_is_reported_instead_of_becoming_an_empty_set() {
        assert!(Favorites::from_json("{broken").is_err());
    }
}
