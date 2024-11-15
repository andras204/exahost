use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::exa::Register;

type IpcChannel = Arc<Mutex<Option<Register>>>;

#[derive(Debug)]
pub struct IpcModule {
    channels: Mutex<HashMap<i16, IpcChannel>>,
}

impl IpcModule {
    pub fn new() -> Self {
        let mut channels = HashMap::new();
        channels.insert(0, Arc::new(Mutex::new(None)));
        Self {
            channels: Mutex::new(channels),
        }
    }

    pub fn get_default_channel(&self) -> IpcChannel {
        self.dial(0)
    }

    pub fn dial(&self, channel: i16) -> IpcChannel {
        let mut channels = self.channels.lock().unwrap();
        if !channels.contains_key(&channel) {
            channels.insert(channel, Arc::new(Mutex::new(None)));
        }
        return channels.get(&channel).unwrap().clone();
    }
}
