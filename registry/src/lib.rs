use crate::data::config::Config;
use crate::data::store::DataStore;
use crate::data::PersistentData;
use crate::net::packets::{ClientPacket, ServerPacket};
use crate::net::types::{NetReadExt, NetWriteExt};
use crate::plugins::{PluginInfo, Plugins};
use anyhow::{bail, Context, Result};
use log::{debug, error, info, warn};
use std::borrow::Cow;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::{io, thread};

const CONFIG_PATH: &str = "config.toml";
const DATA_PATH: &str = "data.toml";

pub mod data;
pub mod net;
pub mod plugins;

pub fn run() -> Result<()> {
    let config = Config::load_or_default(CONFIG_PATH).context("failed to load the config")?;
    debug!("Using config: {config:?}");
    let config = Arc::new(config);

    let data = DataStore::load_or_default(DATA_PATH).context("failed to load the data store")?;
    debug!("Using data store: {data:?}");
    let data = Arc::new(RwLock::new(data));

    if config.save.enabled {
        let save_config = Arc::clone(&config);
        let save_data = Arc::clone(&data);
        thread::Builder::new()
            .name("save".into())
            .spawn(|| save_periodically(save_config, save_data))
            .context("failed to spawn the save thread")?;
    }

    thread::Builder::new()
        .name("listen".into())
        .spawn(|| listen(config, data))
        .context("failed to spawn the listen thread")?
        .join()
        .unwrap()
}

fn listen(config: Arc<Config>, data: Arc<RwLock<DataStore>>) -> Result<()> {
    let bind_addr = &config.server.bind_addr;
    let listener = TcpListener::bind(bind_addr)
        .with_context(|| format!("failed to bind to `{}`", bind_addr))?;
    info!(
        "Listening on {}!",
        format_socket_addr(listener.local_addr(), bind_addr)
    );

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(stream) => stream,
            Err(error) => {
                warn!("Failed to accept a connection request: {error:?}");
                continue;
            }
        };

        let formatted_addr = format_socket_addr(stream.peer_addr(), "<unknown>");
        info!("Accepted a connection request from {formatted_addr}.");

        let connection_config = Arc::clone(&config);
        let connection_data = Arc::clone(&data);
        let result = thread::Builder::new()
            .name(format!("conn/{formatted_addr}"))
            .spawn(|| Connection::new(stream).run(connection_config, connection_data))
            .context("failed to spawn the connection handle thread");
        if let Err(error) = result {
            warn!("Failed to start connection handling for {formatted_addr}: {error:?}");
        }
    }

    Ok(())
}

struct Connection {
    stream: TcpStream,
    did_handshake: bool,
    plugins: Plugins,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            did_handshake: false,
            plugins: Plugins::new(),
        }
    }

    pub fn run(&mut self, config: Arc<Config>, data: Arc<RwLock<DataStore>>) {
        let set_error_tolerance = config.server.error_tolerance >= 0;
        let mut errors = 0;
        loop {
            if let Err(error) = self.next_packet(&data) {
                warn!("Failed to handle a packet: {error:?}");
                if set_error_tolerance {
                    if errors == config.server.error_tolerance {
                        error!("Failed to handle too many packets.");
                        break;
                    }
                    errors += 1;
                }
            } else if set_error_tolerance {
                errors = 0;
            }
        }
        info!("The client will be disconnected.");
    }

    fn next_packet(&mut self, _data: &Arc<RwLock<DataStore>>) -> Result<()> {
        let packet = self
            .stream
            .read_packet()
            .context("failed to read the next packet")?;
        match packet {
            ClientPacket::Handshake { version } => {
                info!("The client is using `{version}`.");
                self.stream
                    .write_packet(&ServerPacket::Handshake)
                    .context("failed to send a handshake response")?;
                self.did_handshake = true;
            }
            _ if !self.did_handshake => bail!("received a non-handshake packet before handshake"),
            ClientPacket::SelectPlugin { name, authors } => self
                .plugins
                .select(name.clone(), || PluginInfo::from_optional_authors(authors))
                .with_context(|| format!("failed to select `{name}`"))?,
            ClientPacket::EnablePlugin(name) => self
                .plugins
                .set_enabled(&name, true)
                .with_context(|| format!("failed to enable `{name}`"))?,
            ClientPacket::DisablePlugin(name) => self
                .plugins
                .set_enabled(&name, false)
                .with_context(|| format!("failed to disable `{name}`"))?,
        }
        Ok(())
    }
}

fn save_periodically(config: Arc<Config>, data: Arc<RwLock<DataStore>>) {
    loop {
        let result = { data.write().unwrap().save(DATA_PATH) };
        if let Err(error) = result {
            error!("Failed to save: {error:?}");
        } else {
            debug!("Saved successfully.");
        }
        thread::sleep(config.save.interval);
    }
}

fn format_socket_addr(addr: io::Result<SocketAddr>, default: &str) -> Cow<str> {
    addr.map_or(Cow::Borrowed(default), |addr| Cow::Owned(addr.to_string()))
}
