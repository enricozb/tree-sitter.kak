use std::{
  io::Write,
  path::Path,
  process::{Command, Stdio},
};

use anyhow::{anyhow, Result};

use crate::highlight::range::Specs as RangeSpecs;

/// A struct for interacting with a kakoune instance.
pub struct Kakoune {
  /// The session id for the kakoune instance.
  session: String,
}

impl Kakoune {
  /// Creates a new `Kakoune`.
  pub fn new(session: String) -> Self {
    Self { session }
  }

  /// Sends the commands to set the fifo paths to stdout.
  ///
  /// This does not use the usual command sending mechanism as this needs to happen
  /// before the server daemonizes.
  pub fn send_fifos(fifo_req: &Path, fifo_buf: &Path) {
    println!("set-option global tree_sitter_fifo_req {fifo_req:?}");
    println!("set-option global tree_sitter_fifo_buf {fifo_buf:?}");
  }

  /// Highlights a buffer at a timestamp.
  pub fn highlight(&mut self, buffer: &str, ranges: &RangeSpecs) -> Result<()> {
    let ranges: String = ranges.iter().map(|range| format!("'{range}' ")).collect();

    // TODO(enricozb): use the correct timestamp
    // TODO(enricozb): consider chunking the ranges
    self.send_command(
      Some(buffer),
      &format!("set-option buffer tree_sitter_ranges %val{{timestamp}} {ranges}"),
    )?;

    Ok(())
  }

  /// Sends a command to the kakoune session.
  pub fn send_command(&mut self, buffer: Option<&str>, command: &str) -> Result<()> {
    let mut kak = Command::new("kak")
      .arg("-p")
      .arg(&self.session)
      .stdin(Stdio::piped())
      .spawn()?;

    let stdin = kak.stdin.as_mut().ok_or(anyhow!("no stdin"))?;

    if let Some(buffer) = buffer {
      writeln!(stdin, "evaluate-commands -no-hooks -buffer {buffer} %[ {command} ]")?;
    } else {
      writeln!(stdin, "evaluate-commands -no-hooks %[ {command} ]")?;
    }

    Ok(())
  }

  /// Log a debug message to the kakoune instance.
  pub fn debug(&mut self, message: &str) -> Result<()> {
    let mut kak = Command::new("kak")
      .arg("-p")
      .arg(&self.session)
      .stdin(Stdio::piped())
      .spawn()?;

    let stdin = kak.stdin.as_mut().ok_or(anyhow!("no stdin"))?;

    writeln!(
      stdin,
      "evaluate-commands -no-hooks %[ echo -debug %(kak-sitter: {message}) ]"
    )?;

    Ok(())
  }
}
