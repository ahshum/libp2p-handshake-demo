use crate::{
    error::{self, Error},
    io::{protobuf_decode, protobuf_encode},
    payload::keys::{KeyType as KeyTypeProto, PublicKey as PublicKeyProto},
};
use std::path::Path;

pub const MULTIHASH_IDENTITY_CODE: u8 = 0x00;

// PrivateKey types
#[derive(Clone)]
pub enum PrivateKey {
    None,
    Ed25519(ed25519_dalek::SigningKey),
    RSA,
    Secp256k1,
    ECDSA,
}

impl PrivateKey {
    pub fn generate_ed25519() -> Self {
        let signing_key = ed25519_dalek::SigningKey::generate(&mut rand::rngs::OsRng);
        Self::Ed25519(signing_key)
    }

    pub fn from_ed25519_pem_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        use ed25519_dalek::pkcs8::DecodePrivateKey;

        ed25519_dalek::SigningKey::read_pkcs8_pem_file(path)
            .map_err(|_| error::parse_error())
            .map(|key| Self::Ed25519(key))
    }

    pub fn public(&self) -> PublicKey {
        match self {
            Self::Ed25519(key) => PublicKey::Ed25519(key.verifying_key()),
            _ => todo!(),
        }
    }

    pub fn sign(&self, msg: &[u8]) -> Vec<u8> {
        match self {
            Self::Ed25519(key) => {
                use ed25519_dalek::Signer;
                key.sign(msg).to_vec()
            }
            _ => todo!(),
        }
    }

    pub fn into_x25519_encoded(&self) -> Vec<u8> {
        match self {
            Self::Ed25519(key) => {
                x25519_dalek::x25519(key.to_bytes(), x25519_dalek::X25519_BASEPOINT_BYTES).to_vec()
            }
            _ => todo!(),
        }
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        match self {
            Self::Ed25519(key) => key.to_bytes().to_vec(),
            _ => todo!(),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Ed25519(key) => key.as_bytes(),
            _ => todo!(),
        }
    }
}

// PublicKey types
#[derive(Debug, PartialEq, Clone)]
pub enum PublicKey {
    None,
    Ed25519(ed25519_dalek::VerifyingKey),
    RSA,
    Secp256k1,
    ECDSA,
}

impl PublicKey {
    pub fn from_ed25519_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let mut buf = [0u8; 32];
        buf.copy_from_slice(&bytes);
        let key =
            ed25519_dalek::VerifyingKey::from_bytes(&buf).map_err(|_| error::parse_error())?;
        Ok(Self::Ed25519(key))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::Ed25519(key) => key.to_bytes().to_vec(),
            _ => todo!(),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Ed25519(key) => key.as_bytes(),
            _ => todo!(),
        }
    }

    pub fn from_protobuf_bytes(bytes: &[u8]) -> Result<Self, Error> {
        // verify multihash format
        if bytes[0] != MULTIHASH_IDENTITY_CODE && usize::from(bytes[1]) != bytes.len() - 2 {
            return Err(error::parse_error());
        }

        let pb: PublicKeyProto = protobuf_decode(&bytes[2..])?;
        match pb.Type {
            KeyTypeProto::Ed25519 => {
                let mut buf = [0u8; 32];
                buf.copy_from_slice(&pb.Data);
                // verify public key and then return
                ed25519_dalek::VerifyingKey::from_bytes(&buf)
                    .map_err(|_| error::parse_error())
                    .and_then(|key| Ok(Self::Ed25519(key)))
            }
            _ => todo!(),
        }
    }

    pub fn to_protobuf_bytes(&self) -> Result<Vec<u8>, Error> {
        match self {
            Self::Ed25519(key) => {
                let pb = PublicKeyProto {
                    Type: KeyTypeProto::Ed25519,
                    Data: key.to_bytes().to_vec(),
                };
                protobuf_encode(&pb).map_err(|_| error::parse_error())
            }
            Self::None => Err(error::missing_key()),
            _ => todo!(),
        }
    }

    pub fn verify(&self, msg: &[u8], sig: &[u8]) -> Result<(), Error> {
        match self {
            Self::Ed25519(key) => key
                .verify_strict(
                    msg,
                    &ed25519_dalek::Signature::from_slice(sig).map_err(|_| error::parse_error())?,
                )
                .map_err(|_| error::verification_failed()),
            _ => todo!(),
        }
    }
}

impl TryFrom<PeerId> for PublicKey {
    type Error = Error;

    fn try_from(peer_id: PeerId) -> Result<Self, Self::Error> {
        multibase::Base::Base58Btc
            .decode(peer_id.0)
            .map_err(|_| error::parse_error())
            .and_then(|decoded| Self::from_protobuf_bytes(&decoded))
    }
}

impl TryInto<PeerId> for PublicKey {
    type Error = Error;

    fn try_into(self) -> Result<PeerId, Self::Error> {
        match self {
            Self::Ed25519(_) => {
                let encoded = self.to_protobuf_bytes()?;
                let len: u8 = encoded.len().try_into().map_err(|_| error::parse_error())?;
                let multihash = &[vec![MULTIHASH_IDENTITY_CODE, len], encoded].concat();
                let peer_id = multibase::Base::Base58Btc.encode(multihash);
                Ok(PeerId(peer_id))
            }
            Self::None => Err(error::missing_key()),
            _ => todo!(),
        }
    }
}

// PeerId which should be generated from PublicKey
#[derive(Debug, PartialEq, Clone)]
pub struct PeerId(String);

impl PeerId {
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PUB_KEY: [u8; 32] = [
        170, 70, 186, 1, 128, 14, 51, 214, 89, 215, 83, 63, 206, 151, 242, 35, 230, 49, 126, 127,
        238, 136, 29, 146, 186, 158, 66, 210, 171, 161, 89, 179,
    ];
    const TEST_PEER_ID: &str = "12D3KooWMH42bj1zkh7wa6Yua9hzs9xbjoH63gYHitLkreXSQQu8";

    #[test]
    fn test_public_key_into_peer_id() {
        assert_eq!(
            TryInto::<PeerId>::try_into(PublicKey::Ed25519(
                ed25519_dalek::VerifyingKey::from_bytes(&self::TEST_PUB_KEY).unwrap()
            ))
            .unwrap(),
            PeerId(self::TEST_PEER_ID.to_string())
        );
    }

    #[test]
    fn test_peer_id_into_public_key() {
        assert_eq!(
            PublicKey::try_from(PeerId(self::TEST_PEER_ID.to_string())).unwrap(),
            PublicKey::Ed25519(
                ed25519_dalek::VerifyingKey::from_bytes(&self::TEST_PUB_KEY).unwrap()
            )
        );
    }
}
