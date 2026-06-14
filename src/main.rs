mod cli;
mod commands;
mod error;
mod hash;
mod matcher;

use cli::{Cli, Command};
use clap::Parser;
use std::process;

fn main() {
    let cli = Cli::parse();

    let exit_code = match cli.command {
        Command::Read {
            file,
            anchor,
            anchor_file,
        } => commands::read::execute(&file, anchor.as_deref(), anchor_file.as_deref()),
        Command::Write {
            file,
            anchor,
            anchor_file,
            expected_hash,
            replacement,
            replacement_file,
        } => commands::write::execute(
            &file,
            anchor.as_deref(),
            anchor_file.as_deref(),
            &expected_hash,
            replacement.as_deref(),
            replacement_file.as_deref(),
        ),
    };

    process::exit(exit_code);
}
