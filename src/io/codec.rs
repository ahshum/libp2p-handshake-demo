use crate::error::{self, Error};
use asynchronous_codec::{Decoder, Encoder};
use bytes::BytesMut;

// U16LengthCodec
pub struct U16LengthCodec {}

impl U16LengthCodec {
    pub fn new() -> Self {
        Self {}
    }
}

impl Encoder for U16LengthCodec {
    type Item<'a> = &'a [u8];
    type Error = Error;

    fn encode(&mut self, item: Self::Item<'_>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let prefix = u16::try_from(item.len()).map_err(|_| error::encode_error())?;
        dst.resize(item.len() + 2, 0);
        dst[..2].copy_from_slice(&prefix.to_be_bytes());
        dst[2..].copy_from_slice(item);
        Ok(())
    }
}

impl Decoder for U16LengthCodec {
    type Item = Vec<u8>;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 2 {
            return Ok(None);
        }
        let mut prefix_buf = [0u8; 2];
        prefix_buf.copy_from_slice(&src[..2]);
        let prefix = usize::from(u16::from_be_bytes(prefix_buf));
        if src.len() < prefix + 2 {
            return Ok(None);
        }
        let msg = src.split_to(prefix + 2);
        Ok(Some(msg[2..].to_vec()))
    }
}

// U8LengthLineCodec
pub struct U8LengthLineCodec {}

impl U8LengthLineCodec {
    pub fn new() -> Self {
        Self {}
    }
}

impl Encoder for U8LengthLineCodec {
    type Item<'a> = &'a [u8];
    type Error = Error;

    fn encode(&mut self, item: Self::Item<'_>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let len = item.len() + 1;
        let prefix = u8::try_from(len).map_err(|_| error::encode_error())?;
        dst.resize(len + 1, 0);
        dst[0] = prefix;
        dst[1..len].copy_from_slice(item);
        dst[len] = b'\n';
        Ok(())
    }
}

impl Decoder for U8LengthLineCodec {
    type Item = Vec<u8>;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() == 0 {
            return Ok(None);
        }
        let prefix = usize::from(src[0]);
        if src.len() < prefix + 1 {
            return Ok(None);
        }
        if src[prefix] != b'\n' {
            return Err(error::message_malformed());
        }
        let msg = src.split_to(prefix + 1);
        Ok(Some(msg[1..prefix].to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use bytes::BytesMut;

    #[test]
    fn test_u16_length_codec() -> Result<(), Error> {
        let mut codec = U16LengthCodec::new();

        let mut buf = BytesMut::new();
        codec.encode("abc".as_bytes(), &mut buf)?;
        assert_eq!(&buf[..], b"\x00\x03abc");
        let dec = codec.decode(&mut buf)?;
        assert!(dec.is_some());
        assert_eq!(dec.unwrap(), b"abc".to_vec());
        Ok(())
    }

    #[test]
    fn test_u8_length_line_codec() -> Result<(), Error> {
        let mut codec = U8LengthLineCodec::new();

        let mut buf = BytesMut::new();
        codec.encode("abc".as_bytes(), &mut buf)?;
        assert_eq!(&buf[..], b"\x04abc\n");
        let dec = codec.decode(&mut buf)?;
        assert!(dec.is_some());
        assert_eq!(dec.unwrap(), b"abc".to_vec());
        Ok(())
    }
}
