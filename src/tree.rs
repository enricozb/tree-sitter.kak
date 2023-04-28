use std::{fs, path::Path, result::Result as StdResult};

use anyhow::{anyhow, Result};
use tree_sitter::{Parser, Tree};
use tree_sitter_highlight::{
  Error as TSError, HighlightConfiguration as TSConfig, HighlightEvent as TSEvent, Highlighter as TSHighlighter,
};

pub struct Highlighter {
  highlighter: TSHighlighter,
  config: TSConfig,
}

impl Highlighter {
  fn new(language: &str) -> Self {
    Self {
      highlighter: TSHighlighter::new(),
      config: TSConfig::new(),
    }
  }

  pub fn highlight_file<'a>(
    &'a mut self,
    content: &'a [u8],
  ) -> Result<impl Iterator<Item = StdResult<TSEvent, TSError>> + 'a> {
    Ok(self.highlighter.highlight(&self.config, &content, None, |_| None)?)
  }
}

pub fn new_parser(language: &str) -> Parser {
  let parser = Parser::new();

  // TODO(enricozb): parser.set_language()

  parser
}

pub fn new_highlighter(language: &str) -> Highlighter {
  let highlighter = Highlighter::new(language);

  // TODO(enricozb): highlighter.set_language()

  highlighter
}

pub fn parse_file(parser: &mut Parser, content_file: &Path) -> Result<Tree> {
  let content = fs::read(content_file)?;

  parser.parse(content, None).ok_or(anyhow!("parsing error"))
}
