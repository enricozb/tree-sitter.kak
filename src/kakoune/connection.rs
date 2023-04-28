use std::{io::Write, os::unix::net::UnixStream};

use anyhow::Result;

/// A connection from an incoming request.
pub struct Connection(UnixStream);

impl Connection {
  /// Creates a new `Connection`.
  pub fn new(stream: UnixStream) -> Self {
    Self(stream)
  }

  /// Sends a command to the kakoune instance.
  pub fn send_command(&mut self, buffer: &str, command: &str) -> Result<()> {
    writeln!(self.0, "evaluate-commands -buffer {buffer} %[ {command} ]")?;

    Ok(())
  }
}
