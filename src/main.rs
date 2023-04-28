mod event;
mod kakoune;
mod server;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
struct Args {
  /// Whether or not to run in the background.
  #[arg(short, long)]
  daemonize: bool,

  /// The kakoune session id to send commands to. If missing, commands will be
  /// written to stdout.
  #[arg(short, long)]
  session_id: i32,
}

fn main() -> Result<()> {
  server::start(&Args::parse())?;

  Ok(())
}
