use anyhow::{anyhow, Result};
use tree_sitter::Language;
use tree_sitter_highlight::HighlightConfiguration;

const HIGHLIGHT_NAMES: [&str; 19] = [
  "attribute",
  "constant",
  "comment",
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

/// Returns the `HighlightConfiguration` for a given language.
pub fn highlighter_config(language: &str) -> Result<HighlightConfiguration> {
  let (lang, hl_query, inj_query) = language_queries(language)?;
  let mut config = HighlightConfiguration::new(lang, hl_query, inj_query, "")?;
  config.configure(&HIGHLIGHT_NAMES);

  Ok(config)
}

/// Returns the queries required to create a `HighlightConfiguration` for a given language.
fn language_queries(language: &str) -> Result<(Language, &str, &str)> {
  match language {
    "rust" => Ok((tree_sitter_rust::language(), tree_sitter_rust::HIGHLIGHT_QUERY, "")),
    _ => Err(anyhow!("unsupported language: {language}")),
  }
}
