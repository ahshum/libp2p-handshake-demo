use crate::error::{self, Error};
use multiaddr::{Multiaddr, Protocol};
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};

pub fn multiaddr_to_tcpaddr(addr: &Multiaddr) -> Result<SocketAddr, Error> {
    let mut hostname: Option<String> = None;
    let mut ip_addr: Option<IpAddr> = None;
    let mut port: Option<u16> = None;

    for proto in addr {
        match proto {
            Protocol::Dns4(dns) => hostname = Some(dns.to_string()),
            Protocol::Ip4(ipv4) => ip_addr = Some(IpAddr::V4(ipv4)),
            Protocol::Tcp(tcp_port) => port = Some(tcp_port),
            _ => todo!(),
        }
    }

    match (hostname, ip_addr, port) {
        (Some(hostname), _, Some(port)) => format!("{}:{}", hostname, port)
            .to_socket_addrs()
            .map_err(|_| error::parse_error())?
            .find(|addr| addr.is_ipv4())
            .ok_or(error::parse_error()),
        (_, Some(ip_addr), Some(port)) => Ok(SocketAddr::new(ip_addr, port)),
        _ => Err(error::parse_error()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_multiaddr_to_tcpaddr() {
        assert_eq!(
            multiaddr_to_tcpaddr(&"/dns4/localhost/tcp/4001".parse::<Multiaddr>().unwrap())
                .unwrap(),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4001)
        );

        assert_eq!(
            multiaddr_to_tcpaddr(
                &"/dns4/localhost/ip4/127.0.0.1/tcp/4001"
                    .parse::<Multiaddr>()
                    .unwrap()
            )
            .unwrap(),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4001)
        );

        assert_eq!(
            multiaddr_to_tcpaddr(
                &"/ip4/127.0.0.1/dns4/localhost/tcp/4001"
                    .parse::<Multiaddr>()
                    .unwrap()
            )
            .unwrap(),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4001)
        );

        assert_eq!(
            multiaddr_to_tcpaddr(&"/ip4/127.0.0.1".parse::<Multiaddr>().unwrap()).is_err(),
            true
        );

        assert_eq!(
            multiaddr_to_tcpaddr(&"/tcp/4001".parse::<Multiaddr>().unwrap()).is_err(),
            true
        );
    }
}
