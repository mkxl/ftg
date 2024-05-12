mod cli;
mod client;
mod config;
mod editor;
mod error;
// mod serve;
mod server;
mod utils;

use crate::{
    cli::{Cli, Command},
    error::Error,
    utils::any::Any,
};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Serve(args)) => args.serve().await?,
        None => cli.client_args.run().await?,
    }

    ().ok()
}
