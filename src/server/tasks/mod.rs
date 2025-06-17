use log::info;
use tokio::net::TcpStream;

use crate::backbone::Backbone;

use crate::backbone::server_command::ServerCommand;
use crate::server::protocol::{message::*, Error, ProtocolStream};

pub mod connection;
pub mod exa;

pub async fn handle_request(backbone: Backbone, tcp: TcpStream) -> Result<(), Error> {
    let mut stream = ProtocolStream::wrap_tcp(tcp);

    let req = read_request(&mut stream).await?;

    info!("[SERVER->RQ_HANDLER] handling request: {:?}", req);

    match req {
        Request::Connect(p) => connection::accept_connection(backbone, stream, p).await?,
        Request::SendExa => exa::recv(backbone, stream).await?,
    }

    Ok(())
}

pub async fn exec_server_command(backbone: Backbone, sc: ServerCommand) -> Result<(), Error> {
    info!("[SERVER->TASK] executing command: {:?}", &sc);
    match sc {
        ServerCommand::Connect { addr, link_id } => {
            let port = backbone.get_server_listening_addr_async().await.port();
            connection::connect(backbone, link_id, addr, port).await?
        }
        ServerCommand::Disconnect { link_id } => {
            backbone.connections().disconnect_async(link_id).await;
        }
        ServerCommand::DisconnectAll => {
            backbone.connections().disconnect_all_async().await;
        }
        ServerCommand::NotifyExaLink => {
            exa::dispatch_all_unhandled(backbone).await;
        }
    };
    Ok(())
}

async fn read_response(stream: &mut ProtocolStream) -> Result<bool, Error> {
    match read_msg(stream).await? {
        Message::Response(r) => match r {
            Response::Yes => Ok(true),
            Response::No => Ok(false),
        },
        _ => Err(Error::invalid_message_sequence()),
    }
}

async fn read_request(stream: &mut ProtocolStream) -> Result<Request, Error> {
    match read_msg(stream).await? {
        Message::Request(r) => Ok(r),
        _ => Err(Error::invalid_message_sequence()),
    }
}

async fn read_msg(stream: &mut ProtocolStream) -> Result<Message, Error> {
    match stream.read_message().await {
        Ok(m) => Ok(m),
        Err(e) => {
            log::error!("message read error: {:?}", e);
            return Err(e);
        }
    }
}

async fn send_msg(stream: &mut ProtocolStream, msg: Message) -> Result<(), Error> {
    match stream.send_message(msg).await {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("message read error: {:?}", e);
            return Err(e);
        }
    }
}
