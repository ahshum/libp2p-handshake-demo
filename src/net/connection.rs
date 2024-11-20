use super::{multiaddr_to_tcpaddr, Multistream, Noise, ProtocolId, UpgradeOutbound};
use crate::{
    error::{self, Error},
    identity::{PeerId, PrivateKey},
};
use async_io::Async;
use std::{
    net::{SocketAddr, TcpStream},
    path::Path,
};

pub struct Manager {
    socket_addr: SocketAddr,
    private_key: PrivateKey,
}

impl Manager {
    pub fn from_key_path_and_addr(
        key_path: impl AsRef<Path>,
        target_addr: &str,
    ) -> Result<Self, Error> {
        let socket_addr =
            multiaddr_to_tcpaddr(&target_addr.parse().map_err(|_| error::parse_error())?)?;
        let private_key = PrivateKey::from_ed25519_pem_file(key_path)?;
        Ok(Self {
            socket_addr,
            private_key,
        })
    }

    pub fn peer_id(&self) -> Result<PeerId, Error> {
        self.private_key.public().try_into()
    }

    pub async fn tcp_connect(&self) -> Result<(), Error> {
        let stream = Async::<TcpStream>::connect(self.socket_addr)
            .await
            .map_err(|_| error::other("async stream"))?;

        // upgrader
        let noise_upgrader = Noise::new(self.private_key.clone());
        let security_upgrader = Multistream::new(vec![Noise::protocol_id().as_bytes().to_vec()]);

        // multistream select
        let (stream, _agreed) = security_upgrader.upgrade_outbound(stream).await?;

        // noise handshake
        let (upgraded_stream, _peer_id) = noise_upgrader.upgrade_outbound(stream).await?;

        // ===
        //
        // Note:
        // Handshake is done here
        // The following is to complete the connection upgrade so that it is easier to check on logs
        //
        // ===

        // multistream select over noise transport
        let muxer_upgrader = Multistream::new(vec!["/yamux/1.0.0".as_bytes().to_vec()]);
        let (_upgraded_stream, _agreed) = muxer_upgrader.upgrade_outbound(upgraded_stream).await?;

        log::info!("sleep for 60s for holding the connection");
        std::thread::sleep(std::time::Duration::from_secs(60));

        Ok(())
    }
}
