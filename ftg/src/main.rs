mod cli_args;
mod client;
mod config;
mod editor;
mod error;
mod server;
mod utils;

use crate::{cli_args::CliArgs, error::Error, utils::any::Any};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), Error> {
    CliArgs::parse().run().await?.ok()
}
