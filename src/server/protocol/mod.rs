const PROTOCOL_VERSION: u32 = 0;

use std::net::SocketAddr;

use log::warn;
use message::Request;
use tokio::net::TcpStream;

mod error;
mod message;
mod message_stream;

pub use error::Error;
pub use message::{generate_header, is_header_version_valid, Message};
use message_stream::MessageStream;

use super::ServerRef;

pub async fn respond(request_stream: TcpStream, server_ref: ServerRef) -> Result<(), Error> {
    let peer_addr = request_stream.peer_addr().unwrap();
    let mut conn = MessageStream::wrap_tcp(request_stream);

    let request = read_request(&mut conn).await?;

    match request {
        Request::Connect(p) => accept_connect(peer_addr, p, server_ref).await,
        Request::SendExa => recv_exa(conn, server_ref).await?,
        _ => todo!(),
    }

    Ok(())
}

pub async fn connect(
    to: impl tokio::net::ToSocketAddrs,
    server_ref: ServerRef,
) -> Result<(), Error> {
    let stream = TcpStream::connect(to).await?;
    let addr = stream.peer_addr().unwrap();
    let mut conn = MessageStream::wrap_tcp(stream);

    let port = server_ref.get_port().await;

    send_msg(&mut conn, Message::connect_request(port)).await?;

    if read_response(&mut conn).await? {
        warn!("read_response success");
        server_ref.add_link_auto(addr, false).await;
    }

    Ok(())
}

async fn accept_connect(mut addr: SocketAddr, port: u16, server_ref: ServerRef) {
    addr.set_port(port);
    server_ref.add_link_auto(addr, true).await;
}

async fn send_exa(link_id: i16, exa_id: usize, server_ref: ServerRef) -> Result<(), Error> {
    let addr = server_ref.get_addr(&link_id).await.unwrap();
    let stream = TcpStream::connect(addr).await.unwrap();
    let mut conn = MessageStream::wrap_tcp(stream);

    conn.send_message(Message::exa_request()).await.unwrap();

    if read_response(&mut conn).await? {
        if let Some((_, (_, pexa))) = server_ref.vm_bridge.lock().await.remove_outgoing(&exa_id) {
            conn.send_message(Message::exa(pexa)).await?;
        } else {
            conn.send_message(Message::abort()).await?;
        }
    }

    Ok(())
}

async fn recv_exa(mut connection: MessageStream, server_ref: ServerRef) -> Result<(), Error> {
    let mut vm_bridge = server_ref.vm_bridge.lock().await;
    if vm_bridge.has_space() {
        connection.send_message(Message::yes()).await?;
    } else {
        connection.send_message(Message::no()).await?;
    }

    let asd = connection.read_message().await?;

    match asd {
        Message::Action(a) => match a {
            message::Action::Exa(pexa) => vm_bridge.push_incoming(pexa),
            message::Action::Abort => (),
            _ => return Err(Error::invalid_seq()),
        },
        _ => return Err(Error::invalid_seq()),
    }

    Ok(())
}

async fn read_response(connection: &mut MessageStream) -> Result<bool, Error> {
    match read_msg(connection).await? {
        Message::Response(r) => match r {
            message::Response::Yes => Ok(true),
            message::Response::No => Ok(false),
        },
        _ => Err(Error::invalid_seq()),
    }
}

async fn read_request(connection: &mut MessageStream) -> Result<Request, Error> {
    match read_msg(connection).await? {
        Message::Request(r) => Ok(r),
        _ => Err(Error::invalid_seq()),
    }
}

async fn read_msg(connection: &mut MessageStream) -> Result<Message, Error> {
    match connection.read_message().await {
        Ok(m) => Ok(m),
        Err(e) => {
            log::error!("message read error: {:?}", e);
            return Err(e);
        }
    }
}

async fn send_msg(connection: &mut MessageStream, msg: Message) -> Result<(), Error> {
    match connection.send_message(msg).await {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("message read error: {:?}", e);
            return Err(e);
        }
    }
}
