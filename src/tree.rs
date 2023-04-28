use std::{fs, path::Path, result::Result as StdResult};

use anyhow::{anyhow, Result};
use tree_sitter::{Parser, Tree};
use tree_sitter_highlight::{
  Error as TSError, HighlightConfiguration as TSConfig, HighlightEvent as TSEvent, Highlighter as TSHighlighter,
};

use crate::languages;

const HIGHLIGHT_NAMES: [&str; 18] = [
  "attribute",
  "constant",
  "function.builtin",
  "function",
  "keyword",
  "operator",
  "property",
  "punctuation",
  "punctuation.bracket",
  "punctuation.delimiter",
  "string",
  "string.special",
  "tag",
  "type",
  "type.builtin",
  "variable",
  "variable.builtin",
  "variable.parameter",
];

pub struct Highlighter {
  highlighter: TSHighlighter,
  config: TSConfig,
}

impl Highlighter {
  fn new(language: &str) -> Self {
    let rust = unsafe { languages::tree_sitter_rust() };
    let mut config = TSConfig::new(tree_sitter_rust::language(), tree_sitter_rust::HIGHLIGHT_QUERY, "", "").unwrap();
    config.configure(&HIGHLIGHT_NAMES);

    Self {
      highlighter: TSHighlighter::new(),
      config,
    }
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

pub fn new_highlighter(language: &str) -> Highlighter {
  // TODO(enricozb): highlighter.set_language()
  Highlighter::new(language)
}

pub fn parse_file(parser: &mut Parser, content_file: &Path) -> Result<Tree> {
  let content = fs::read(content_file)?;

  parser.parse(content, None).ok_or(anyhow!("parsing error"))
}
