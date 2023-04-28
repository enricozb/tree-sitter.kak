use tree_sitter::Language;

extern "C" {
  pub fn tree_sitter_rust() -> Language;
}
