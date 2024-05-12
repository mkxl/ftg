mod cli_args;
mod client;
mod config;
mod editor;
mod error;
mod server;
mod utils;

use crate::{
    cli_args::{CliArgs, Command},
    client::Client,
    error::Error,
    server::Server,
    utils::any::Any,
};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli_args = CliArgs::parse();

    match cli_args.command {
        Some(Command::Debug) => {}
        None if cli_args.serve => Server::serve(&cli_args).await?.await??,
        None => Client::run(cli_args).await?,
    }

    ().ok()
}
