use crate::{
    buffer::Buffer,
    buffer_map::BufferMap,
    config::Config,
    window::{Window, WindowCrosstermEvent, WindowOutput},
    window_map::WindowMap,
};
use anyhow::{Context, Error as AnyhowError};
use camino::{Utf8Path, Utf8PathBuf};
use clap::Args;
use mkutils::{Output, PointU16, Rope, Screen, Socket, Utils};
use serde::{Deserialize, Serialize};
use std::{future::Future, io::Error as IoError, path::Path, pin::Pin, time::Duration};
use tokio::{
    net::{UnixListener, UnixStream, unix::SocketAddr},
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::{JoinError, JoinSet},
    time::Interval,
};
use ulid::Ulid;

enum Operation {
    InitializeBuffer { buffer_id: Ulid, rope: Rope },
}

type OperationFuture = Pin<Box<dyn 'static + Future<Output = Result<Operation, AnyhowError>> + Send + Sync>>;

#[derive(Args, Default, Deserialize, Serialize)]
pub struct Connect {
    paths: Vec<Utf8PathBuf>,

    #[arg(skip = Screen::size().unwrap())]
    size: PointU16,
}

pub struct Server {
    unix_listener: UnixListener,
    tick_interval: Interval,
    config: Config,
    buffer_map: BufferMap,
    window_map: WindowMap,
    operation_future_sender: UnboundedSender<OperationFuture>,
    operation_future_receiver: UnboundedReceiver<OperationFuture>,
    join_set: JoinSet<Result<Operation, AnyhowError>>,
    operation_sender: UnboundedSender<Operation>,
    operation_receiver: UnboundedReceiver<Operation>,
}

impl Server {
    const CONTEXT_TOP_LEVEL: &str = "error in server top-level loop";
    const CONTEXT_NEW_VIEW: &str = "unable to create new view";
    const DEFAULT_CONFIG_YAML_STR: &str = std::include_str!("default-config.yaml");
    const TICK_PERIOD_DURATION: Duration = Duration::from_millis(15);

    fn new(config_filepath: Option<&Path>) -> Result<Self, AnyhowError> {
        let unix_listener = Self::get_unix_listener()?;
        let config = Self::get_config(config_filepath)?;
        let buffer_map = BufferMap::new();
        let tick_interval = Self::TICK_PERIOD_DURATION.into_interval();
        let window_map = WindowMap::new();
        let (operation_future_sender, operation_future_receiver) = tokio::sync::mpsc::unbounded_channel();
        let join_set = JoinSet::new();
        let (operation_sender, operation_receiver) = tokio::sync::mpsc::unbounded_channel();
        let server = Self {
            unix_listener,
            tick_interval,
            config,
            buffer_map,
            window_map,
            operation_future_sender,
            operation_future_receiver,
            join_set,
            operation_sender,
            operation_receiver,
        };

        server.ok()
    }

    pub fn get_unix_listener() -> Result<UnixListener, AnyhowError> {
        let socket_filepath = Self::socket_filepath();

        if socket_filepath.try_exists()? {
            socket_filepath.remove_file()?;
        }

        UnixListener::bind(socket_filepath)?.ok()
    }

    pub fn socket_filepath() -> &'static Utf8Path {
        Utf8Path::new("/tmp/ftg.socket")
    }

    fn get_config<P: AsRef<Path>>(config_filepath: Option<P>) -> Result<Config, AnyhowError> {
        if let Some(config_filepath) = config_filepath {
            config_filepath
                .open()?
                .buf_reader()
                .to_value_from_yaml_reader::<Config>()?
        } else {
            Self::DEFAULT_CONFIG_YAML_STR.to_value_from_yaml_slice()?
        }
        .ok()
    }

    async fn initialize_buffer(buffer_id: Ulid, filepath: Utf8PathBuf) -> Result<Operation, AnyhowError> {
        let rope = Buffer::rope_from_filepath(&filepath).await?;
        let operation = Operation::InitializeBuffer { buffer_id, rope };

        operation.ok()
    }

    async fn get_or_insert_buffer_from_filepath(&mut self, filepath: &Utf8Path) -> Result<Ulid, AnyhowError> {
        let (buffer_id, should_create_buffer) = self.buffer_map.get_or_insert_from_filepath(filepath).await?;

        if should_create_buffer {
            let operation_future = Self::initialize_buffer(buffer_id, filepath.to_owned());

            self.operation_future_sender.send(operation_future.pin())?;
        }

        buffer_id.ok()
    }

    async fn new_window(&mut self, socket: Socket, connect: Connect) -> Result<Window, IoError> {
        let mut window = Window::new(socket, connect.size, &self.config.key_map)?;

        for path in connect.paths {
            match self.get_or_insert_buffer_from_filepath(&path).await {
                Ok(buffer_id) => window.push(buffer_id, path.some()),
                Err(error) => error.context(Self::CONTEXT_NEW_VIEW).log_error().mem_drop(),
            }
        }

        window.ok()
    }

    async fn on_unix_stream(&mut self, pair_res: Result<(UnixStream, SocketAddr), IoError>) -> Result<(), AnyhowError> {
        let (unix_stream, _socket_addr) = pair_res?;
        let mut socket = unix_stream.convert::<Socket>();
        let connect = socket.recv::<Connect>().await.into_option().check_next()??;
        let window = self.new_window(socket, connect).await?;

        self.window_map.insert(window);

        ().ok()
    }

    fn close_window(&mut self, window_id: &Ulid, anyhow_res: Result<(), AnyhowError>) {
        self.window_map.remove(window_id);

        mkutils::trace!(
            level = anyhow_res.level(),
            status = %anyhow_res.status_display(),
            "terminated client"
        );
    }

    async fn on_window_crossterm_event(
        &mut self,
        window_crossterm_event: WindowCrosstermEvent,
    ) -> Result<(), AnyhowError> {
        let window = self
            .window_map
            .get_mut(&window_crossterm_event.window_id)
            .check_present()?;
        let anyhow_output = match window_crossterm_event.crossterm_event {
            Output::Ok(crossterm_event) => {
                window
                    .on_crossterm_event(&self.config, &mut self.buffer_map, crossterm_event)
                    .await
            }
            Output::EndOk => Output::EndOk,
            Output::EndErr(anyhow_error) => Output::EndErr(anyhow_error),
        };
        let Some(anyhow_res) = anyhow_output.into_end() else { return ().ok() };

        self.close_window(&window_crossterm_event.window_id, anyhow_res);

        ().ok()
    }

    fn on_window_output(&mut self, window_output: WindowOutput) {
        let Some(anyhow_res) = window_output.output.into_end() else { return };

        self.close_window(&window_output.window_id, anyhow_res);
    }

    async fn on_tick(&mut self) {
        self.window_map
            .on_tick(&self.config, &mut self.buffer_map)
            .await
            .into_iter()
            .for_each(|window_output| self.on_window_output(window_output));
    }

    fn on_operation_future(&mut self, operation_future_opt: Option<OperationFuture>) -> Result<(), AnyhowError> {
        self.join_set.spawn(operation_future_opt.check_next()?).unit().ok()
    }

    fn on_operation(
        &self,
        operation_res_res: Result<Result<Operation, AnyhowError>, JoinError>,
    ) -> Result<(), AnyhowError> {
        self.operation_sender.send(operation_res_res??)?.ok()
    }

    fn complete_operation(&mut self, operation_opt: Option<Operation>) -> Result<(), AnyhowError> {
        match operation_opt.check_next()? {
            Operation::InitializeBuffer { buffer_id, rope } => {
                self.buffer_map.get_mut(&buffer_id).check_present()?.set_rope(rope).ok()
            }
        }
    }

    async fn step(&mut self) -> Result<(), AnyhowError> {
        tokio::select! {
            pair_res = self.unix_listener.accept() => self.on_unix_stream(pair_res).await,
            window_crossterm_event = self.window_map.next_window_crossterm_event() => {
                self.on_window_crossterm_event(window_crossterm_event).await
            },
            _instant = self.tick_interval.tick() => self.on_tick().await.ok(),
            operation_future_opt = self.operation_future_receiver.recv(), if self.join_set.len() < self.config.max_concurrency => self.on_operation_future(operation_future_opt),
            operation_res_res = self.join_set.join_next().wait_then_unwrap_or_pending() => self.on_operation(operation_res_res),
            operation_opt = self.operation_receiver.recv() => self.complete_operation(operation_opt)
        }
    }

    async fn run(mut self) {
        loop {
            self.step()
                .await
                .context(Self::CONTEXT_TOP_LEVEL)
                .log_if_error()
                .mem_drop();
        }
    }

    #[tracing::instrument(err)]
    pub async fn serve(config_filepath: Option<&Utf8Path>) -> Result<(), AnyhowError> {
        Self::new(config_filepath.map_as_ref())?.run().await.ok()
    }
}
