use std::{collections::HashMap, fs, path::Path};

use anyhow::Result;
use serde::Deserialize;

/// Server configuration.
#[derive(Default, Deserialize)]
pub struct Config {
  language: HashMap<String, Language>,
}

impl Config {
  /// Parses a `Config` from a file.
  pub fn from_file(path: &Path) -> Result<Self> {
    if !path.exists() {
      return Ok(Config::default());
    }

    Ok(toml::from_str(&fs::read_to_string(path)?)?)
  }

  /// Get the faces for a language.
  pub fn faces(&self, language: &str) -> Option<&HashMap<String, String>> {
    self.language.get(language).map(|language| &language.faces)
  }
}

/// Language-specific configuration.
#[derive(Deserialize)]
pub struct Language {
  /// Kakoune faces to use when highlighting.
  faces: HashMap<String, String>,
}
