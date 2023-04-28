mod buffer;
mod kakoune;
mod languages;
mod request;
mod server;
mod tree;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
pub struct Args {
  /// The kakoune session id to send commands to. If missing, commands will be
  /// written to stdout.
  #[arg(short, long)]
  session_id: i32,

  /// Whether to run in the background.
  #[arg(short, long)]
  daemonize: bool,
}

fn main() -> Result<()> {
  server::start(&Args::parse())?;

  Ok(())
}
