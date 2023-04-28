use std::{collections::HashMap, fs, thread};

use anyhow::Result;
use tempfile::TempDir;
use tree_sitter::{Parser, Tree};

use crate::{
  event::{Event, Reader as EventReader},
  kakoune::{range::Range, Kakoune},
  tree,
  tree::Highlighter,
  Args,
};

struct Server {
  /// The event reader.
  event_reader: EventReader,

  /// The kakoune instance.
  kakoune: Kakoune,

  /// The parsed trees keyed by buffer.
  trees: HashMap<String, Tree>,

  /// Tree-sitter parsers to be reused.
  parsers: HashMap<String, Parser>,

  /// Tree-sitter highlighters to be reused.
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

    Ok(Self {
      event_reader: EventReader::new(&tempdir.path().join("socket"))?,
      kakoune: Kakoune::new(args.session_id, tempdir.path().join("buffers"))?,
      trees: HashMap::new(),
      parsers: HashMap::new(),
      highlighters: HashMap::new(),
      tempdir,
    })
  }

  /// Runs the server.
  fn run(&mut self) -> Result<()> {
    loop {
      match self.event_reader.read() {
        Ok(Event::Highlight { buffer, language }) => {
          self.highlight(&buffer, language)?;
        }

        Err(err) => println!("failed to read event: {err}"),
      }
    }
  }

  /// Updates the buffer's tree.
  #[allow(unused)]
  fn update_tree(&mut self, buffer: String, language: String) -> Result<()> {
    let content_file = self.kakoune.save_buffer(&buffer)?;
    let parser = self
      .parsers
      .entry(language)
      .or_insert_with_key(|language| tree::new_parser(language));

    self.trees.insert(buffer, tree::parse_file(parser, &content_file)?);

    Ok(())
  }

  /// Highlights a buffer.
  fn highlight(&mut self, buffer: &str, language: String) -> Result<()> {
    // TODO(enricozb): omitted b/c only supporting highlighting right now
    // self.update_tree(buffer, language)?;

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
