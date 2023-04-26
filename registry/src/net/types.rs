use crate::net::packets::{ClientPacket, PartialPacket, ServerPacket};
use anyhow::{Context, Result};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

pub trait NetReadExt: Read {
    fn read_packet(&mut self) -> Result<ClientPacket> {
        let mut partial = PartialPacket::new();
        loop {
            let next = self.read_u8().context("failed to read the next byte")?;
            match partial.next(next) {
                PartialPacket::Complete { id, packet } => {
                    break ClientPacket::read(id, &mut &packet[..])
                }
                p => partial = p,
            }
        }
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
}

impl<R> NetReadExt for R where R: Read + ?Sized {}

pub trait NetWriteExt: Write {
    fn write_packet(&mut self, packet: &ServerPacket) -> Result<()> {
        let mut buf = Vec::with_capacity(1024);
        let id = packet
            .write(&mut buf)
            .context("failed to write the packet to a temporary buffer")?;
        let len = buf
            .len()
            .try_into()
            .context("the packet length doesn't fit in a u16")?;

        self.write_u16::<BigEndian>(len)
            .context("failed to write the packet length")?;
        self.write_u8(id).context("failed to write the packet ID")?;
        self.write_all(&buf)
            .context("failed to write the packet payload")
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
}

impl<W> NetWriteExt for W where W: Write + ?Sized {}
