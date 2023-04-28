use std::{
  fmt::{Display, Formatter, Result as FmtResult},
  result::Result as StdResult,
};

use tree_sitter_highlight::{Error as TSError, Highlight, HighlightEvent as TSEvent};

static FACES: [&str; 18] = [
  "meta",  // "attribute"
  "value", // "constant"
  "comment",
  "function", // "function"
  "function", // "function.builtin"
  "meta",     // "function.macro"
  "keyword",
  "operator",
  "identifier", // "property"
  "string",
  "red", // "string.special"
  "red", // "tag"
  "type",
  "type", // "type.builtin"
  "variable",
  "value",    // "variable.builtin"
  "variable", // "variable.parameter"
  "value",    // "constructor"
];

/// A location in a code source.
#[derive(Clone, Copy, Debug, Default)]
pub struct Loc {
  line: usize,
  col: usize,
}

impl Loc {
  /// Offsets both `line` and `col` by one, as kakoune is 1-indexed.
  fn one_idx(self) -> Self {
    Self {
      line: self.line + 1,
      col: self.col + 1,
    }
  }
}

/// A Kakoune range.
#[derive(Debug)]
pub struct Range {
  start: Loc,
  end: Loc,
  highlight: Highlight,
}

impl Range {
  /// Returns `Ranges`
  pub fn from_events(content: &[u8], events: impl Iterator<Item = StdResult<TSEvent, TSError>>) -> Vec<Self> {
    let mut ranges = Vec::new();
    let mut cursor = Cursor::new(content);

    for event in events {
      let event = match event {
        Ok(event) => event,
        Err(err) => {
          println!("event error: {err:?}");
          continue;
        }
      };

      println!("event: {event:?}");

      if let Some(range) = cursor.process_event(event) {
        println!("range: {range:?}");
        ranges.push(range);
      }
    }

    ranges
  }
}

impl Display for Range {
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    write!(
      f,
      "{}.{},{}.{}|{}",
      self.start.line, self.start.col, self.end.line, self.end.col, FACES[self.highlight.0]
    )
  }
}

/// Used to converted `TSEvent` byte offsets to line and column numbers.
struct Cursor<'a> {
  content: &'a [u8],
  offset: usize,
  loc: Loc,
  highlights: Vec<Highlight>,
}

impl<'a> Cursor<'a> {
  /// Creates a new `Cursor`.
  fn new(content: &'a [u8]) -> Self {
    Self {
      content,
      offset: 0,
      loc: Loc::default(),
      highlights: Vec::new(),
    }
  }

  /// Processes a highlight event and updates internal state.
  fn process_event(&mut self, event: TSEvent) -> Option<Range> {
    match event {
      TSEvent::Source { end, .. } => {
        let start = self.loc;
        let end = self.advance(end);

        self.highlights.last().map(|highlight| Range {
          start: start.one_idx(),
          end: end.one_idx(),
          highlight: *highlight,
        })
      }
      TSEvent::HighlightStart(highlight) => {
        self.highlights.push(highlight);
        None
      }
      TSEvent::HighlightEnd => {
        self.highlights.pop();
        None
      }
    }
  }

  /// Advance the cursor to the provided byte offset,
  /// returning the location preceding the new location.
  fn advance(&mut self, new_offset: usize) -> Loc {
    let mut prev_loc = self.loc;

    while self.offset < new_offset {
      let c = self.content[self.offset];

      prev_loc = self.loc;
      if c == b'\n' {
        self.loc.line += 1;
        self.loc.col = 0;
      } else {
        self.loc.col += 1;
      }

      self.offset += 1;
    }

    prev_loc
  }
}
