use std::{collections::HashMap, thread};

use anyhow::{anyhow, Result};
use tempfile::TempDir;

use crate::{
  buffer::Buffer,
  kakoune::Kakoune,
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

    Ok(Self {
      requests: RequestReader::new(&tempdir.path().join("socket"))?,
      kakoune: Kakoune::new(args.session_id, tempdir.path().join("buffers"))?,
      parsers: HashMap::new(),
      buffers: HashMap::new(),
      tempdir,
    })
  }

  /// Runs the server.
  fn run(&mut self) -> Result<()> {
    loop {
      match self.requests.read() {
        Ok(Request::SetLanguage { buffer, language }) => {
          self.set_buffer_language(buffer, language)?;
        }

        Ok(Request::Parse { buffer }) => {
          self.parse_buffer(buffer)?;
        }

        Err(err) => println!("failed to read request: {err}"),
      }
    }
  }

  /// Sets a buffer's language.
  fn set_buffer_language(&mut self, buffer: String, language: String) -> Result<()> {
    let content_file = self.kakoune.save_buffer(&buffer)?;
    let parser = self.get_parser(language.clone());
    let tree = parser.parse_file(&content_file)?;

    self.buffers.insert(buffer, Buffer::new(language, tree));

    Ok(())
  }

  /// Updates the buffer's syntax tree.
  fn parse_buffer(&mut self, buffer: String) -> Result<()> {
    let mut buf = self.buffers.remove(&buffer).ok_or(anyhow!("unknown buffer {buffer}"))?;

    let content_file = self.kakoune.save_buffer(&buffer)?;
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

  /*
  /// Highlights a buffer.
  fn highlight(&mut self, buffer: &str, language: String) -> Result<()> {
    // TODO(enricozb): omitted b/c only supporting highlighting right now
    // self.parse_buffer(buffer, language)?;

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
    thread::spawn(move || server.run().expect("run"));
  } else {
    server.run()?;
  }

  Ok(())
}
