const PROTOCOL_VERSION: u32 = 0;

use std::{net::SocketAddr, sync::Arc};

use message::Request;
use tokio::{net::TcpStream, sync::Mutex};

mod message;
mod message_stream;

pub use message::{generate_header, is_header_version_valid, Message};
use message_stream::MessageStream;

use crate::exa::PackedExa;

use super::ServerRef;

pub async fn respond(request_stream: TcpStream, server_ref: ServerRef) -> Result<(), ()> {
    let peer_addr = request_stream.peer_addr().unwrap();
    let mut conn = MessageStream::wrap_tcp(request_stream);

    let request = read_request(&mut conn).await?;

    match request {
        Request::Connect(p) => accept_connect(peer_addr, p, server_ref).await,
        _ => todo!(),
    }

    Ok(())
}

pub async fn connect(to: impl tokio::net::ToSocketAddrs, server_ref: ServerRef) -> Result<(), ()> {
    let stream = TcpStream::connect(to).await.unwrap();
    let addr = stream.peer_addr().unwrap();
    let mut conn = MessageStream::wrap_tcp(stream);

    let port = server_ref.get_port().await;

    conn.send_message(Message::Request(Request::Connect(port)))
        .await
        .unwrap();

    if read_response(&mut conn).await? {
        server_ref.add_link_auto(addr, false).await;
    }

    Ok(())
}

async fn accept_connect(mut addr: SocketAddr, port: u16, server_ref: ServerRef) {
    addr.set_port(port);
    server_ref.add_link_auto(addr, true).await;
}

async fn send_exa(id: i16, server_ref: ServerRef) -> Result<(), ()> {
    let addr = server_ref.get_addr(&id).await.unwrap();
    let stream = TcpStream::connect(addr).await.unwrap();
    let mut conn = MessageStream::wrap_tcp(stream);

    conn.send_message(Message::exa_request()).await.unwrap();

    if read_response(&mut conn).await? {
        if let Some(pexa) = shared_exa.lock().await.take() {
            conn.send_message(Message::exa(pexa)).await.unwrap();
        } else {
            conn.send_message(Message::abort()).await.unwrap();
        }
    }

    Ok(())
}

async fn recv_exa(mut connection: MessageStream, server_ref: ServerRef) -> Result<(), ()> {
    todo!()
}

async fn read_response(connection: &mut MessageStream) -> Result<bool, ()> {
    match connection.read_message().await {
        Ok(m) => match m {
            Message::Response(r) => match r {
                message::Response::Yes => Ok(true),
                message::Response::No => Ok(false),
            },
            _ => Err(()),
        },
        Err(_) => Err(()),
    }
}

async fn read_request(connection: &mut MessageStream) -> Result<Request, ()> {
    match connection.read_message().await {
        Ok(m) => match m {
            Message::Request(r) => Ok(r),
            _ => Err(()),
        },
        Err(_) => Err(()),
    }
}
