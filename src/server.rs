use std::{collections::HashMap, path::PathBuf};

use anyhow::{anyhow, Result};
use daemonize::Daemonize;
use tempfile::TempDir;

use crate::{
  buffer::Buffer,
  kakoune::{connection::Connection, Kakoune},
  request::{Reader as RequestReader, Request},
  tree::Parser,
  Args,
};

struct Server {
  /// The request reader.
  requests: RequestReader,

  /// The kakoune instance.
  kakoune: Kakoune,

  /// Tree-sitter parsers to be reused.
  parsers: HashMap<String, Parser>,

  /// The buffers.
  buffers: HashMap<String, Buffer>,

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

    println!("{}", socket.to_str().ok_or(anyhow!("non-unicode socket path"))?);

    Ok(Self {
      requests: RequestReader::new(&socket)?,
      kakoune: Kakoune::new(args.session_id, tempdir.path().join("buffers"))?,
      parsers: HashMap::new(),
      buffers: HashMap::new(),
      tempdir,
    })
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
      let (mut connection, request) = match self.requests.listen() {
        Ok(ok) => ok,
        Err(err) => {
          println!("failed to read request: {err}");
          continue;
        }
      };

      match request {
        Request::SaveBuffer { buffer } => {
          self.save_buffer(&mut connection, &buffer)?;
        }

        Request::SetLanguage { buffer, language } => {
          self.set_buffer_language(buffer, language)?;
        }

        Request::Parse { buffer } => {
          self.parse_buffer(buffer)?;
        }
      }
    }
  }

  /// Saves a buffer to disk.
  fn save_buffer(&mut self, connection: &mut Connection, buffer: &str) -> Result<()> {
    self.kakoune.save_buffer(connection, buffer)?;

    Ok(())
  }

  /// Sets a buffer's language.
  fn set_buffer_language(&mut self, buffer: String, language: String) -> Result<()> {
    let content_file = self.kakoune.content_file(&buffer)?;
    let parser = self.get_parser(language.clone());
    let tree = parser.parse_file(&content_file)?;

    self.buffers.insert(buffer, Buffer::new(language, tree));

    Ok(())
  }

  /// Updates the buffer's syntax tree.
  fn parse_buffer(&mut self, buffer: String) -> Result<()> {
    // TODO(enricozb): can we do this without removing the entry?
    let mut buf = self.buffers.remove(&buffer).ok_or(anyhow!("unknown buffer {buffer}"))?;

    let content_file = self.kakoune.content_file(&buffer)?;
    buf.tree = self.get_parser(buf.language.clone()).parse_file(&content_file)?;

    self.buffers.insert(buffer, buf);

    Ok(())
  }

  /// Returns the parser for the provided language, creating one if needed.
  fn get_parser(&mut self, language: String) -> &mut Parser {
    self
      .parsers
      .entry(language)
      .or_insert_with_key(|language| Parser::new(language).expect("new parser"))
  }

  // TODO(enricozb): re-implement highlighting
  /*
  /// Highlights a buffer.
  fn highlight(&mut self, buffer: &str, language: String) -> Result<()> {
    let content_file = self.kakoune.save_buffer(buffer)?;
    let content = fs::read(content_file)?;

    let events = self
      .highlighters
      .entry(language)
      .or_insert_with_key(|language| Highlighter::new(language).expect("Highlighter::new"))
      .highlight_file(&content)?;

    self.kakoune.highlight(buffer, &Range::from_events(&content, events))?;

    Ok(())
  }
  */
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