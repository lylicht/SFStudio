//! Ordered favorites and the JSON array format used by the Kotlin app.

use std::collections::HashSet;
#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
use std::io::Write;
#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
use std::path::Path;

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

    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    pub fn load_file(path: &Path) -> Result<Self, FavoritesFileError> {
        use std::fs;

        fs::create_dir_all(favorites_parent(path)).map_err(FavoritesFileError::Io)?;
        let json = match fs::read_to_string(path) {
            Ok(json) => json,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::default())
            }
            Err(error) => return Err(FavoritesFileError::Io(error)),
        };
        Self::from_json(&json).map_err(FavoritesFileError::InvalidJson)
    }

    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    pub fn save_file(&self, path: &Path) -> Result<(), FavoritesFileError> {
        use std::fs;

        let parent = favorites_parent(path);
        fs::create_dir_all(parent).map_err(FavoritesFileError::Io)?;
        let permissions = match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(FavoritesFileError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "refusing to replace a symlink",
                )))
            }
            Ok(metadata) if metadata.is_file() => {
                let permissions = metadata.permissions();
                if permissions.readonly() {
                    return Err(FavoritesFileError::Io(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        "favorites file is read-only",
                    )));
                }
                Some(permissions)
            }
            Ok(_) => {
                return Err(FavoritesFileError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "favorites destination is not a regular file",
                )))
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(FavoritesFileError::Io(error)),
        };

        let json = self.to_json().map_err(FavoritesFileError::Serialization)?;
        let mut temporary =
            tempfile::NamedTempFile::new_in(parent).map_err(FavoritesFileError::Io)?;
        temporary
            .write_all(json.as_bytes())
            .map_err(FavoritesFileError::Io)?;
        temporary
            .as_file()
            .sync_all()
            .map_err(FavoritesFileError::Io)?;
        if let Some(permissions) = permissions {
            temporary
                .as_file()
                .set_permissions(permissions)
                .map_err(FavoritesFileError::Io)?;
        }
        // shortcut: parent-directory sync is not portable here; add platform-specific sync if power-loss durability becomes a requirement.
        temporary
            .persist(path)
            .map_err(|error| FavoritesFileError::Io(error.error))?;
        Ok(())
    }
}

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
#[derive(Debug)]
pub enum FavoritesFileError {
    InvalidJson(serde_json::Error),
    Serialization(serde_json::Error),
    Io(std::io::Error),
}

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
fn favorites_parent(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
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
