use crate::{editor::keymap::KeyBinding, server::Server};
use serde::Deserialize;
use std::net::Ipv4Addr;

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "Server::default_host")]
    pub host: Ipv4Addr,

    #[serde(default = "Server::default_port")]
    pub port: u16,

    #[serde(default)]
    pub keymap: Vec<KeyBinding>,
}
