mod analysis;
mod archive;
mod cli;
mod diagnostic;
mod elf;
mod evidence;
mod fingerprint;
mod model;
mod output;
mod render;
mod snapshot;
mod vm_oracle;

use clap::Parser;

pub use diagnostic::{ClutterError, Result};

pub fn run() -> Result<()> {
    cli::run(cli::Cli::parse())
}
