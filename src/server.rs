use crate::{
    buffer_map::BufferMap,
    config::Config,
    view::View,
    window::{Window, WindowCrosstermEvent},
    window_map::WindowMap,
};
use anyhow::{Context, Error as AnyhowError};
use camino::{Utf8Path, Utf8PathBuf};
use clap::Args;
use mkutils::{Output, PointU16, Screen, Socket, Utils};
use serde::{Deserialize, Serialize};
use std::{io::Error as IoError, ops::ControlFlow, time::Duration};
use tokio::net::{UnixListener, UnixStream, unix::SocketAddr};
use ulid::Ulid;

#[derive(Args, Default, Deserialize, Serialize)]
pub struct Connect {
    paths: Vec<Utf8PathBuf>,

    #[arg(skip = Screen::size().unwrap())]
    size: PointU16,
}

pub struct Server {
    config: Config,
    buffer_map: BufferMap,
    window_map: WindowMap,
}

impl Server {
    const CONTEXT_TOP_LEVEL: &str = "error in server top-level loop";
    const CONTEXT_NEW_VIEW: &str = "unable to create new view";
    const DEFAULT_CONFIG_YAML_STR: &str = std::include_str!("default-config.yaml");
    const RENDER_PERIOD_DURATION: Duration = Duration::from_millis(5);

    fn new(config: Config) -> Self {
        let buffer_map = BufferMap::empty();
        let window_map = WindowMap::empty();

        Self {
            config,
            buffer_map,
            window_map,
        }
    }

    pub fn socket_filepath() -> &'static Utf8Path {
        Utf8Path::new("/tmp/ftg.socket")
    }

    fn config(config_filepath: Option<&Utf8Path>) -> Result<Config, AnyhowError> {
        if let Some(config_filepath) = config_filepath {
            config_filepath
                .open()?
                .buf_reader()
                .to_value_from_json_reader::<Config>()?
        } else {
            Self::DEFAULT_CONFIG_YAML_STR.to_value_from_yaml_slice()?
        }
        .ok()
    }

    async fn new_view(&mut self, filepath: Utf8PathBuf) -> Result<View, AnyhowError> {
        let buffer_id = self.buffer_map.get_or_insert_from_filepath(&filepath).await?;
        let view = View::new(buffer_id, filepath.some());

        view.ok()
    }

    async fn new_window(&mut self, socket: Socket, connect: Connect) -> Result<Window, IoError> {
        let mut window = Window::new(socket, connect.size, &self.config.key_map)?;

        for path in connect.paths {
            match self.new_view(path).await {
                Ok(view) => window.add_view(view),
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
        let status = anyhow_res.into_status();

        self.window_map.remove(window_id);

        mkutils::trace!(level = status.level(), %status, "terminated client");
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
        let ControlFlow::Break(anyhow_res) = anyhow_output.into_control_flow() else { return ().ok() };

        self.close_window(&window_crossterm_event.window_id, anyhow_res);

        ().ok()
    }

    async fn on_tick(&mut self) {
        let window_outputs = self.window_map.on_tick(&self.config, &mut self.buffer_map).await;

        for window_output in window_outputs {
            let Some(anyhow_res) = window_output.output.into_end() else { continue };

            self.close_window(&window_output.window_id, anyhow_res);
        }
    }

    #[tracing::instrument(err)]
    pub async fn serve(config_filepath: Option<&Utf8Path>) -> Result<(), AnyhowError> {
        let socket_filepath = Self::socket_filepath();
        let config = Self::config(config_filepath)?;
        let mut server = Self::new(config);
        let mut render_interval = Self::RENDER_PERIOD_DURATION.into_interval();

        socket_filepath.remove_file().mem_drop();

        let unix_listener = UnixListener::bind(socket_filepath)?;

        loop {
            let unit_res = tokio::select! {
                pair_res = unix_listener.accept() => server.on_unix_stream(pair_res).await,
                window_crossterm_event = server.window_map.next_window_crossterm_event() => {
                    server.on_window_crossterm_event(window_crossterm_event).await
                },
                _instant = render_interval.tick() => server.on_tick().await.ok(),
            };

            unit_res.context(Self::CONTEXT_TOP_LEVEL).log_if_error().mem_drop();
        }
    }
}
