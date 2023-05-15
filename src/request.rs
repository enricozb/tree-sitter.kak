use std::{fs, path::PathBuf};

use anyhow::Result;
use nix::{sys::stat::Mode, unistd};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Request {
  /// Reloads the config file.
  ReloadConfig { config: PathBuf },

  // TODO(enricozb): change references from `buffer` to `bufname`
  /// Creates a new buffer, Responds with path for kakoune to write buffer contents to.
  NewBuffer { buffer: String, language: String },

  /// Sets a buffer's language.
  SetLanguage { buffer: String, language: String },

  /// Reconstructs the buffer's AST.
  ParseBuffer {
    buffer: String,

    #[serde(default)]
    content: Vec<u8>,
  },

  /// Highlights the currently parsed buffer asynchronously.
  Highlight { buffer: String },
}

pub struct Reader {
  /// The fifo for requests.
  fifo_req: PathBuf,

  /// The fifo for buffer contents.
  fifo_buf: PathBuf,
}

impl Reader {
  /// Creates a new `Reader`.
  pub fn new(fifo_req: PathBuf, fifo_buf: PathBuf) -> Result<Self> {
    unistd::mkfifo(&fifo_req, Mode::S_IRUSR | Mode::S_IWUSR)?;
    unistd::mkfifo(&fifo_buf, Mode::S_IRUSR | Mode::S_IWUSR)?;

    Ok(Self { fifo_req, fifo_buf })
  }

  /// Return the most recent request, blocks if no event is ready.
  pub fn read_request(&self) -> Result<Request> {
    let mut request = toml::from_str(&fs::read_to_string(&self.fifo_req)?)?;

    if let Request::ParseBuffer { content, .. } = &mut request {
      *content = fs::read(&self.fifo_buf)?;
    }

    Ok(request)
  }
}
