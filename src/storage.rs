use std::collections::HashMap;

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

    pub fn insert_user_channel(&mut self, user: u64, channel: u64) {
        self.channel_for_user.insert(user, channel);
        self.user_for_channel.insert(channel, user);
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
        }
    }
}
