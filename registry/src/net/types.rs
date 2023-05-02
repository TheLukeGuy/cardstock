use crate::net::packets::{ClientPacket, PartialPacket, ServerPacket};
use anyhow::{bail, Context, Result};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use log::Level;
use std::io;
use std::io::{ErrorKind, Read, Write};

pub trait NetReadExt: Read {
    fn read_packet(&mut self) -> Result<PacketOpResult<ClientPacket>> {
        let mut partial = PartialPacket::new();
        loop {
            let next = match self.read_u8() {
                Ok(next) => next,
                Err(error) => {
                    return PacketOpResult::from_io_error(error)
                        .context("failed to read the next byte");
                }
            };
            match partial.next(next) {
                PartialPacket::Complete { id, packet } => {
                    let packet = ClientPacket::read(id, &mut &packet[..])?;
                    break Ok(PacketOpResult::Ok(packet));
                }
                p => partial = p,
            }
        }
    }

    fn read_option<T>(&mut self, read: impl FnOnce(&mut Self) -> Result<T>) -> Result<Option<T>> {
        let present = self
            .read_bool()
            .context("failed to read the presence indicator")?;
        if present {
            Ok(Some(read(self)?))
        } else {
            Ok(None)
        }
    }

    fn read_bool(&mut self) -> Result<bool> {
        let byte = self.read_u8().context("failed to read the boolean byte")?;
        Ok(byte != 0)
    }

    fn read_string(&mut self) -> Result<String> {
        let len = self
            .read_u16::<BigEndian>()
            .context("failed to read the string length")?
            .into();
        let mut buf = vec![0; len];
        self.read_exact(&mut buf)
            .context("failed to read the string contents")?;
        String::from_utf8(buf).context("the string is malformed")
    }

    fn read_log_level(&mut self) -> Result<Level> {
        let byte = self
            .read_u8()
            .context("failed to read the log level byte")?;
        let level = match byte {
            4 => Level::Error,
            3 => Level::Warn,
            2 => Level::Info,
            1 => Level::Debug,
            0 => Level::Trace,
            invalid => bail!("invalid log level ({invalid})"),
        };
        Ok(level)
    }
}

impl<R> NetReadExt for R where R: Read + ?Sized {}

pub trait NetWriteExt: Write {
    fn write_packet(&mut self, packet: &ServerPacket) -> Result<PacketOpResult<()>> {
        let mut payload = Vec::with_capacity(1024);
        let id = packet
            .write(&mut payload)
            .context("failed to write the packet payload to a temporary buffer")?;
        let len = payload
            .len()
            .try_into()
            .context("the packet length doesn't fit in a u16")?;

        let mut buf = Vec::with_capacity(2 + 1 + payload.len());
        buf.write_u16::<BigEndian>(len)
            .context("failed to write the packet length")?;
        buf.write_u8(id).context("failed to write the packet ID")?;
        buf.write_all(&payload)
            .context("failed to write the packet payload")?;

        if let Err(error) = self.write_all(&buf) {
            PacketOpResult::from_io_error(error).context("failed to write the full packet")
        } else {
            Ok(PacketOpResult::Ok(()))
        }
    }

    fn write_option<T>(
        &mut self,
        o: Option<T>,
        write: impl FnOnce(&mut Self, T) -> Result<()>,
    ) -> Result<()> {
        let mut write_presence = |present| {
            self.write_bool(present)
                .context("failed to write the presence indicator")
        };
        if let Some(v) = o {
            write_presence(true)?;
            write(self, v)
        } else {
            write_presence(false)
        }
    }

    fn write_bool(&mut self, b: bool) -> Result<()> {
        self.write_u8(if b { 1 } else { 0 })
            .context("failed to write the boolean byte")
    }

    fn write_str(&mut self, s: &str) -> Result<()> {
        let bytes = s.as_bytes();
        let len = bytes
            .len()
            .try_into()
            .context("the string length doesn't fit in a u16")?;
        self.write_u16::<BigEndian>(len)
            .context("failed to write the string length")?;
        self.write_all(bytes)
            .context("failed to write the string contents")
    }

    fn write_log_level(&mut self, level: Level) -> Result<()> {
        let byte = match level {
            Level::Error => 4,
            Level::Warn => 3,
            Level::Info => 2,
            Level::Debug => 1,
            Level::Trace => 0,
        };
        self.write_u8(byte)
            .context("failed to write the log level byte")
    }
}

impl<W> NetWriteExt for W where W: Write + ?Sized {}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum PacketOpResult<T> {
    Ok(T),
    AppearsDisconnected,
}

impl<T> PacketOpResult<T> {
    pub fn from_io_error(error: io::Error) -> Result<Self> {
        if let ErrorKind::ConnectionAborted
        | ErrorKind::ConnectionReset
        | ErrorKind::UnexpectedEof = error.kind()
        {
            Ok(Self::AppearsDisconnected)
        } else {
            Err(error.into())
        }
    }
}
