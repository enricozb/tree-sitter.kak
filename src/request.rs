use std::{io::Read, os::unix::net::UnixListener, path::Path};

use anyhow::Result;
use serde::Deserialize;

use crate::kakoune::connection::Connection;

#[derive(Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Request {
  /// Responds with kakoune commands to write the buffer to disk.
  SaveBuffer { buffer: String },

  /// Sets a buffer's language.
  SetLanguage { buffer: String, language: String },

  /// Reconstructs the buffer's AST.
  Parse { buffer: String },
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

  /// Return the most recent request, blocks if no event is ready.
  pub fn listen(&self) -> Result<(Connection, Request)> {
    let mut stream = self.socket.accept()?.0;

    // TODO(enricozb): log this value
    let mut data = String::new();
    stream.read_to_string(&mut data)?;

    Ok((Connection::new(stream), serde_json::from_str(&data)?))
  }
}
