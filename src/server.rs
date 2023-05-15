use std::{
  collections::{hash_map::Entry, HashMap},
  fs::File,
  path::PathBuf,
};

use anyhow::{anyhow, bail, Context, Result};
use daemonize::Daemonize;
use tempfile::TempDir;

use crate::{
  buffer::Buffer,
  config::Config,
  highlight::Highlighter,
  kakoune::Kakoune,
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
  fn new(args: Args) -> Result<Self> {
    let tempdir = tempfile::tempdir()?;

    let fifo_req = tempdir.path().join("fifo_req");
    let fifo_buf = tempdir.path().join("fifo_buf");

    Kakoune::send_fifos(&fifo_req, &fifo_buf);

    let mut server = Self {
      config: Config::from_file(args.config.clone())?,
      requests: RequestReader::new(fifo_req, fifo_buf)?,
      kakoune: Kakoune::new(args.session),
      parsers: HashMap::new(),
      buffers: HashMap::new(),
      highlighters: HashMap::new(),
      tempdir,
    };

    server.kakoune.debug(&format!("loading config {:?}", &args.config))?;

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
    self.kakoune.debug("started")?;

    loop {
      let request = self.requests.read_request().context("read_request")?;

      if let Err(err) = self.handle_request(request) {
        self.kakoune.debug(&format!("{err:?}")).context("log_error")?;
      }
    }
  }

  /// Handle a single request
  fn handle_request(&mut self, request: Request) -> Result<()> {
    match request {
      Request::ReloadConfig { config } => {
        self.kakoune.debug(&format!("reloading config from {config:?}"))?;
        self.config.reload(config).context("reload")?;
      }

      Request::NewBuffer { buffer, language } => {
        self.new_buffer(buffer, language);
      }

      Request::SetLanguage { buffer, language } => {
        self.set_buffer_language(&buffer, language).context("set language")?;
      }

      Request::ParseBuffer { buffer, content } => {
        self.parse_buffer(buffer, content).context("parse buffer")?;
      }

      Request::Highlight { buffer } => {
        self.highlight(&buffer).context("highlight")?;
      }
    }

    Ok(())
  }

  /// New buffer.
  fn new_buffer(&mut self, buffer: String, language: String) {
    self.buffers.insert(buffer, Buffer::new(language, None, vec![]));
  }

  /// Sets a buffer's language.
  fn set_buffer_language(&mut self, buffer: &str, language: String) -> Result<()> {
    let Some(mut buffer) = self.buffers.get_mut(buffer) else {
      bail!("buffer {buffer} doesn't exist");
    };

    buffer.language = language;
    buffer.tree = None;

    Ok(())
  }

  /// Updates the buffer's syntax tree.
  fn parse_buffer(&mut self, buffer: String, content: Vec<u8>) -> Result<()> {
    // TODO(enricozb): can we do this without removing the entry?
    // We can by getting a mutable reference to self.parsers, and
    // doing something like Self::get_parser(&mut self.parsers).
    let Some(mut buf) = self.buffers.remove(&buffer) else {
      bail!("unknown buffer: {buffer}");
    };

    let tree = self.get_parser(&buf.language)?.parse(&content)?;

    buf.content = content;
    buf.tree = Some(tree);

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

  /// Highlights a buffer at a timestamp.
  fn highlight(&mut self, bufname: &str) -> Result<()> {
    let buffer = self.buffers.get(bufname).ok_or(anyhow!("unknown buffer: {bufname}"))?;

    let Some(ref tree) = buffer.tree else {
      bail!("buffer {bufname} not parsed");
    };

    println!("highlighting {} bytes", buffer.content.len());

    let highlighter = match self.highlighters.entry(buffer.language.clone()) {
      Entry::Occupied(o) => o.into_mut(),
      Entry::Vacant(v) => v.insert(Highlighter::new(&buffer.language).context("new highlighter")?),
    };

    if let Some(faces) = self.config.faces(&buffer.language) {
      println!("have faces");

      // TODO(enricozb): spawn async thread, or drop the connection.
      let range_specs = highlighter.highlight(faces, tree, &buffer.content);

      println!("found {} range-specs", range_specs.len());

      self.kakoune.highlight(bufname, &range_specs)?;
    } else {
      println!("no faces found");
    }

    Ok(())
  }
}

/// Starts the server with the provided arguments.
pub fn start(args: Args) -> Result<()> {
  let daemonize = args.daemonize;

  let mut server = Server::new(args)?;

  if daemonize {
    let daemon = Daemonize::new()
      .stdout(File::create(server.tempdir.path().join("stdout"))?)
      .stderr(File::create(server.tempdir.path().join("stderr"))?)
      .pid_file(server.pid_file());

    daemon.start()?;
  }

  server.run()?;

  Ok(())
}
