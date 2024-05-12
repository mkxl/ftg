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
use std::path::Path;

fn init_tracing(log_filepath: Option<&Path>) -> Result<(), Error> {
    let Some(log_filepath) = log_filepath else {
        return ().ok();
    };
    let log_file = log_filepath.create()?;

    // TODO: consider using tracing-appender for writing to a file
    tracing_subscriber::fmt().with_writer(log_file).json().init();

    ().ok()
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli_args = CliArgs::parse();

    init_tracing(cli_args.log_filepath.as_deref())?;

    match cli_args.command {
        Some(Command::Debug) => ().ok(),
        None if cli_args.serve_only => Server::serve(&cli_args).await,
        None => Client::run(cli_args).await,
    }
}
