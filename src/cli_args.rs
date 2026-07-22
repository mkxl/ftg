use crate::{
    client::Client,
    server::{Connect, Server},
};
use anyhow::Error as AnyhowError;
use camino::Utf8PathBuf;
use clap::{Args, Parser, Subcommand};
use derive_more::From;
use mkutils::{Tracing, Utils};
use tracing_subscriber::fmt::MakeWriter;

#[derive(Args)]
struct Serve {}

#[derive(From, Subcommand)]
enum CliCommand {
    Connect(Connect),
    Serve(Serve),
}

#[derive(Parser)]
pub struct CliArgs {
    #[arg(long = "config-file")]
    config_filepath: Option<Utf8PathBuf>,

    #[arg(long = "tracing-file")]
    tracing_filepath: Option<Utf8PathBuf>,

    #[arg(long = "tokio-console")]
    tokio_console_enabled: bool,

    #[arg(default_value_t = Tracing::DEFAULT_TOKIO_CONSOLE_PORT, long = "tokio-console-port")]
    tokio_console_port: u16,

    #[command(subcommand)]
    cli_command: Option<CliCommand>,

    #[command(flatten)]
    connect: Connect,
}

impl CliArgs {
    const TRACING_JSON_ENABLED: bool = true;

    fn init_tracing<W: 'static + for<'a> MakeWriter<'a> + Send + Sync>(&self, writer: W) {
        Tracing::default()
            .with_json_enabled(Self::TRACING_JSON_ENABLED)
            .with_tokio_console_port(self.tokio_console_port)
            .with_tokio_console_enabled(self.tokio_console_enabled)
            .with_writer(writer)
            .init();
    }

    async fn connect(self, connect: Connect) -> Result<(), AnyhowError> {
        if let Some(tracing_filepath) = &self.tracing_filepath {
            self.init_tracing(tracing_filepath.create()?);
        }

        Client::connect(connect, self.config_filepath).await.into_result()
    }

    async fn serve(&self) -> Result<(), AnyhowError> {
        if let Some(tracing_filepath) = &self.tracing_filepath {
            self.init_tracing(tracing_filepath.create()?);
        } else {
            self.init_tracing(Tracing::default_writer);
        }

        Server::serve(self.config_filepath.as_deref()).await
    }

    fn take_cli_command(&mut self) -> CliCommand {
        if let Some(cli_command) = self.cli_command.take() {
            cli_command
        } else {
            self.connect.mem_take().into()
        }
    }

    pub async fn run(mut self) -> Result<(), AnyhowError> {
        match self.take_cli_command() {
            CliCommand::Connect(connect) => self.connect(connect).await,
            CliCommand::Serve(_serve) => self.serve().await,
        }
    }
}
