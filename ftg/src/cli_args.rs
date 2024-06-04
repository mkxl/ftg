use crate::{client::Client, error::Error, server::Server, utils::any::Any};
use clap::{Parser, Subcommand};
use std::{net::Ipv4Addr, path::PathBuf};

#[derive(Clone, Subcommand)]
pub enum Command {
    Debug,
}

#[derive(Clone, Parser)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[arg(long)]
    pub serve_only: bool,

    #[arg(default_value_t = Server::DEFAULT_HOST, long)]
    pub host: Ipv4Addr,

    #[arg(default_value_t = Server::DEFAULT_PORT, long)]
    pub port: u16,

    #[arg(long = "config")]
    pub config_filepath: Option<PathBuf>,

    #[arg(long = "logs")]
    pub log_filepath: Option<PathBuf>,

    pub paths: Vec<PathBuf>,
}

impl CliArgs {
    fn init_tracing(&self) -> Result<(), Error> {
        let Some(log_filepath) = &self.log_filepath else {
            return ().ok();
        };
        let log_file = log_filepath.create()?;

        // TODO: consider using tracing-appender for writing to a file
        tracing_subscriber::fmt().with_writer(log_file).json().init();

        ().ok()
    }

    pub async fn run(self) -> Result<(), Error> {
        self.init_tracing()?;

        match self.command {
            Some(Command::Debug) => {}
            None if self.serve_only => Server::serve(&self).await?,
            None => Client::run(self).await?,
        }

        ().ok()
    }
}
