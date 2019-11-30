mod commands;
mod handler;
mod storage;

use commands::*;
use dotenv::dotenv;
use log::{error, info};
use serenity::{
    client::bridge::gateway::ShardManager,
    framework::{standard::macros::group, StandardFramework},
    model::{event::ResumedEvent, gateway::Ready, id::ChannelId, id::GuildId, id::RoleId},
    prelude::*,
};
use std::{collections::HashSet, env, sync::Arc};
use storage::Storage;

const MODMAIL_SERVER: GuildId = GuildId(553329127705411614);
const MODMAIL_CATEGORY: ChannelId = ChannelId(650325086838194177);
const MOD_ROLE: RoleId = RoleId(562248175646146571);
const MODMAIL_ARCHIVE: ChannelId = ChannelId(0);

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
    commands: [start_modmail, reply, close]
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
