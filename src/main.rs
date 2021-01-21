mod commands;
mod handler;
mod storage;

use commands::*;
use dotenv::dotenv;
use lazy_static::lazy_static;
use serenity::{
    async_trait,
    client::bridge::gateway::{ShardId, ShardManager},
    framework::standard::{
        help_commands,
        macros::{check, command, group, help, hook},
        Args, CommandGroup, CommandOptions, CommandResult, DispatchError, HelpOptions,
        StandardFramework,
    },
    http::Http,
    model::{
        channel::{Channel, Message},
        gateway::Ready,
        id::{ChannelId, GuildId, RoleId, UserId},
        permissions::Permissions,
    },
    utils::{content_safe, ContentSafeOptions},
};
use serenity::{client::ClientBuilder, prelude::*};
use std::{collections::HashSet, env, sync::Arc};
use storage::Storage;
use tokio::sync::Mutex;
use tracing::{debug, error};

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
    static ref MODMAIL_STORAGE: String = env::var("MODMAIL_STORAGE").expect("missing env var");
}

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

impl TypeMapKey for Storage {
    type Value = Arc<Mutex<Storage>>;
}

#[group]
#[commands(init, reply, close, silentclose)]
struct General;

#[hook]
async fn dispatch_error(_ctx: &Context, _msg: &Message, error: DispatchError) {
    debug!(?error, "Dispatch failed");
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt().with_env_filter("info").init();

    let token = env::var("DISCORD_TOKEN").expect("No DISCORD_TOKEN found in environment");

    let http = Http::new_with_token(&token);

    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("Could not access the bot id: {:?}", why),
            }
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| {
            c.with_whitespace(true)
                .on_mention(Some(bot_id))
                .prefix("~")
                // In this case, if "," would be first, a message would never
                // be delimited at ", ", forcing you to trim your arguments if you
                // want to avoid whitespaces at the start of each.
                .delimiter(" ")
                // Sets the bot's owners. These will be used for commands that
                // are owners only.
                .owners(owners)
        })
        .on_dispatch_error(dispatch_error)
        .group(&GENERAL_GROUP);

    let mut client = ClientBuilder::new(&token)
        .event_handler(handler::Handler)
        .framework(framework)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<Storage>(Arc::new(Mutex::new(Storage::load_or_default(
            &MODMAIL_STORAGE,
        ))));
    }

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
