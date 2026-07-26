mod buffer;
mod buffer_map;
mod cli_args;
mod client;
mod command;
mod config;
mod operation;
mod region;
mod selection;
mod server;
mod tabs_view;
mod tabs_view_state;
mod view;
mod view_state;
mod view_state_set;
mod window;
mod window_map;
mod window_render_state;

use crate::cli_args::CliArgs;
use anyhow::Error as AnyhowError;
use clap::Parser;

// NOTE: see [stack-overflow-1665eb]
#[expect(clippy::cast_possible_truncation)]
const THREAD_STACK_SIZE: usize = lits::bytes!("8 MiB") as usize;

#[mkutils::tokio_main(thread_stack_size = THREAD_STACK_SIZE)]
async fn main() -> Result<(), AnyhowError> {
    CliArgs::parse().run().await
}
