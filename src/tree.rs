use std::{fs, path::Path};

use anyhow::{anyhow, Result};
use tree_sitter::{Parser as TSParser, Tree};

use crate::languages;

/// A wrapper to tree-sitter's [`TSParser`].
pub struct Parser(TSParser);

impl Parser {
  /// Creates a new `Parser`.
  pub fn new(language: &str) -> Result<Self> {
    let mut parser = TSParser::new();
    parser.set_language(languages::from(language)?)?;

    Ok(Self(parser))
  }

  /// Parses the file located at `content_file`.
  pub fn parse_file(&mut self, content_file: &Path) -> Result<Tree> {
    let content = fs::read(content_file)?;

    self.0.parse(content, None).ok_or(anyhow!("parsing error"))
  }
}
