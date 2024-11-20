use super::{protobuf_decode, protobuf_encode, U16LengthCodec};
use crate::{
    error::{self, Error},
    payload::noise::NoiseHandshakePayload,
};
use asynchronous_codec::{Decoder, Encoder};
use bytes::BytesMut;
use futures::{
    task::{Context, Poll},
    AsyncRead, AsyncWrite,
};
use snow::{HandshakeState, TransportState};
use std::{cmp::min, pin::Pin};

const MAX_BUFFER_SIZE: usize = 65535 + 2;

// NoiseCodec
pub struct NoiseCodec<T> {
    state: T,
    prefixer: U16LengthCodec,
}

impl<T> NoiseCodec<T> {
    pub fn new(state: T) -> Self {
        Self {
            state,
            prefixer: U16LengthCodec::new(),
        }
    }

    pub fn state(&self) -> &T {
        &self.state
    }
}

// NoiseCodec for HandshakeState
impl NoiseCodec<HandshakeState> {
    pub fn into_transport(self) -> Result<NoiseCodec<TransportState>, Error> {
        let NoiseCodec { state, .. } = self;
        match state.into_transport_mode() {
            Ok(transport) => Ok(NoiseCodec::new(transport)),
            Err(err) => {
                log::debug!("noise into transport state failed, {:?}", err);
                Err(error::other("noise transport mode"))
            }
        }
    }
}

impl Encoder for NoiseCodec<HandshakeState> {
    type Item<'a> = &'a NoiseHandshakePayload;
    type Error = Error;

    fn encode(&mut self, item: Self::Item<'_>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        log::debug!("NoiseCodec<HandshakeState>::encode");
        let mut enc_buf = [0u8; MAX_BUFFER_SIZE];
        let enc_len = self
            .state
            .write_message(&protobuf_encode(item)?, &mut enc_buf)
            .map_err(|_| error::encode_error())?;
        log::debug!(
            "NoiseCodec<HandshakeState>::encode - encoded - enc_len:{:?}",
            enc_len
        );
        self.prefixer.encode(&enc_buf[..enc_len], dst)
    }
}

impl Decoder for NoiseCodec<HandshakeState> {
    type Item = NoiseHandshakePayload;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        log::debug!(
            "NoiseCodec<HandshakeState>::decode - src_len:{:?}",
            src.len()
        );
        let msg = match self.prefixer.decode(src) {
            Ok(Some(msg)) => msg,
            Ok(None) => return Ok(None),
            Err(err) => return Err(err),
        };
        if msg.len() == 0 {
            return Ok(None);
        }
        let mut dec_buf = [0u8; MAX_BUFFER_SIZE];
        let dec_len = self
            .state
            .read_message(&msg, &mut dec_buf)
            .map_err(|_| error::decode_error())?;
        let item: NoiseHandshakePayload = protobuf_decode(&dec_buf[..dec_len])?;
        log::debug!(
            "NoiseCodec<HandshakeState>::decode - decoded - dec_len:{:?}",
            dec_len
        );
        // discard consumed buffer
        Ok(Some(item))
    }
}

// NoiseCodec for TransportState
impl Encoder for NoiseCodec<TransportState> {
    type Item<'a> = &'a [u8];
    type Error = Error;

    fn encode(&mut self, item: Self::Item<'_>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let mut enc_buf = [0u8; MAX_BUFFER_SIZE];
        let enc_len = self
            .state
            .write_message(item, &mut enc_buf)
            .map_err(|_| error::encode_error())?;
        log::debug!(
            "NoiseCodec<TransportState>::decode - encoded - enc_len:{:?}",
            enc_len
        );
        self.prefixer.encode(&enc_buf[..enc_len], dst)
    }
}

impl Decoder for NoiseCodec<TransportState> {
    type Item = Vec<u8>;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let msg = match self.prefixer.decode(src) {
            Ok(Some(msg)) => msg,
            Ok(None) => return Ok(None),
            Err(err) => return Err(err),
        };
        if msg.len() == 0 {
            return Ok(None);
        }
        let mut dec_buf = [0u8; MAX_BUFFER_SIZE];
        let dec_len = self
            .state
            .read_message(&msg, &mut dec_buf)
            .map_err(|_| error::decode_error())?;
        log::debug!(
            "NoiseCodec<TransportState>::decode - decoded - dec_len:{:?}",
            dec_len
        );
        // discard consumed buffer
        Ok(Some(dec_buf[..dec_len].to_vec()))
    }
}

// NoiseUpgradedStream
pub struct NoiseUpgradedStream<T> {
    io: T,
    codec: NoiseCodec<TransportState>,
    read_buffer: BytesMut,
    dec_buffer: BytesMut,
}

impl<T> NoiseUpgradedStream<T> {
    pub fn new(io: T, codec: NoiseCodec<TransportState>) -> Self {
        Self {
            io,
            codec,
            read_buffer: BytesMut::new(),
            dec_buffer: BytesMut::new(),
        }
    }
}

impl<T> AsyncRead for NoiseUpgradedStream<T>
where
    T: AsyncRead + Unpin,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, futures::io::Error>> {
        let this = self.get_mut();
        let mut io = Pin::new(&mut this.io);
        loop {
            // exhaust the buffer
            if this.dec_buffer.len() > 0 {
                let n = min(buf.len(), this.dec_buffer.len());
                buf[..n].copy_from_slice(&this.dec_buffer.split_to(n));
                log::debug!("NoiseUpgradedStream::poll_read - copy - n:{:?}", n);
                return Poll::Ready(Ok(n));
            }
            // read from io
            let mut read_buf = [0u8; MAX_BUFFER_SIZE];
            match io.as_mut().poll_read(cx, &mut read_buf) {
                Poll::Ready(Ok(read_len)) => {
                    this.read_buffer.extend_from_slice(&read_buf[..read_len])
                }
                Poll::Ready(Err(err)) => return Poll::Ready(Err(err)),
                Poll::Pending => return Poll::Pending,
            }
            log::debug!(
                "NoiseUpgradedStream::poll_read - read - read_len:{:?}",
                this.read_buffer.len()
            );
            // decode by codec
            match this.codec.decode(&mut this.read_buffer) {
                Ok(Some(msg)) => this.dec_buffer.extend_from_slice(&msg),
                Ok(None) => return Poll::Ready(Ok(0)),
                Err(err) => return Poll::Ready(Err(futures::io::Error::other(err))),
            }
        }
    }
}

impl<T> AsyncWrite for NoiseUpgradedStream<T>
where
    T: AsyncWrite + Unpin,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, futures::io::Error>> {
        let this = self.get_mut();
        let mut io = Pin::new(&mut this.io);
        log::debug!(
            "NoiseUpgradedStream::poll_write - write - buf_len:{:?}",
            buf.len()
        );
        let mut write_buf = BytesMut::new();
        // encode buf into noise format & U16 prefixed
        if let Err(err) = this.codec.encode(buf, &mut write_buf) {
            return Poll::Ready(Err(futures::io::Error::other(err)));
        }
        log::debug!(
            "NoiseUpgradedStream::poll_write - encoded - enc_len:{:?}",
            write_buf.len()
        );
        // write entire buffer to the stream
        if write_buf.len() > 0 {
            match io.as_mut().poll_write(cx, &write_buf.split()) {
                // return the len of buf instead of write_buf, due to
                // Framed will verify the lengths of buf and size returned
                Poll::Ready(Ok(_)) => return Poll::Ready(Ok(buf.len())),
                res => return res,
            }
        }
        Poll::Ready(Ok(0))
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), futures::io::Error>> {
        Pin::new(&mut self.get_mut().io).poll_flush(cx)
    }

    fn poll_close(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), futures::io::Error>> {
        Pin::new(&mut self.get_mut().io).poll_close(cx)
    }
}
