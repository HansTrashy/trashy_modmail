use crate::storage::Storage;
use serenity::framework::standard::{macros::command, Args, CommandError, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
#[description = "Creates a modmail channel with the mentioned user"]
pub fn start_modmail(_ctx: &mut Context, _msg: &Message, _args: Args) -> CommandResult {
    Ok(())
}

#[command]
#[description = "Answer in a modmail channel to the user"]
pub fn reply(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read();
    let storage = match data.get::<Storage>() {
        Some(v) => v.lock(),
        None => {
            let _ = msg.reply(
                &ctx,
                "Could not retrieve the needed Storage, please inform Staff",
            );
            panic!("could not retrieve storage");
        }
    };

    let modmail_user = storage.get_user(msg.channel_id.as_u64());

    if modmail_user.is_none() {
        return Err(CommandError(
            "This channel is not associated with a user for modmail purposes".to_string(),
        ));
    }

    let user = UserId(*modmail_user.unwrap());

    let user_channel = user.create_dm_channel(&ctx)?;

    let _ = user_channel.send_message(&ctx, |m| {
        m.embed(|e| {
            e.author(|a| {
                a.name(msg.author.name.clone())
                    .icon_url(msg.author.static_avatar_url().unwrap_or_default())
            })
            .description(args.rest())
            .color((50, 100, 200))
        })
    });

    Ok(())
}

#[command]
#[description = "Delete the channel"]
pub fn close(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    let data = ctx.data.read();
    let mut storage = match data.get::<Storage>() {
        Some(v) => v.lock(),
        None => {
            let _ = msg.reply(
                &ctx,
                "Could not retrieve the needed Storage, please inform Staff",
            );
            panic!("could not retrieve storage");
        }
    };

    let modmail_user = storage.get_user(msg.channel_id.as_u64());

    if modmail_user.is_none() {
        return Err(CommandError(
            "This channel is not associated with a user for modmail purposes".to_string(),
        ));
    }

    let user = UserId(*modmail_user.unwrap());

    let user_channel = user.create_dm_channel(&ctx)?;

    let _ = user_channel.send_message(&ctx, |m| {
        m.embed(|e| {
            e.author(|a| {
                a.name(msg.author.name.clone())
                    .icon_url(msg.author.static_avatar_url().unwrap_or_default())
            })
            .description("Has closed the modmail session")
            .color((200, 50, 50))
        })
    });

    storage.remove_user_channel(msg.channel_id.as_u64());

    msg.channel_id.delete(&ctx)?;

    Ok(())
}
