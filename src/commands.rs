use crate::storage::Storage;
use crate::{MODMAIL_CATEGORY, MODMAIL_SERVER, MOD_ROLE};
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

#[command]
#[description = "Creates a modmail channel with the mentioned user"]
pub async fn init(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let mut storage = match data.get::<Storage>() {
        Some(v) => v.lock().await,
        None => {
            msg.reply(
                &ctx,
                "Could not retrieve the needed Storage, please inform Staff",
            )
            .await;
            return Err("could not retrieve storage".into());
        }
    };

    let user = match msg.mentions.get(0) {
        Some(u) => u,
        None => return Err("No user mentioned in command".into()),
    };

    if storage.get_channel(user.id.as_u64()).is_some() {
        return Err("A modmail channel for this user already exists".into());
    }

    let modmail_channel = MODMAIL_SERVER
        .create_channel(&ctx, |c| {
            c.name(format!("{}-{}", user.name, user.discriminator))
                .kind(ChannelType::Text)
                .category(&*MODMAIL_CATEGORY)
        })
        .await
        .expect("Could not create modmail channel");

    storage.insert_user_channel(*user.id.as_u64(), *modmail_channel.id.as_u64());

    modmail_channel
        .send_message(&ctx, |m| {
            m.content(
                MessageBuilder::new()
                    .mention(&*MOD_ROLE)
                    .push(" ")
                    .mention(&msg.author)
                    .push(" has started a modmail session!")
                    .build(),
            )
        })
        .await;

    Ok(())
}

#[command]
#[description = "Answer in a modmail channel to the user"]
pub async fn reply(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let storage = match data.get::<Storage>() {
        Some(v) => v.lock().await,
        None => {
            let _ = msg.reply(
                &ctx,
                "Could not retrieve the needed Storage, please inform Staff",
            );
            return Err("could not retrieve storage".into());
        }
    };

    let modmail_user = storage.get_user(msg.channel_id.as_u64());

    if modmail_user.is_none() {
        return Err("This channel is not associated with a user for modmail purposes".into());
    }

    let user = UserId(*modmail_user.unwrap());

    let user_channel = user
        .create_dm_channel(&ctx)
        .await
        .map_err(|e| format!("Failed to create dm channel: {:?}", e))?;

    let result = user_channel
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.author(|a| {
                    a.name(msg.author.name.clone())
                        .icon_url(msg.author.static_avatar_url().unwrap_or_default())
                })
                .description(args.rest())
                .color((50, 100, 200))
            })
        })
        .await;

    match result {
        Ok(_) => {
            msg.react(&ctx, ReactionType::Unicode("✅".to_string()))
                .await?;
        }
        Err(e) => {
            msg.react(&ctx, ReactionType::Unicode("❎".to_string()))
                .await?;
        }
    }

    Ok(())
}

#[command]
#[description = "Delete the channel"]
pub async fn close(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let mut storage = match data.get::<Storage>() {
        Some(v) => v.lock().await,
        None => {
            msg.reply(
                &ctx,
                "Could not retrieve the needed Storage, please inform Staff",
            )
            .await?;
            return Err("could not retrieve storage".into());
        }
    };

    let modmail_user = storage.get_user(msg.channel_id.as_u64());

    if modmail_user.is_none() {
        return Err("This channel is not associated with a user for modmail purposes".into());
    }

    let user = UserId(*modmail_user.unwrap());

    let user_channel = user.create_dm_channel(&ctx).await?;

    user_channel
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.author(|a| {
                    a.name(msg.author.name.clone())
                        .icon_url(msg.author.static_avatar_url().unwrap_or_default())
                })
                .description("Has closed the modmail session")
                .color((200, 50, 50))
            })
        })
        .await?;

    storage.remove_user_channel(msg.channel_id.as_u64());

    msg.channel_id.delete(&ctx).await?;

    Ok(())
}
