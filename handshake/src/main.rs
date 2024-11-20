use libp2p_handshake_lib::{
    error::{self, Error},
    net::Manager,
};
use std::{env, fs::File, io::Write};

#[async_std::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    let addr = match env::var("HANDSHAKE_TARGET_ADDR") {
        Ok(addr) => addr,
        Err(_) => return Err(error::invalid_input("missing env HANDSHAKE_TARGET_ADDR")),
    };
    let manager = Manager::from_key_path_and_addr("../ed25519.pem", &addr)?;

    // save peer id for log filtering
    let mut file = File::create("../peerid").unwrap();
    file.write_all(manager.peer_id()?.as_bytes()).unwrap();

    // connect to target node
    log::info!("connecting to {}", addr);
    manager.tcp_connect().await?;
    log::info!("connection ended");

    Ok(())
}
