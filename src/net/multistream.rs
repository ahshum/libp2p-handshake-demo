use super::{ProtocolId, UpgradeOutbound, PROTOCOL_UNSUPPORTED};
use crate::{
    error::{self, Error},
    io,
};
use asynchronous_codec::Framed;
use futures::{AsyncRead, AsyncWrite, Future, FutureExt, SinkExt, StreamExt};
use std::{boxed::Box, pin::Pin};

const PROTOCOL_MULTISTREAM: &str = "/multistream/1.0.0";

// Multistream
pub struct Multistream {
    initial_protocols: Vec<Vec<u8>>,
}

impl Multistream {
    pub fn new(initial_protocols: Vec<Vec<u8>>) -> Self {
        Self { initial_protocols }
    }
}

impl ProtocolId for Multistream {
    fn protocol_id() -> &'static str {
        PROTOCOL_MULTISTREAM
    }
}

impl<'a, T> UpgradeOutbound<'a, T> for Multistream
where
    T: AsyncRead + AsyncWrite + Unpin + Send + 'a,
{
    type Output = (T, Vec<u8>);
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Output, Self::Error>> + 'a>>;

    fn upgrade_outbound(self, stream: T) -> Self::Future {
        async move {
            let protocol_id = Self::protocol_id().as_bytes();
            let Self { initial_protocols } = self;
            let mut framed = Framed::new(stream, io::U8LengthLineCodec::new());

            // respond protocols
            let msg = match framed.next().await {
                Some(Ok(msg)) => msg,
                _ => return Err(error::other("connection error")),
            };
            log::info!("responder protocol recv {:?}", std::str::from_utf8(&msg));
            if msg != protocol_id.to_vec() {
                framed.send(PROTOCOL_UNSUPPORTED.as_bytes()).await?;
                log::info!("responder protocol na");
                return Err(error::unsupported("multistream"));
            }
            framed.send(protocol_id).await?;
            log::info!(
                "responder protocol send {:?}",
                std::str::from_utf8(protocol_id)
            );
            log::info!("multistream agreed");

            // initiate protocols
            for protocol in initial_protocols {
                framed.send(&protocol).await?;
                log::info!(
                    "initiator protocol send {:?}",
                    std::str::from_utf8(&protocol)
                );
                let res = framed.next().await;
                log::debug!("initiator protocol recv {:?}", res);
                if let Some(Ok(msg)) = res {
                    log::info!("initiator protocol recv {:?}", std::str::from_utf8(&msg));
                    if msg != PROTOCOL_UNSUPPORTED.as_bytes().to_vec() {
                        log::info!("{:?} agreed", std::str::from_utf8(&msg));
                        return Ok((framed.into_inner(), msg));
                    }
                }
            }
            log::info!("initiator protocol na");
            Err(error::unsupported("upgrade protocol"))
        }
        .boxed()
    }
}
