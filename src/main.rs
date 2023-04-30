mod buffer;
mod config;
mod highlight;
mod kakoune;
mod languages;
mod request;
mod server;
mod tree;

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
pub struct Args {
  // TODO(enricozb): make this optional, and make commands appear on command-line
  /// The kakoune session id to send commands to.
  #[arg(short, long)]
  session: i32,

  /// Whether to run in the background.
  ///
  /// The socket will be printed if --daemonize is set.
  #[arg(short, long)]
  daemonize: bool,

  /// Read config from FILE.
  #[arg(short, long, value_name = "FILE", default_value = "/etc/kak-tree-sitter/config.toml")]
  config: PathBuf,
}

fn main() -> Result<()> {
  server::start(&Args::parse())?;

  Ok(())
}
