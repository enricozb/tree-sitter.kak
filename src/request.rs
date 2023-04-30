use std::{io::Read, os::unix::net::UnixListener, path::Path};

use anyhow::Result;
use serde::Deserialize;

use crate::kakoune::connection::Connection;

#[derive(Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Request {
  /// Reloads the config file.
  ReloadConfig,

  // TODO(enricozb): change references from `buffer` to `bufname`
  /// Creates a new buffer, Responds with path for kakoune to write buffer contents to.
  NewBuffer { buffer: String, language: String },

  /// Sets a buffer's language.
  SetLanguage { buffer: String, language: String },

  /// Reconstructs the buffer's AST.
  ParseBuffer { buffer: String, timestamp: usize },

  /// Highlights the currently parsed buffer asynchronously.
  Highlight { buffer: String },
}

pub struct Reader {
  /// The listener to the socket.
  socket: UnixListener,
}

impl Reader {
  /// Creates a new `Reader`.
  pub fn new(socket_path: &Path) -> Result<Self> {
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

    println!("request: {data}");

    Ok((Connection::new(stream), toml::from_str(&data)?))
  }
}
