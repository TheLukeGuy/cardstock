use crate::data::config::Config;
use crate::data::store::DataStore;
use crate::data::PersistentData;
use crate::net::packets::{ClientPacket, ServerPacket};
use crate::net::types::{NetReadExt, NetWriteExt, PacketOpResult};
use crate::plugins::{GlobalCommandStatus, PluginInfo, Plugins};
use anyhow::{bail, Context, Result};
use log::{debug, error, info, trace, warn, Level};
use rand::seq::SliceRandom;
use rand::Rng;
use std::borrow::Cow;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::{io, thread};

const CONFIG_PATH: &str = "config.toml";
const DATA_PATH: &str = "data.toml";

pub mod data;
pub mod net;
pub mod plugins;
pub mod suggest;

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
            .spawn(|| Connection::new(stream, connection_config, connection_data).run())
            .context("failed to spawn the connection handle thread");
        if let Err(error) = result {
            warn!("Failed to start connection handling for {formatted_addr}: {error:?}");
        }
    }

    Ok(())
}

struct Connection {
    stream: TcpStream,
    config: Arc<Config>,
    data: Arc<RwLock<DataStore>>,

    did_handshake: bool,
    plugins: Plugins,
}

impl Connection {
    pub fn new(stream: TcpStream, config: Arc<Config>, data: Arc<RwLock<DataStore>>) -> Self {
        Self {
            stream,
            config,
            data,
            did_handshake: false,
            plugins: Plugins::new(),
        }
    }

    pub fn run(&mut self) {
        let set_error_tolerance = self.config.server.error_tolerance >= 0;
        let mut errors = 0;
        loop {
            match self.next_packet() {
                Err(error) => {
                    warn!("Failed to handle a packet: {error:?}");
                    if set_error_tolerance {
                        if errors == self.config.server.error_tolerance {
                            error!("Failed to handle too many packets.");
                            break;
                        }
                        errors += 1;
                    }
                }
                Ok(PacketResult::Disconnect) => break,
                _ if set_error_tolerance => errors = 0,
                _ => {}
            }
        }

        if self.send_packet(&ServerPacket::Disconnect).is_err() {
            warn!("Failed to gracefully disconnect the client.");
        }
        info!("The connection will be dropped.");
    }

    fn next_packet(&mut self) -> Result<PacketResult> {
        let packet = self
            .stream
            .read_packet()
            .context("failed to read the next packet")?;
        let packet = match packet {
            PacketOpResult::Ok(packet) => packet,
            PacketOpResult::AppearsDisconnected => {
                warn!("The client forcefully disconnected.");
                return Ok(PacketResult::Disconnect);
            }
        };
        trace!("Received packet: {packet:?}");

        match packet {
            ClientPacket::Handshake { version } => {
                info!("The client is using `{version}`.");
                self.send_packet(&ServerPacket::Handshake {
                    ads_enabled: self.config.ads.enabled,
                })
                .context("failed to send a handshake response")?;
                self.did_handshake = true;
            }
            _ if !self.did_handshake => bail!("received a non-handshake packet before handshake"),
            ClientPacket::SelectPlugin { name, authors } => self
                .plugins
                .select(name.clone(), || PluginInfo::from_optional_authors(authors))
                .with_context(|| format!("failed to select `{name}`"))?,
            ClientPacket::EnablePlugin => self
                .plugins
                .set_enabled(true)
                .context("failed to enable the selected plugin")?,
            ClientPacket::DisablePlugin => self
                .plugins
                .set_enabled(false)
                .context("failed to disable the selected plugin")?,
            ClientPacket::RegisterCmd(name) => self
                .handle_register(name)
                .context("failed to handle command registration")?,
            ClientPacket::Disconnect => {
                info!("The client is gracefully disconnecting.");
                return Ok(PacketResult::Disconnect);
            }
        }
        Ok(PacketResult::Ok)
    }

    fn handle_register(&mut self, cmd: String) -> Result<()> {
        let owner = self.data.read().unwrap().check(&cmd);
        let current_plugin = self.plugins.selected();
        match owner {
            Some(plugin) if *plugin == current_plugin => {
                debug!("Allowing registered command `{cmd}`.");
                self.send_msg(
                    Level::Debug,
                    format!(
                        "{}, thank you for registering /{cmd}!",
                        self.plugins.current_authors()
                    ),
                )
                .context("failed to send the message packet")?;
                self.plugins
                    .register_cmd(cmd, GlobalCommandStatus::Registered)
            }
            Some(owner) => {
                let suggestions = {
                    let read_guard = self.data.read().unwrap();
                    suggest::gen(self.plugins.selected(), &cmd, |name| {
                        read_guard.check(name).is_some()
                    })
                }
                .join(", ");
                debug!("Denying command `{cmd}` and suggesting `{suggestions}`.");

                self.send_msg(
                    Level::Error,
                    format!(
                        "/{cmd} is already registered to {owner}. Please choose a different name."
                    ),
                )
                .context("failed to send the message packet")?;
                self.send_msg(
                    Level::Error,
                    format!("Try one of these instead: {suggestions}"),
                )
                .context("failed to send the suggestion message packet")?;
                self.send_packet(&ServerPacket::Deny)
                    .context("failed to send the deny packet")?;
            }
            None => {
                debug!("Allowing unregistered command `{cmd}`.");
                self.send_msg(
                    Level::Warn,
                    format!(
                        concat!(
                            "Hey, {}! Your command /{cmd} is unregistered. ",
                            "Please register it with \"/register {cmd} {}\"."
                        ),
                        self.plugins.current_authors(),
                        self.plugins.selected(),
                        cmd = cmd,
                    ),
                )
                .context("failed to send the message packet")?;
                self.plugins
                    .register_cmd(cmd, GlobalCommandStatus::Unregistered);
            }
        }
        self.send_packet(&ServerPacket::Done)
            .context("failed to send the done packet")?;
        Ok(())
    }

    pub fn send_packet(&mut self, packet: &ServerPacket) -> Result<()> {
        trace!("Sending packet: {packet:?}");
        self.stream
            .write_packet(packet)
            .with_context(|| format!("failed to write the packet ({packet:?})"))?;
        // Cardstock TODO: Handle client disconnections
        Ok(())
    }

    pub fn send_msg(&mut self, log_level: Level, msg: impl ToString) -> Result<()> {
        self.send_packet(&ServerPacket::Msg {
            log_level,
            contents: msg.to_string(),
        })
        .context("failed to send the message packet")?;

        if self.config.ads.enabled {
            let mut rng = rand::thread_rng();
            let send_ad = rng.gen_ratio(1, self.config.ads.one_in_x_chance);
            if send_ad {
                if let Some(ad) = self.config.ads.list.choose(&mut rng) {
                    debug!("Sending an ad.");
                    self.send_packet(&ServerPacket::Msg {
                        log_level: Level::Info,
                        contents: format!("Ad | {ad}"),
                    })
                    .context("failed to send the ad message packet")?;
                }
            }
        }
        Ok(())
    }
}

enum PacketResult {
    Ok,
    Disconnect,
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
