use crate::net::types::{NetReadExt, NetWriteExt};
use anyhow::{bail, Context, Result};
use log::Level;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum ClientPacket {
    Handshake {
        version: String,
    },
    SelectPlugin {
        name: String,
        authors: Option<String>,
    },
    EnablePlugin,
    DisablePlugin,
    RegisterCmd(String),
}

impl ClientPacket {
    pub fn read(id: u8, buf: &mut impl Read) -> Result<Self> {
        let packet = match id {
            0x00 => {
                let version = buf.read_string().context("failed to read the version")?;
                Self::Handshake { version }
            }
            0x01 => {
                let name = buf
                    .read_string()
                    .context("failed to read the plugin name")?;
                let authors = buf
                    .read_option(NetReadExt::read_string)
                    .context("failed to read the plugin authors")?;
                Self::SelectPlugin { name, authors }
            }
            0x02 => Self::EnablePlugin,
            0x03 => Self::DisablePlugin,
            0x04 => {
                let name = buf
                    .read_string()
                    .context("failed to read the command name")?;
                Self::RegisterCmd(name)
            }
            _ => bail!("the packet ID is invalid ({id:#04x})"),
        };
        Ok(packet)
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum ServerPacket {
    Handshake { ads_enabled: bool },
    Msg { log_level: Level, contents: String },
    Deny,
    Done,
}

impl ServerPacket {
    pub fn write(&self, buf: &mut impl Write) -> Result<u8> {
        let id = match self {
            ServerPacket::Handshake { ads_enabled } => {
                buf.write_bool(*ads_enabled)
                    .context("failed to write the ad indicator")?;
                0x00
            }
            ServerPacket::Msg {
                log_level,
                contents,
            } => {
                buf.write_log_level(*log_level)
                    .context("failed to write the log level")?;
                buf.write_str(contents)
                    .context("failed to write the contents")?;
                0x01
            }
            ServerPacket::Deny => 0x02,
            ServerPacket::Done => 0x03,
        };
        Ok(id)
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum PartialPacket {
    AwaitingLen(Option<u8>),
    Incomplete {
        expected: usize,
        id: Option<u8>,
        packet: Vec<u8>,
    },
    Complete {
        id: u8,
        packet: Vec<u8>,
    },
}

impl PartialPacket {
    pub fn new() -> Self {
        Self::AwaitingLen(None)
    }

    pub fn next(self, byte: u8) -> Self {
        match self {
            Self::AwaitingLen(Some(packet)) => {
                let len = u16::from_be_bytes([packet, byte]).into();
                Self::Incomplete {
                    expected: len,
                    id: None,
                    packet: Vec::with_capacity(len),
                }
            }
            Self::AwaitingLen(_) => Self::AwaitingLen(Some(byte)),
            Self::Incomplete {
                expected,
                id: Some(id),
                mut packet,
            } => {
                packet.push(byte);
                if packet.len() == expected {
                    Self::Complete { id, packet }
                } else {
                    Self::Incomplete {
                        expected,
                        id: Some(id),
                        packet,
                    }
                }
            }
            Self::Incomplete {
                expected,
                id: _,
                packet,
            } if expected == 0 => Self::Complete { id: byte, packet },
            Self::Incomplete {
                expected,
                id: _,
                packet,
            } => Self::Incomplete {
                expected,
                id: Some(byte),
                packet,
            },
            Self::Complete { .. } => panic!("`next` called on a complete packet"),
        }
    }
}

impl Default for PartialPacket {
    fn default() -> Self {
        Self::new()
    }
}
