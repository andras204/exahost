use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::exa::Register;

pub type IpcChannel = Arc<Mutex<Option<Register>>>;
pub type ChannelHandle = (i16, IpcChannel);

#[derive(Debug)]
pub struct IpcModule {
    channels: HashMap<i16, IpcChannel>,
}

impl IpcModule {
    pub fn new() -> Self {
        let mut channels = HashMap::new();
        channels.insert(0, Arc::new(Mutex::new(None)));
        Self { channels }
    }

    pub fn get_default_channel(&self) -> ChannelHandle {
        (0, self.channels.get(&0).unwrap().clone())
    }

    pub fn dial(&mut self, channel: i16) -> ChannelHandle {
        if !self.channels.contains_key(&channel) {
            self.channels.insert(channel, Arc::new(Mutex::new(None)));
        }
        (channel, self.channels.get(&channel).unwrap().clone())
    }
}
