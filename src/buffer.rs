use tree_sitter::Tree;

/// A syntax tree and it's language.
pub struct Buffer {
  pub language: String,
  pub content: Vec<u8>,
  pub tree: Option<Tree>,
}

impl Buffer {
  /// Creates a new `Buffer`.
  pub fn new<C: Into<Vec<u8>>>(language: String, tree: Option<Tree>, content: C) -> Self {
    Self {
      language,
      tree,
      content: content.into(),
    }
  }
}
