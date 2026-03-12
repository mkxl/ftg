mod buffer;
mod buffer_map;
mod cli_args;
mod client;
mod command;
mod config;
mod draw_state;
mod region;
mod selection;
mod selection_set;
mod server;
mod view;
mod view_state;
mod view_state_set;
mod window;
mod window_map;

use crate::cli_args::CliArgs;
use anyhow::Error as AnyhowError;
use clap::Parser;

// NOTE: see [stack-overflow-1665eb]
#[allow(clippy::cast_possible_truncation)]
const THREAD_STACK_SIZE: usize = lits::bytes!("8 MiB") as usize;

#[mkutils::main(thread_stack_size = THREAD_STACK_SIZE)]
async fn main() -> Result<(), AnyhowError> {
    CliArgs::parse().run().await
}
