use crate::error::{self, Error};

// protobuf encode
pub fn protobuf_encode<T>(payload: &T) -> Result<Vec<u8>, Error>
where
    T: quick_protobuf::MessageWrite,
{
    let mut buf = Vec::with_capacity(payload.get_size());
    let mut writer = quick_protobuf::Writer::new(&mut buf);
    payload
        .write_message(&mut writer)
        .map_err(|_| error::encode_error())?;
    Ok(buf)
}

// protobuf decode
pub fn protobuf_decode<'a, T>(buf: &'a [u8]) -> Result<T, Error>
where
    T: quick_protobuf::MessageRead<'a>,
{
    let mut reader = quick_protobuf::BytesReader::from_bytes(buf);
    T::from_reader(&mut reader, buf).map_err(|_| error::decode_error())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::payload::noise::NoiseHandshakePayload;

    #[test]
    fn test_protobuf_encode() {
        assert_eq!(
            protobuf_encode(&NoiseHandshakePayload::default()).unwrap(),
            vec![]
        );
        assert_eq!(
            protobuf_encode(&NoiseHandshakePayload {
                identity_key: Some(Vec::new()),
                identity_sig: Some(Vec::new()),
                extensions: None,
            })
            .unwrap(),
            vec![0x0a, 0x00, 0x12, 0x00]
        );
    }

    #[test]
    fn test_protobuf_decode() {
        assert_eq!(
            protobuf_decode::<'_, NoiseHandshakePayload>(&[]).unwrap(),
            NoiseHandshakePayload {
                identity_key: None,
                identity_sig: None,
                extensions: None,
            }
        );
        assert_eq!(
            protobuf_decode::<'_, NoiseHandshakePayload>(&[0x0a, 0x00, 0x12, 0x00]).unwrap(),
            NoiseHandshakePayload {
                identity_key: Some(Vec::new()),
                identity_sig: Some(Vec::new()),
                extensions: None,
            }
        );
    }
}
