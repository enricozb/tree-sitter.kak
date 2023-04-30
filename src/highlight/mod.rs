pub mod range;

use std::collections::HashMap;

use anyhow::{anyhow, Result};
use tree_sitter::{Query, QueryCursor, Tree};

use self::range::{Point, Spec as RangeSpec, Specs as RangeSpecs};
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
    content: &[u8],
  ) -> Result<RangeSpecs<'a>> {
    let mut cursor = QueryCursor::new();
    let captures = cursor.captures(&self.query, tree.root_node(), content);
    let capture_names = self.query.capture_names();

    let mut capture_stack: RangeSpecs<'a> = RangeSpecs::new();
    let mut highlights: RangeSpecs<'a> = RangeSpecs::new();
    let mut cur_loc = Point::default();

    for query_match in captures {
      for capture in query_match.0.captures {
        let ts_range = capture.node.range();
        let capture_name = &capture_names[capture.index as usize];

        let Some(face) = faces.get(capture_name) else {
          continue;
        };

        let range = RangeSpec::from((ts_range, face));

        if let Some(last) = capture_stack.last().cloned() {
          if range.start < last.end && last.start != range.start {
            highlights.push(RangeSpec::new(last.start, range.start.prev(), last.face));
          } else if last.end < range.start {
            highlights.push(RangeSpec::new(last.start, last.end, last.face));
            cur_loc = last.end.next();
            capture_stack.pop();

            while capture_stack.last().map_or(false, |last| last.end < range.start) {
              let last = capture_stack.pop().unwrap();
              if cur_loc <= last.end {
                highlights.push(RangeSpec::new(cur_loc, last.end, last.face));
                cur_loc = last.end.next();
              }
            }

            if let Some(last) = capture_stack.last() {
              if cur_loc < range.start {
                highlights.push(RangeSpec::new(cur_loc, range.start.prev(), last.face));
              }
            }
          }
        }

        capture_stack.push(range);
      }
    }

    // remove any remaining ranges in the capture_stack.
    if let Some(last) = capture_stack.last() {
      cur_loc = last.start;
    }

    for capture in capture_stack.into_iter().rev() {
      if cur_loc <= capture.end {
        highlights.push(RangeSpec::new(cur_loc, capture.end, capture.face));
        cur_loc = capture.end.next();
      }
    }

    Ok(highlights)
  }
}
