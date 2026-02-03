mod buffer;
mod buffer_map;
mod cli_args;
mod client;
mod command;
mod config;
mod draw_state;
mod server;
mod view;
mod view_set;
mod window;
mod window_map;

use crate::cli_args::CliArgs;
use anyhow::Error as AnyhowError;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), AnyhowError> {
    CliArgs::parse().run().await
}
