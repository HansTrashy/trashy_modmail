mod commands;
mod handler;
mod storage;

use commands::*;
use dotenv::dotenv;
use lazy_static::lazy_static;
use log::*;
use serenity::{
    client::bridge::gateway::ShardManager,
    framework::{standard::macros::group, StandardFramework},
    model::{id::ChannelId, id::GuildId, id::RoleId, id::UserId},
    prelude::*,
};
use std::{collections::HashSet, env, sync::Arc};
use storage::Storage;

fn load_env(name: &str) -> u64 {
    env::var(name)
        .expect("missing env var")
        .parse::<u64>()
        .expect("env var is not valid u64")
}

lazy_static! {
    static ref SELF_ID: UserId = UserId(load_env("SELF_ID"));
    static ref MODMAIL_SERVER: GuildId = GuildId(load_env("MODMAIL_SERVER"));
    static ref MODMAIL_CATEGORY: ChannelId = ChannelId(load_env("MODMAIL_CATEGORY"));
    static ref MOD_ROLE: RoleId = RoleId(load_env("MOD_ROLE"));
}

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

impl TypeMapKey for Storage {
    type Value = Arc<Mutex<Storage>>;
}

group!({
    name: "general",
    options: {},
    commands: [init, reply, close]
});

fn main() {
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("No DISCORD_TOKEN found in environment");

    let mut client = Client::new(&token, handler::Handler).expect("Could not create client");

    {
        let mut data = client.data.write();
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<Storage>(Arc::new(Mutex::new(Storage::new())));
    }

    let owners = match client.cache_and_http.http.get_current_application_info() {
        Ok(info) => {
            let mut set = HashSet::new();
            set.insert(info.owner.id);

            set
        }
        Err(why) => panic!("Could not get application info: {:?}", why),
    };

    client.with_framework(
        StandardFramework::new()
            .configure(|c| c.owners(owners).prefix("~"))
            .group(&GENERAL_GROUP),
    );

    if let Err(why) = client.start() {
        error!("Client error: {:?}", why);
    }
}
