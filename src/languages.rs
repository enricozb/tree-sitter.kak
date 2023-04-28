use anyhow::{anyhow, Result};
use tree_sitter::Language;
use tree_sitter_highlight::HighlightConfiguration;

static HIGHLIGHT_NAMES: [&str; 18] = [
  "attribute",
  "constant",
  "comment",
  "function",
  "function.builtin",
  "function.macro",
  "keyword",
  "operator",
  "property",
  "string",
  "string.special",
  "tag",
  "type",
  "type.builtin",
  "variable",
  "variable.builtin",
  "variable.parameter",
  "constructor",
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
