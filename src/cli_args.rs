use crate::server::Server;
use clap::{Parser, Subcommand};
use std::{net::Ipv4Addr, path::PathBuf};

#[derive(Parser)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[arg(long)]
    pub serve: bool,

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

#[derive(Subcommand)]
pub enum Command {
    Debug,
}
