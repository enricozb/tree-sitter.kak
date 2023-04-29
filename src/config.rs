use std::{
  collections::HashMap,
  fs,
  path::{Path, PathBuf},
};

use anyhow::Result;
use serde::Deserialize;

/// Server configuration.
#[derive(Deserialize)]
pub struct Config {
  file: PathBuf,
  language: HashMap<String, Language>,
}

impl Config {
  /// Creates an empty `Config`.
  pub fn new(file: PathBuf) -> Self {
    Self {
      file,
      language: HashMap::new(),
    }
  }

  /// Loads a `Config` from a file.
  pub fn from_file(file: &Path) -> Result<Self> {
    let mut config = Self::new(file.to_path_buf());
    config.reload()?;

    Ok(config)
  }

  /// Reloads a `Config` from its file.
  pub fn reload(&mut self) -> Result<()> {
    if !self.file.exists() {
      self.language = HashMap::new();
    }

    self.language = toml::from_str(&fs::read_to_string(&self.file)?)?;

    Ok(())
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
