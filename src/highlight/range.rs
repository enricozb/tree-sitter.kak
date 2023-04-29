use std::fmt::{Display, Formatter, Result as FmtResult};

use tree_sitter::{Point as TSPoint, Range as TSRange};

pub type RangeSpecs<'a> = Vec<RangeSpec<'a>>;

#[derive(Clone)]
pub struct RangeSpec<'a> {
  pub start: Point,
  pub end: Point,
  pub face: &'a str,
}

impl<'a> RangeSpec<'a> {
  /// Creates a new `RangeSpec`.
  pub fn new(start: Point, end: Point, face: &'a str) -> Self {
    Self {
      start,
      end: end.prev(),
      face,
    }
  }
}

impl<'a> From<(TSRange, &'a String)> for RangeSpec<'a> {
  fn from((range, face): (TSRange, &'a String)) -> Self {
    Self {
      start: range.start_point.into(),
      end: range.end_point.into(),
      face,
    }
  }
}

impl<'a> Display for RangeSpec<'a> {
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    write!(
      f,
      "{}.{},{}.{}|{}",
      self.start.row, self.start.column, self.end.row, self.end.column, self.face
    )?;

    Ok(())
  }
}

/// A start or end point for a [`RangeSpec`].
///
/// Points are 1-indexed.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point {
  row: usize,
  column: usize,
}

impl Point {
  /// The previous point.
  fn prev(&self) -> Self {
    Self {
      row: self.row,
      column: self.column - 1,
    }
  }
}

impl From<TSPoint> for Point {
  fn from(point: TSPoint) -> Self {
    Self {
      row: point.row + 1,
      column: point.column + 1,
    }
  }
}
