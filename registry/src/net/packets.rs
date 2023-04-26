use crate::net::types::NetReadExt;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum ClientPacket {
    Handshake { version: String },
}

impl ClientPacket {
    pub fn read(id: u8, buf: &mut impl Read) -> Result<Self> {
        if id != 0 {
            bail!("the packet ID is invalid ({id:#04x})");
        }
        let version = buf.read_string().context("failed to read the version")?;
        Ok(Self::Handshake { version })
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum ServerPacket {
    Handshake,
}

impl ServerPacket {
    pub fn write(&self, _buf: &mut impl Write) -> Result<u8> {
        Ok(0)
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum PartialPacket {
    AwaitingLen(Vec<u8>),
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
        Self::AwaitingLen(Vec::with_capacity(2))
    }

    pub fn next(mut self, byte: u8) -> Self {
        match self {
            Self::AwaitingLen(ref mut packet) => {
                packet.push(byte);
                if packet.len() == 2 {
                    let len = u16::from_be_bytes(packet[..].try_into().unwrap()).into();
                    Self::Incomplete {
                        expected: len,
                        id: None,
                        packet: Vec::with_capacity(len),
                    }
                } else {
                    self
                }
            }
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
