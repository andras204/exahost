use std::net::SocketAddr;

use log::info;
use tokio::net::TcpStream;

use crate::backbone::Backbone;

use crate::server::protocol::{message::*, Error as ProtocolError, ProtocolStream};

use super::{read_msg, read_response, send_msg};

pub async fn connect(
    backbone: Backbone,
    link_id: Option<i16>,
    addr: SocketAddr,
    self_listening_port: u16,
) -> Result<(), ProtocolError> {
    let tcp = TcpStream::connect(addr).await?;
    let mut stream = ProtocolStream::wrap_tcp(tcp);

    send_msg(&mut stream, Message::connect_request(self_listening_port)).await?;

    if read_response(&mut stream).await? {
        let lid = match link_id {
            Some(id) => id,
            None => backbone.connections().auto_enum_async(false).await,
        };
        backbone.connections().add_async(lid, addr).await;

        info!(
            "[SERVER->CONNECTION] successfully connected to {}, entering keepalive loop",
            addr
        );

        let res = keepalive_loop(stream).await;

        info!("[SERVER->CONNECTION] {} disconnected", addr);

        backbone.connections().remove_async(lid).await;
        return res;
    }

    info!("[SERVER->CONNECTION] {} refused connection", addr);

    Ok(())
}

pub async fn accept_connection(
    backbone: Backbone,
    mut stream: ProtocolStream,
    port: u16,
) -> Result<(), ProtocolError> {
    let mut addr = stream.peer_addr()?;
    addr.set_port(port);

    send_msg(&mut stream, Message::yes()).await?;

    let lid = backbone.connections().auto_enum_async(true).await;
    backbone.connections().add_async(lid, addr).await;

    info!(
        "[SERVER->CONNECTION] accepted connection from {}, entering keepalive loop",
        addr
    );

    let res = keepalive_loop(stream).await;

    info!("[SERVER->CONNECTION] {} disconnected", addr);

    backbone.connections().remove_async(lid).await;

    res
}

async fn keepalive_loop(mut stream: ProtocolStream) -> Result<(), ProtocolError> {
    loop {
        send_msg(&mut stream, Message::KeepAlive).await?;
        match read_msg(&mut stream).await? {
            Message::KeepAlive => (),
            _ => return Err(ProtocolError::invalid_message_sequence()),
        }

        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
}
