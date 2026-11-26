use crate::{
    client::Client,
    server::{Connect, Server},
};
use anyhow::Error as AnyhowError;
use camino::Utf8PathBuf;
use clap::{Args, Parser, Subcommand};
use derive_more::From;
use mkutils::{Tracing as Logging, TracingStore as LogsStore, Utils};
use std::io::Error as IoError;
use tracing_subscriber::fmt::{MakeWriter, writer::MakeWriterExt};

#[derive(Args)]
struct Serve;

#[derive(From, Subcommand)]
enum CliCommand {
    Connect(Connect),
    Serve(Serve),
}

#[derive(Args, Debug)]
pub struct LoggingArgs {
    #[arg(long = "log-file")]
    logs_filepath: Option<Utf8PathBuf>,

    #[arg(long = "tokio-console")]
    tokio_console_enabled: bool,

    #[arg(default_value_t = Logging::DEFAULT_TOKIO_CONSOLE_PORT, long = "tokio-console-port")]
    tokio_console_port: u16,
}

#[expect(clippy::unused_self, dead_code)]
impl LoggingArgs {
    const LOGGING_JSON_ENABLED: bool = true;

    fn init_logging_impl<
        W1: 'static + for<'a> MakeWriter<'a> + Send + Sync,
        W2: 'static + for<'a> MakeWriter<'a> + Send + Sync,
    >(
        &self,
        writer_1: W1,
        writer_2: W2,
    ) {
        let writer = writer_1.and(writer_2);

        Logging::default()
            .with_json_enabled(Self::LOGGING_JSON_ENABLED)
            .with_tokio_console_port(self.tokio_console_port)
            .with_tokio_console_enabled(self.tokio_console_enabled)
            .with_writer(writer)
            .init();
    }

    pub fn init_logging(&self, _logs_store_limit: usize) -> Result<LogsStore, IoError> {
        // let [writer_logs_store, reader_logs_store] = LogsStore::new(logs_store_limit).repeated();
        // let [writer_logs_store, reader_logs_store] = LogsStore::inactive().repeated();

        // if let Some(logs_filepath) = self.logs_filepath.as_deref() {
        //     self.init_logging_impl(logs_filepath.create()?, std::io::sink);
        // } else {
        //     self.init_logging_impl(Logging::default_writer, std::io::sink);
        // }

        // reader_logs_store.ok()
        LogsStore::inactive().ok()
    }
}

#[derive(Parser)]
pub struct CliArgs {
    #[arg(long = "config-file")]
    config_filepath: Option<Utf8PathBuf>,

    #[command(flatten)]
    logging_args: LoggingArgs,

    #[command(subcommand)]
    cli_command: Option<CliCommand>,

    #[command(flatten)]
    connect: Connect,
}

impl CliArgs {
    async fn connect(self, connect: Connect) -> Result<(), AnyhowError> {
        Client::connect(connect, self.config_filepath.as_deref(), &self.logging_args)
            .await
            .into_result()
    }

    async fn serve(&self) -> Result<(), AnyhowError> {
        Server::serve(self.config_filepath.as_deref(), &self.logging_args).await
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
