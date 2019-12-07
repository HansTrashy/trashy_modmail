use crate::MODMAIL_STORAGE;
use log::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct Storage {
    channel_for_user: HashMap<u64, u64>,
    user_for_channel: HashMap<u64, u64>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            channel_for_user: HashMap::new(),
            user_for_channel: HashMap::new(),
        }
    }

    pub fn load_or_default(path: &str) -> Self {
        if let Ok(file_content) = std::fs::read_to_string(path) {
            if let Ok(storage) = serde_json::from_str::<Self>(&file_content) {
                return storage;
            }
        }
        Self::new()
    }

    pub fn persist_changes_if_possible(&self) {
        match serde_json::to_string(self)
            .map_err(|e| format!("Json Error: {:?}", e))
            .and_then(|json| {
                std::fs::write(Path::new(&*MODMAIL_STORAGE), json)
                    .map_err(|e| format!("File access error: {:?}", e))
            }) {
            Ok(()) => (),
            Err(e) => error!("{}", e),
        }
    }

    pub fn insert_user_channel(&mut self, user: u64, channel: u64) {
        self.channel_for_user.insert(user, channel);
        self.user_for_channel.insert(channel, user);
        self.persist_changes_if_possible();
    }

    pub fn get_user(&self, channel: &u64) -> Option<&u64> {
        self.user_for_channel.get(channel)
    }

    pub fn get_channel(&self, user: &u64) -> Option<&u64> {
        self.channel_for_user.get(user)
    }

    pub fn remove_user_channel(&mut self, channel: &u64) {
        if let Some(user) = self.user_for_channel.remove(channel) {
            self.channel_for_user.remove(&user);
            self.persist_changes_if_possible();
        }
    }
}
