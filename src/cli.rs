use crate::{client::Client, error::Error, server::Server};
use clap::{Args, Parser, Subcommand};
use http::Uri;
use std::{net::Ipv4Addr, path::PathBuf};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[command(flatten)]
    pub client_args: ClientArgs,
}

#[derive(Subcommand)]
pub enum Command {
    Serve(ServerArgs),
}

#[derive(Args)]
pub struct ClientArgs {
    #[arg(long, default_value_t = Client::default_server_address())]
    pub server_address: Uri,

    #[arg(long)]
    pub log_filepath: Option<PathBuf>,

    pub filepath: Option<PathBuf>,
}

impl ClientArgs {
    pub async fn run(self) -> Result<(), Error> {
        Client::run(self).await
    }
}

#[derive(Args)]
pub struct ServerArgs {
    #[arg(long, default_value_t = Server::DEFAULT_HOST)]
    pub host: Ipv4Addr,

    #[arg(long, default_value_t = Server::DEFAULT_PORT)]
    pub port: u16,
}

impl ServerArgs {
    pub async fn serve(self) -> Result<(), Error> {
        Server::serve(self).await
    }
}
