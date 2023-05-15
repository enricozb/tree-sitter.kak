use anyhow::{anyhow, Result};
use tree_sitter::{Parser as TSParser, Tree};

use crate::languages::Language;

/// A wrapper to tree-sitter's [`TSParser`].
pub struct Parser(TSParser);

impl Parser {
  /// Creates a new `Parser`.
  pub fn new(language: &str) -> Result<Self> {
    let mut parser = TSParser::new();
    parser.set_language(Language::try_from(language)?.into())?;

    Ok(Self(parser))
  }

  /// Parses the `content`.
  pub fn parse(&mut self, content: &[u8]) -> Result<Tree> {
    self.0.parse(content, None).ok_or(anyhow!("parsing error"))
  }
}
