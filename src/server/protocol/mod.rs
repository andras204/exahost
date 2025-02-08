const PROTOCOL_VERSION: u32 = 0;

pub struct Message {
    pub bytes: Box<[u8]>,
}

pub struct ProtocolHeader {
    version: u32,
    payload_len: u32,
}

pub fn message_from_bytes(raw_bytes: &[u8]) -> Message {
    let mut buffer = Vec::new();
    let header = generate_header(raw_bytes.len());
    buffer.extend_from_slice(&header.version.to_be_bytes());
    buffer.extend_from_slice(&header.payload_len.to_be_bytes());
    buffer.extend_from_slice(raw_bytes);
    Message {
        bytes: buffer.into_boxed_slice(),
    }
}

pub fn recover_header(message: &Message) -> Result<ProtocolHeader, ()> {
    if message.bytes.len() < 8 {
        return Err(());
    }
    let vb: [u8; 4] = match message.bytes[0..4].try_into() {
        Ok(vb) => vb,
        Err(_) => return Err(()),
    };
    let lb: [u8; 4] = match message.bytes[4..8].try_into() {
        Ok(lb) => lb,
        Err(_) => return Err(()),
    };
    Ok(ProtocolHeader {
        version: u32::from_be_bytes(vb),
        payload_len: u32::from_be_bytes(lb),
    })
}

pub fn generate_header(payload_len: usize) -> ProtocolHeader {
    ProtocolHeader {
        version: PROTOCOL_VERSION,
        payload_len: payload_len as u32,
    }
}

pub fn validate_header_version(v: u32) -> bool {
    v == PROTOCOL_VERSION
}

pub fn handle_connection() {
    unimplemented!()
}
