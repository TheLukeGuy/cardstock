use crate::data::config::Config;
use crate::data::store::DataStore;
use crate::data::PersistentData;
use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use std::borrow::Cow;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::{io, thread};

const CONFIG_PATH: &str = "config.toml";
const DATA_PATH: &str = "data.toml";

pub mod data;

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
