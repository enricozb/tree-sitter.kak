use std::{
  collections::{hash_map::Entry, HashMap},
  path::PathBuf,
};

use anyhow::{anyhow, Context, Result};
use daemonize::Daemonize;
use tempfile::TempDir;

use crate::{
  buffer::Buffer,
  config::Config,
  highlight::Highlighter,
  kakoune::{connection::Connection, Kakoune},
  request::{Reader as RequestReader, Request},
  tree::Parser,
  Args,
};

struct Server {
  /// Server configuration.
  config: Config,

  /// The request reader.
  requests: RequestReader,

  /// The kakoune instance.
  kakoune: Kakoune,

  /// Tree-sitter parsers to be reused.
  parsers: HashMap<String, Parser>,

  /// The buffers.
  buffers: HashMap<String, Buffer>,

  /// Highlighters to be reused.
  highlighters: HashMap<String, Highlighter>,

  /// The temporary directory containing scratch space.
  /// This is destroyed after this structure is dropped.
  #[allow(unused)]
  tempdir: TempDir,
}

impl Server {
  /// Creates a new `Server`.
  fn new(args: &Args) -> Result<Self> {
    let tempdir = tempfile::tempdir()?;
    let socket = tempdir.path().join("socket");

    let mut server = Self {
      config: Config::from_file(args.config.clone())?,
      requests: RequestReader::new(&socket)?,
      kakoune: Kakoune::new(args.session, tempdir.path().join("buffers"))?,
      parsers: HashMap::new(),
      buffers: HashMap::new(),
      highlighters: HashMap::new(),
      tempdir,
    };

    server.kakoune.send_socket(&socket).context("send socket")?;

    Ok(server)
  }

  /// The path to the pid file.
  ///
  /// This will only exist when the server is daemonized.
  fn pid_file(&self) -> PathBuf {
    self.tempdir.path().join("pid")
  }

  /// Runs the server.
  fn run(&mut self) -> Result<()> {
    loop {
      let (mut connection, request) = self.requests.listen().context("listen")?;

      if let Err(err) = self.handle_request(&mut connection, request) {
        self.kakoune.log_error(&format!("{err:?}")).context("log_error")?;
      }
    }
  }

  /// Handle a single request
  fn handle_request(&mut self, connection: &mut Connection, request: Request) -> Result<()> {
    match request {
      Request::ReloadConfig => {
        self.config.reload()?;
      }

      Request::SaveBuffer { buffer } => {
        self.save_buffer(connection, &buffer)?;
      }

      Request::SetLanguage { buffer, language } => {
        self.set_buffer_language(buffer, language)?;
      }

      Request::Parse { buffer } => {
        self.parse_buffer(buffer)?;
      }

      Request::Highlight { buffer } => {
        // close early so user doesn't wait for parsing.
        connection.close()?;
        self.highlight(&buffer)?;
      }
    }

    Ok(())
  }

  /// Saves a buffer to disk.
  fn save_buffer(&mut self, connection: &mut Connection, buffer: &str) -> Result<()> {
    self.kakoune.save_buffer(connection, buffer)?;

    Ok(())
  }

  /// Sets a buffer's language.
  fn set_buffer_language(&mut self, buffer: String, language: String) -> Result<()> {
    let content_file = self.kakoune.content_file(&buffer)?;
    let parser = self.get_parser(&language)?;
    let tree = parser.parse_file(&content_file)?;

    self.buffers.insert(buffer, Buffer::new(language, tree));

    Ok(())
  }

  /// Updates the buffer's syntax tree.
  fn parse_buffer(&mut self, buffer: String) -> Result<()> {
    // TODO(enricozb): can we do this without removing the entry?
    // We can by getting a mutable reference to self.parsers, and
    // doing something like Self::get_parser(&mut self.parsers).
    let mut buf = self
      .buffers
      .remove(&buffer)
      .ok_or(anyhow!("unknown buffer: {buffer}"))?;

    let content_file = self.kakoune.content_file(&buffer)?;
    buf.tree = self.get_parser(&buf.language)?.parse_file(&content_file)?;

    self.buffers.insert(buffer, buf);

    Ok(())
  }

  /// Returns the parser for the provided language, creating one if needed.
  fn get_parser(&mut self, language: &str) -> Result<&mut Parser> {
    let parser = match self.parsers.entry(language.to_string()) {
      Entry::Occupied(o) => o.into_mut(),
      Entry::Vacant(v) => v.insert(Parser::new(language).context("new parser")?),
    };

    Ok(parser)
  }

  /// Highlight the provided buffer asynchronously.
  fn highlight(&mut self, bufname: &str) -> Result<()> {
    let content_file = self.kakoune.content_file(bufname)?;
    let buffer = self.buffers.get(bufname).ok_or(anyhow!("unknown buffer: {bufname}"))?;
    let highlighter = match self.highlighters.entry(buffer.language.clone()) {
      Entry::Occupied(o) => o.into_mut(),
      Entry::Vacant(v) => v.insert(Highlighter::new(&buffer.language).context("new highlighter")?),
    };

    if let Some(faces) = self.config.faces(&buffer.language) {
      // TODO(enricozb): spawn async thread, or drop the connection.
      let range_specs = highlighter.highlight(faces, &buffer.tree, &content_file)?;

      self.kakoune.highlight(bufname, &range_specs)?;
    }

    Ok(())
  }
}

/// Starts the server with the provided arguments.
pub fn start(args: &Args) -> Result<()> {
  let mut server = Server::new(args)?;

  if args.daemonize {
    let daemon = Daemonize::new().pid_file(server.pid_file());
    daemon.start()?;
  }

  server.run()?;

  Ok(())
}
