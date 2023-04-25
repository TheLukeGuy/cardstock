use crate::data::config::Config;
use crate::data::store::DataStore;
use crate::data::PersistentData;
use anyhow::{Context, Result};
use log::{debug, info, warn};
use std::borrow::Cow;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::{io, thread};

pub mod data;

pub fn run() -> Result<()> {
    let config = Config::load_or_default("config.toml").context("failed to load the config")?;
    debug!("Using config: {config:?}");

    let data = DataStore::load_or_default("data.toml").context("failed to load the data store")?;
    debug!("Using data store: {data:?}");
    let data = Arc::new(RwLock::new(data));

    thread::Builder::new()
        .name("listen".into())
        .spawn(|| listen(config, data))
        .context("failed to spawn the listen thread")?
        .join()
        .unwrap()
}

fn listen(config: Config, data: Arc<RwLock<DataStore>>) -> Result<()> {
    let listener = TcpListener::bind(&config.bind_addr)
        .with_context(|| format!("failed to bind to `{}`", config.bind_addr))?;
    info!(
        "Listening on {}!",
        format_socket_addr(listener.local_addr(), &config.bind_addr)
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
        info!("Accepted a connection request from {formatted_addr}.",);

        let connection_data = data.clone();
        let result = thread::Builder::new()
            .name(format!("conn/{formatted_addr}"))
            .spawn(|| handle_connection(stream, connection_data))
            .context("failed to spawn the connection handle thread");
        if let Err(error) = result {
            warn!("Failed to start connection handling for {formatted_addr}: {error:?}");
        }
    }

    Ok(())
}

fn handle_connection(_stream: TcpStream, _data: Arc<RwLock<DataStore>>) {}

fn format_socket_addr(addr: io::Result<SocketAddr>, default: &str) -> Cow<str> {
    addr.map_or(Cow::Borrowed(default), |addr| Cow::Owned(addr.to_string()))
}
