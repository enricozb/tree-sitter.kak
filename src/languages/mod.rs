use std::result::Result as StdResult;

use anyhow::{anyhow, Result};
use tree_sitter::{Language as TSLanguage, Query as TSQuery};

/// A language.
pub struct Language {
  /// The tree sitter `Language`.
  language: TSLanguage,

  /// Queries for highlighting.
  highlight_query: Option<TSQuery>,
}

impl Language {
  /// Creates a new `Language`.
  pub fn new(language: TSLanguage, query: Option<&str>) -> Result<Self> {
    let highlight_query = query.map(|query| TSQuery::new(language, query)).transpose()?;

    Ok(Self {
      language,
      highlight_query,
    })
  }
}

impl<'a> TryFrom<&'a str> for Language {
  type Error = anyhow::Error;

  /// Return a language given a kakoune filetype string.
  fn try_from(language: &'a str) -> StdResult<Self, Self::Error> {
    let (language, query) = match language {
      "rust" => (tree_sitter_rust::language(), Some(include_str!("./highlight/rust.scm"))),
      _ => return Err(anyhow!("unknown language: {language}")),
    };

    Self::new(language, query)
  }
}

impl From<Language> for TSLanguage {
  fn from(language: Language) -> Self {
    language.language
  }
}

impl From<Language> for Option<TSQuery> {
  fn from(language: Language) -> Self {
    language.highlight_query
  }
}
