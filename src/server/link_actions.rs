use crate::exa::PackedExa;

#[derive(Debug, Clone)]
pub enum LinkInput {
    Connect,
    SendExa(PackedExa),
}

#[derive(Debug, Clone)]
pub enum LinkOutput {
    ConnectionDropped,
    RecievedExa(PackedExa),
}
