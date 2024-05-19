use crate::server::Server;
use clap::{Parser, Subcommand};
use std::{net::Ipv4Addr, path::PathBuf};

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

    pub filepath: Option<PathBuf>,
}

#[derive(Clone, Subcommand)]
pub enum Command {
    Debug,
}
