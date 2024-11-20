use super::upgrade::{ProtocolId, UpgradeOutbound};
use crate::{
    error::{self, Error},
    identity::{PeerId, PrivateKey, PublicKey},
    io,
    payload::{
        keys::{KeyType, PublicKey as PublicKeyPayload},
        noise::NoiseHandshakePayload,
    },
};
use asynchronous_codec::{Framed, FramedParts};
use futures::{AsyncRead, AsyncWrite, Future, FutureExt, SinkExt, StreamExt};
use std::{boxed::Box, pin::Pin};

const PROTOCOL_NOISE: &str = "/noise";
const NOISE_PROTOCOL_NAME: &str = "Noise_XX_25519_ChaChaPoly_SHA256";
const STATIC_KEY_PREFIX: &str = "noise-libp2p-static-key:";

// Noise
pub struct Noise {
    private_key: PrivateKey,
    static_key: PrivateKey,
}

impl Noise {
    pub fn new(private_key: PrivateKey) -> Self {
        Self {
            private_key,
            static_key: PrivateKey::generate_ed25519(),
        }
    }
}

impl ProtocolId for Noise {
    fn protocol_id() -> &'static str {
        PROTOCOL_NOISE
    }
}

impl<'a, T> UpgradeOutbound<'a, T> for Noise
where
    T: AsyncRead + AsyncWrite + Unpin + Send + 'a,
{
    type Output = (io::NoiseUpgradedStream<T>, PeerId);
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Output, Self::Error>> + 'a>>;

    fn upgrade_outbound(self, stream: T) -> Self::Future {
        async move {
            let Self {
                static_key,
                private_key,
            } = self;
            let noise_state = snow::Builder::new(
                NOISE_PROTOCOL_NAME
                    .parse()
                    .map_err(|_| error::other("noise protocol parse failed"))?,
            )
            .local_private_key(static_key.as_bytes())
            .build_initiator()
            .map_err(|_| error::other("noise builder failed"))?;

            let mut framed = Framed::new(stream, io::NoiseCodec::new(noise_state));

            // stage 1
            framed.send(&NoiseHandshakePayload::default()).await?;
            log::info!("noise handshake stage 1 complete");

            // stage 2
            let remote_payload = match framed.next().await {
                Some(Ok(payload)) => payload,
                _ => {
                    log::debug!("remote payload invalid");
                    return Err(error::verification_failed());
                }
            };
            log::debug!("remote payload recv");

            let remote_pub = if let Some(rawtext) = remote_payload.identity_key {
                let payload: PublicKeyPayload = io::protobuf_decode(&rawtext)?;
                match payload.Type {
                    KeyType::Ed25519 => PublicKey::from_ed25519_bytes(&payload.Data)?,
                    _ => todo!(),
                }
            } else {
                log::debug!("remote public key missing");
                return Err(error::verification_failed());
            };
            log::debug!("remote public key get");

            let remote_sig = if let Some(buf) = remote_payload.identity_sig {
                // only ED25519 can reach here
                buf
            } else {
                log::debug!("remote signature missing");
                return Err(error::verification_failed());
            };
            log::debug!("remote signature get");

            let remote_static = match framed.codec().state().get_remote_static() {
                Some(key) => key,
                _ => return Err(error::verification_failed()),
            };
            log::debug!("remote static key get");

            remote_pub
                .verify(
                    &[STATIC_KEY_PREFIX.as_bytes(), remote_static].concat(),
                    &remote_sig,
                )
                .map_err(|_| {
                    log::debug!("remote signature invalid");
                    error::verification_failed()
                })?;
            log::debug!("remote static key verified");
            log::info!("noise handshake stage 2 complete");

            // stage 3
            let my_sig = private_key.sign(
                &[
                    STATIC_KEY_PREFIX.as_bytes(),
                    &static_key.into_x25519_encoded(),
                ]
                .concat(),
            );
            log::debug!("my signature get");

            let my_payload = NoiseHandshakePayload {
                identity_key: Some(io::protobuf_encode(&PublicKeyPayload {
                    Type: KeyType::Ed25519,
                    Data: private_key.public().to_bytes(),
                })?),
                identity_sig: Some(my_sig),
                extensions: None,
            };
            framed.send(&my_payload).await?;
            log::debug!("my signature send");
            log::info!("noise handshake stage 3 complete");

            // prepare output
            let FramedParts { io, codec, .. } = framed.into_parts();
            let noise_transport = codec.into_transport()?;
            let upgraded = io::NoiseUpgradedStream::new(io, noise_transport);

            let peer_id: PeerId = remote_pub.try_into()?;
            log::info!(
                "handshake complete my peer_id: {:?} and remote peer_id: {:?}",
                TryInto::<PeerId>::try_into(private_key.public()).unwrap(),
                peer_id
            );

            Ok((upgraded, peer_id))
        }
        .boxed()
    }
}
