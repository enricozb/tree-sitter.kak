use anyhow::{anyhow, Result};
use tree_sitter::Language;

/// Return a language given a kakoune filetype string.
pub fn from(language: &str) -> Result<Language> {
  match language {
    "rust" => Ok(tree_sitter_rust::language()),
    _ => Err(anyhow!("unknown language: {language}")),
  }
}
