use std::{io::Read, os::unix::net::UnixListener, path::Path};

use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Event {
  Highlight { buffer: String, language: String },
}

pub struct Reader {
  /// The listener to the socket.
  socket: UnixListener,
}

impl Reader {
  /// Creates a new `Reader`.
  pub fn new(socket_path: &Path) -> Result<Self> {
    println!("listening on: {socket_path:?}");

    Ok(Self {
      socket: UnixListener::bind(socket_path)?,
    })
  }

  /// Return the most recent event, blocks if no event is ready.
  pub fn read(&self) -> Result<Event> {
    let mut stream = self.socket.accept()?.0;

    let mut data = String::new();

    // TODO(enricozb): log this value
    let _num = stream.read_to_string(&mut data)?;

    Ok(serde_json::from_str(&data)?)
  }
}
