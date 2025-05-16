pub mod config;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Init,
    Fmt {
        #[arg(long)]
        minify: bool,

        /// Dirs to format
        #[arg(value_name = "Dirs")]
        dirs: Vec<PathBuf>,
    },
}
