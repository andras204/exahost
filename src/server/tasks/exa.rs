use std::net::SocketAddr;

use tokio::net::TcpStream;

use crate::backbone::Backbone;

use crate::server::protocol::{message::*, Error as ProtocolError, ProtocolStream};

use super::{read_msg, read_response, send_msg};

pub async fn dispatch_all_unhandled(backbone: Backbone) {
    for exa_id in backbone.outgoing().get_unhandled_keys_async().await {
        let bb = backbone.clone();
        tokio::spawn(async move {
            let _ = send(bb, exa_id).await;
        });
    }
}

pub async fn send(backbone: Backbone, exa_id: usize) -> Result<(), ProtocolError> {
    let link = match backbone.outgoing().get_link_async(exa_id).await {
        Some(l) => l,
        None => return Ok(()),
    };
    let addr = match backbone.connections().get_addr_async(link).await {
        Some(a) => a,
        None => {
            backbone.outgoing().take_exa_async(exa_id).await;
            return Ok(());
        }
    };

    backbone.outgoing().mark_handled_async(exa_id).await;

    let res = try_send_loop(backbone.clone(), exa_id, addr).await;

    backbone.outgoing().mark_unhandled_async(exa_id).await;

    res
}

async fn try_send_loop(
    backbone: Backbone,
    exa_id: usize,
    destination: SocketAddr,
) -> Result<(), ProtocolError> {
    loop {
        let tcp = TcpStream::connect(destination).await?;
        let mut stream = ProtocolStream::wrap_tcp(tcp);

        send_msg(&mut stream, Message::exa_request()).await?;

        if read_response(&mut stream).await? {
            let exa = match backbone.outgoing().take_exa_async(exa_id).await {
                Some((e, _)) => e,
                None => {
                    send_msg(&mut stream, Message::abort()).await?;
                    return Ok(());
                }
            };
            send_msg(&mut stream, Message::exa(exa)).await?;
        }
    }
}

pub async fn recv(backbone: Backbone, mut stream: ProtocolStream) -> Result<(), ProtocolError> {
    let t = match backbone.cap_pool().take_token() {
        Some(t) => t,
        None => {
            send_msg(&mut stream, Message::no()).await?;
            return Ok(());
        }
    };

    send_msg(&mut stream, Message::yes()).await?;

    let exa = match read_msg(&mut stream).await? {
        Message::Action(a) => match a {
            Action::Exa(exa) => exa,
            _ => return Err(ProtocolError::invalid_message_sequence()),
        },
        _ => return Err(ProtocolError::invalid_message_sequence()),
    };

    backbone.incoming().push_async(exa, t).await;

    Ok(())
}
