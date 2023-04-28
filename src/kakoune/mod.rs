pub mod connection;
pub mod range;

use std::{collections::HashMap, fs, path::PathBuf};

use anyhow::Result;

use self::connection::Connection;

/// A struct for interacting with a kakoune instance.
pub struct Kakoune {
  /// The session id for the kakoune instance.
  session_id: i32,

  /// The directory for storing buffer contents.
  buffers_dir: PathBuf,

  // TODO(enricozb): maybe make the value of this map a PathBuf
  /// The buffers that the server is keeping track of. The value is the directory
  /// the buffer is stored in.
  buffers: HashMap<String, usize>,
}

impl Kakoune {
  /// Creates a new `Kakoune`.
  pub fn new(session_id: i32, buffers_dir: PathBuf) -> Result<Self> {
    if !buffers_dir.exists() {
      fs::create_dir(&buffers_dir)?;
    }

    Ok(Self {
      session_id,
      buffers_dir,
      buffers: HashMap::new(),
    })
  }

  /// Returns a path to where the contents of buffer should be stored.
  ///
  /// [`save_buffer`] must be called before [`content_file`] in order to ensure
  /// that the returned path exists and has the expected contents.
  pub fn content_file(&mut self, buffer: &str) -> Result<PathBuf> {
    let buffer_dir = self.buffer_dir(buffer)?;

    Ok(buffer_dir.join("content"))
  }

  /// Writes the buffer contents the diff from the previous buffer, returning the
  /// path of the buffer's content.
  pub fn save_buffer(&mut self, connection: &mut Connection, buffer: &str) -> Result<()> {
    let content_file = self.content_file(buffer)?;

    // TODO(enricozb): implement diffing between content files
    connection.send_command(buffer, &format!("write -force {content_file:?}"))?;

    Ok(())
  }

  // TODO(enricozb): re-implement highlighting
  /*
  pub fn highlight(&mut self, buffer: &str, ranges: &[range::Range]) -> Result<()> {
    self.send_command(buffer, "declare-option -hidden range-specs tree_kak_ranges")?;
    self.send_command(buffer, "declare-option -hidden range-specs tree_kak_ranges_spare")?;
    self.send_command(buffer, "set-option buffer tree_kak_ranges_spare %val{timestamp}")?;
    self.send_command(buffer, "add-highlighter buffer/ ranges tree_kak_ranges")?;

    // TODO(enricozb): determine if chunking is necessary
    for ranges in ranges.chunks(20) {
      let ranges: String = ranges.iter().map(|range| format!("'{range}' ")).collect();

      self.send_command(
        buffer,
        &format!("set-option -add buffer tree_kak_ranges_spare {ranges}"),
      )?;
    }

    self.send_command(buffer, "set-option buffer tree_kak_ranges %opt{tree_kak_ranges_spare}")?;

    Ok(())
  }
  */

  /// Returns a buffer's directory, creating it if necessary.
  fn buffer_dir(&mut self, buffer: &str) -> Result<PathBuf> {
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
}
