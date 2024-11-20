// Automatically generated rust module for 'noise.proto' file

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt_skip)]


use quick_protobuf::{MessageInfo, MessageRead, MessageWrite, BytesReader, Writer, WriterBackend, Result};
use quick_protobuf::sizeofs::*;
use super::*;

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct NoiseExtensions {
    pub webtransport_certhashes: Vec<Vec<u8>>,
    pub stream_muxers: Vec<Vec<u8>>,
}

impl<'a> MessageRead<'a> for NoiseExtensions {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.webtransport_certhashes.push(r.read_bytes(bytes).map(Vec::from)?),
                Ok(18) => msg.stream_muxers.push(r.read_bytes(bytes).map(Vec::from)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for NoiseExtensions {
    fn get_size(&self) -> usize {
        0
        + self.webtransport_certhashes.iter().map(|s| 1 + sizeof_len((s).len())).sum::<usize>()
        + self.stream_muxers.iter().map(|s| 1 + sizeof_len((s).len())).sum::<usize>()
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        for s in &self.webtransport_certhashes { w.write_with_tag(10, |w| w.write_bytes(&**s))?; }
        for s in &self.stream_muxers { w.write_with_tag(18, |w| w.write_bytes(&**s))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct NoiseHandshakePayload {
    pub identity_key: Option<Vec<u8>>,
    pub identity_sig: Option<Vec<u8>>,
    pub extensions: Option<NoiseExtensions>,
}

impl<'a> MessageRead<'a> for NoiseHandshakePayload {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.identity_key = Some(r.read_bytes(bytes).map(Vec::from)?),
                Ok(18) => msg.identity_sig = Some(r.read_bytes(bytes).map(Vec::from)?),
                Ok(34) => msg.extensions = Some(r.read_message::<NoiseExtensions>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for NoiseHandshakePayload {
    fn get_size(&self) -> usize {
        0
        + self.identity_key.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.identity_sig.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.extensions.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.identity_key { w.write_with_tag(10, |w| w.write_bytes(&**s))?; }
        if let Some(ref s) = self.identity_sig { w.write_with_tag(18, |w| w.write_bytes(&**s))?; }
        if let Some(ref s) = self.extensions { w.write_with_tag(34, |w| w.write_message(s))?; }
        Ok(())
    }
}

