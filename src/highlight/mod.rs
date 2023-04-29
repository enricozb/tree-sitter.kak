pub mod range;

use std::{collections::HashMap, fs, path::Path};

use anyhow::{anyhow, Result};
use tree_sitter::{Query, QueryCursor, Tree};

use self::range::{RangeSpec, RangeSpecs};
use crate::languages::Language;

/// A syntax highlighter.
pub struct Highlighter {
  query: Query,
}

impl Highlighter {
  /// Creates a new `Highlighter`.
  pub fn new(language: &str) -> Result<Self> {
    let query: Option<_> = Language::try_from(language)?.into();
    let query = query.ok_or(anyhow!("no highlights"))?;

    Ok(Self { query })
  }

  /// Returns the `RangeSpecs` for highlighting.
  pub fn highlight<'a>(
    &self,
    faces: &'a HashMap<String, String>,
    tree: &Tree,
    content_file: &Path,
  ) -> Result<RangeSpecs<'a>> {
    let source = fs::read(content_file)?;

    let mut cursor = QueryCursor::new();
    let captures = cursor.captures(&self.query, tree.root_node(), source.as_slice());
    let capture_names = self.query.capture_names();

    let mut capture_stack: RangeSpecs<'a> = RangeSpecs::new();
    let mut highlights: RangeSpecs<'a> = RangeSpecs::new();

    for query_match in captures {
      for capture in query_match.0.captures {
        let ts_range = capture.node.range();
        let capture_name = &capture_names[capture.index as usize];
        let face = if let Some(face) = faces.get(capture_name) {
          face
        } else {
          continue;
        };

        let range = RangeSpec::from((ts_range, face));

        if let Some(last) = capture_stack.last().cloned() {
          if range.start < last.end {
            highlights.push(RangeSpec::new(last.start, range.start, last.face));
          } else {
            highlights.push(RangeSpec::new(last.start, last.end, last.face));
            let mut cur_loc = last.end;
            capture_stack.pop();

            while capture_stack
              .last()
              .map(|last| last.end <= range.start)
              .unwrap_or(false)
            {
              let last = capture_stack.pop().unwrap();
              highlights.push(RangeSpec::new(cur_loc, last.end, last.face));
              cur_loc = last.end;
            }

            if let Some(last) = capture_stack.last() {
              highlights.push(RangeSpec::new(cur_loc, range.start, last.face));
            }
          }
        }

        capture_stack.push(range);
      }
    }

    Ok(highlights)
  }
}
