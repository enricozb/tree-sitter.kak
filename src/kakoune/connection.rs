use std::{io::Write, os::unix::net::UnixStream};

use anyhow::Result;

/// A connection from an incoming request.
pub struct Connection(UnixStream);

impl Connection {
  /// Creates a new `Connection`.
  pub fn new(stream: UnixStream) -> Self {
    Self(stream)
  }

  /// Sends a synchronous command to the kakoune instance.
  ///
  /// This is not wrapped in an `evaluate-commands` call as kakoune will
  /// execute the response with `evaluate-commands`.
  pub fn send_sync_command(&mut self, command: &str) -> Result<()> {
    writeln!(self.0, "{command}")?;

    Ok(())
  }

  /// Log an error to the kakoune instance.
  pub fn log_error(&mut self, message: &str) -> Result<()> {
    writeln!(
      self.0,
      "evaluate-commands %[ echo -debug %(kak-tree-sitter: {message}) ]"
    )?;

    Ok(())
  }
}
