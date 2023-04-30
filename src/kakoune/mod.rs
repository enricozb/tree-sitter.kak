pub mod connection;
use std::{
  collections::HashMap,
  fs,
  io::Write,
  path::{Path, PathBuf},
  process::{Command, Stdio},
};

use anyhow::{anyhow, Result};

use self::connection::Connection;
use crate::highlight::range::Specs as RangeSpecs;

/// A struct for interacting with a kakoune instance.
pub struct Kakoune {
  // TODO(enricozb): change to usize (or is it a string?)
  /// The session id for the kakoune instance.
  session: i32,

  /// The directory for storing buffer contents.
  buffers_dir: PathBuf,

  // TODO(enricozb): maybe make the value of this map a PathBuf
  /// The buffers that the server is keeping track of. The value is the directory
  /// the buffer is stored in.
  buffers: HashMap<String, usize>,
}

impl Kakoune {
  /// Creates a new `Kakoune`.
  pub fn new(session: i32, buffers_dir: PathBuf) -> Result<Self> {
    if !buffers_dir.exists() {
      fs::create_dir(&buffers_dir)?;
    }

    Ok(Self {
      session,
      buffers_dir,
      buffers: HashMap::new(),
    })
  }

  /// Sends the socket path to the kakoune instance.
  pub fn send_socket(&mut self, socket: &Path) -> Result<()> {
    self.send_command(None, &format!("set-option global tree_sitter_socket {socket:?}"))
  }

  /// Creates a new buffer directory.
  pub fn new_buffer(&mut self, connection: &mut Connection, buffer: &str) -> Result<()> {
    let buffer_dir = self.buffer_dir(buffer)?;
    connection.send_sync_command(&format!("set-option buffer tree_sitter_dir {buffer_dir:?}"))?;

    Ok(())
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

  /// Returns a buffer's content file path.
  pub fn content_file(&mut self, buffer: &str, timestamp: usize) -> Result<PathBuf> {
    Ok(self.buffer_dir(buffer)?.join(timestamp.to_string()))
  }

  /// Returns a buffer's directory, creating it if necessary.
  pub fn buffer_dir(&mut self, buffer: &str) -> Result<PathBuf> {
    if let Some(dir) = self.buffers.get(buffer) {
      Ok(self.buffers_dir.join(dir.to_string()))
    } else {
      let dir = self.buffers.len();
      self.buffers.insert(buffer.to_owned(), dir);
      let dir = self.buffers_dir.join(dir.to_string());

      fs::create_dir(&dir)?;

      Ok(dir)
    }
  }

  /// Sends a command to the kakoune session.
  pub fn send_command(&mut self, buffer: Option<&str>, command: &str) -> Result<()> {
    let mut kak = Command::new("kak")
      .arg("-p")
      .arg(self.session.to_string())
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
      .arg(self.session.to_string())
      .stdin(Stdio::piped())
      .spawn()?;

    let stdin = kak.stdin.as_mut().ok_or(anyhow!("no stdin"))?;

    writeln!(
      stdin,
      "evaluate-commands -no-hooks %[ echo -debug %(kak-tree-sitter: {message}) ]"
    )?;

    Ok(())
  }
}
