use std::{fs, path::Path, result::Result as StdResult};

use anyhow::{anyhow, Result};
use tree_sitter::{Parser, Tree};
use tree_sitter_highlight::{
  Error as TSError, HighlightConfiguration as TSConfig, HighlightEvent as TSEvent, Highlighter as TSHighlighter,
};

use crate::languages;

pub struct Highlighter {
  highlighter: TSHighlighter,
  config: TSConfig,
}

impl Highlighter {
  /// Creates a new `Highlighter`.
  pub fn new(language: &str) -> Result<Self> {
    Ok(Self {
      highlighter: TSHighlighter::new(),
      config: languages::highlighter_config(language)?,
    })
  }

  pub fn highlight_file<'a>(
    &'a mut self,
    content: &'a [u8],
  ) -> Result<impl Iterator<Item = StdResult<TSEvent, TSError>> + 'a> {
    Ok(self.highlighter.highlight(&self.config, content, None, |_| None)?)
  }
}

pub fn new_parser(language: &str) -> Parser {
  // TODO(enricozb): parser.set_language()
  Parser::new()
}

pub fn parse_file(parser: &mut Parser, content_file: &Path) -> Result<Tree> {
  let content = fs::read(content_file)?;

  parser.parse(content, None).ok_or(anyhow!("parsing error"))
}
