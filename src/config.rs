use std::{
  collections::HashMap,
  fs,
  path::{Path, PathBuf},
};

use anyhow::Result;
use serde::Deserialize;

pub struct Config {
  file: PathBuf,

  config: Server,
}

impl Config {
  /// Loads a `Config` from a file.
  ///
  /// If the file does not exist, the server configuration will be
  /// the default configuration.
  pub fn from_file(file: PathBuf) -> Result<Self> {
    Ok(Self {
      config: Server::from_file(&file)?,
      file,
    })
  }

  /// Reloads a `Config` from its file.
  pub fn reload(&mut self) -> Result<()> {
    self.config = Server::from_file(&self.file)?;

    Ok(())
  }

  /// Get the faces for a language.
  pub fn faces(&self, language: &str) -> Option<&HashMap<String, String>> {
    self.config.language.get(language).map(|language| {
      let faces = &language.faces;
      println!("getting faces: {}", faces.len());

      faces
    })
  }
}

/// Server configuration.
#[derive(Default, Deserialize)]
struct Server {
  /// Language-specific configuration.
  language: HashMap<String, Language>,
}

impl Server {
  /// Loads a `Server` config from a file.
  pub fn from_file(file: &Path) -> Result<Self> {
    if !file.exists() {
      Ok(Self::default())
    } else {
      Ok(toml::from_str(&fs::read_to_string(&file)?)?)
    }
  }
}

/// Language-specific configuration.
#[derive(Deserialize)]
struct Language {
  /// Kakoune faces to use when highlighting.
  faces: HashMap<String, String>,
}