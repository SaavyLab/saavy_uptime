use anyhow::Result;
use clap::Subcommand;
use rusqlite::Connection;

use crate::{cli::Cli, commands::init::InitArgs, D1CConfig};

mod dump_schema;
mod generate;
mod init;
pub use init::run as run_init;

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Initialize d1c in the current project (creates d1c.toml)
    Init(InitArgs),
    /// Generate Rust bindings from queries (currently only replays migrations)
    Generate,
    /// Print the schema resulting from your migrations
    DumpSchema,
}

pub fn run(conn: &Connection, cli: &Cli, config: &D1CConfig) -> Result<()> {
    match &cli.command {
        Command::Generate => generate::run(conn, &config),
        Command::DumpSchema => dump_schema::run(conn),
        Command::Init(_) => unreachable!("Init command handled in main"),
    }
}
