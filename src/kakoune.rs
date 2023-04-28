use std::{
  collections::HashMap,
  fs,
  io::Write,
  path::PathBuf,
  process::{Command, Stdio},
};

use anyhow::{anyhow, Result};

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

  /// Writes the buffer contents the diff from the previous buffer, returning the
  /// path of the buffer's content.
  pub fn save_buffer(&mut self, buffer: &str) -> Result<PathBuf> {
    let buffer_dir = self.buffer_dir(buffer)?;
    let content_file = buffer_dir.join("content");

    self.send_command(buffer, &format!("write -force {content_file:?}"))?;

    // TODO(enricozb): implement diffing between content files

    Ok(content_file)
  }

  /// Sends a command to the kakoune instance.
  fn send_command(&self, buffer: &str, command: &str) -> Result<()> {
    println!("sending command: {command}");

    let mut kak = Command::new("kak")
      .stdin(Stdio::piped())
      .args(["-p", &self.session_id.to_string()])
      .spawn()?;

    let stdin = kak.stdin.as_mut().ok_or(anyhow!("failed to open stdin"))?;
    write!(stdin, "evaluate-commands -buffer {buffer} %[ {command} ]")?;

    let status = kak.wait()?;

    println!("command status: {status}");

    Ok(())
  }

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
