//! DotDog 🐶 - Modern dotfile compositor with git versioning & DotDog TUI
//!
//! Companion to NoteDog in the Woofson canine tool suite.

extern crate dotdog_core as dmcore;

pub mod app;
pub mod cli;
pub mod theme;
pub mod tui;
pub mod ui;

use clap::Parser;

pub fn run() -> anyhow::Result<()> {
    let cli_args = cli::Cli::parse();

    if let Some(cmd) = cli_args.command {
        if cmd == cli::Commands::Tui {
            tui::run_tui()
        } else {
            cli::run_cli(cmd, cli_args.json)
        }
    } else {
        // Default when no subcommand is provided: launch TUI!
        tui::run_tui()
    }
}
