use std::thread;

use anyhow::Result;
use tempfile::TempDir;

use crate::{
  event::{Event, Reader as EventReader},
  kakoune::Kakoune,
  Args,
};

struct Server {
  /// The event reader.
  event_reader: EventReader,

  /// The kakoune instance.
  kakoune: Kakoune,

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
      tempdir,
    })
  }

  /// Runs the server.
  fn run(&mut self) -> Result<()> {
    loop {
      match self.event_reader.read() {
        Ok(Event::Highlight { buffer }) => {
          self.highlight(&buffer)?;
        }

        Err(err) => println!("failed to read event: {err}"),
      }
    }
  }

  fn update_tree(&mut self, buffer: &str) -> Result<()> {
    self.kakoune.save_buffer(buffer)?;

    Ok(())
  }

  // /// Create a buffer's directory if it doesn't exist.
  // fn get_or_create_dir(&self, buffer: &str) -> String {
  //   if let Some(dir) = self.buffers.get(buffer) {
  //     dir.to_string()
  //   } else {
  //     let dir = self.buffers.len();
  //     self.buffers.insert(buffer.to_owned(), dir);

  //     dir.to_string()
  //   }
  // }

  /// Highlights a buffer.
  fn highlight(&mut self, buffer: &str) -> Result<()> {
    self.update_tree(buffer)?;

    Ok(())
  }
}

/// Starts the server with the provided arguments.
pub(crate) fn start(args: &Args) -> Result<()> {
  let mut server = Server::new(args)?;

  if args.daemonize {
    thread::spawn(move || server.run().expect("run"));
  } else {
    server.run()?;
  }

  Ok(())
}
