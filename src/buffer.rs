use tree_sitter::Tree;

/// A syntax tree and it's language.
pub struct Buffer {
  pub language: String,
  pub tree: Tree,
}

impl Buffer {
  /// Creates a new `Buffer`.
  pub fn new(language: String, tree: Tree) -> Self {
    Self { language, tree }
  }
}
