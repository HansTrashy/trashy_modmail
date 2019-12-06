use super::{MODMAIL_CATEGORY, MODMAIL_SERVER, MOD_ROLE, SELF_ID};
use crate::storage::Storage;
use log::*;
use serenity::{
    model::{
        channel::{Attachment, ChannelType, Message},
        event::ResumedEvent,
        gateway::Ready,
        id::ChannelId,
    },
    prelude::*,
    utils::MessageBuilder,
};

pub struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }

    fn message(&self, ctx: Context, msg: Message) {
        if !msg.is_private() || msg.author.id == *SELF_ID {
            return;
        }
        let data = ctx.data.read();
        let mut storage = match data.get::<Storage>() {
            Some(v) => v.lock(),
            None => {
                let _ = msg.reply(
                    &ctx,
                    "Could not retrieve the needed Storage, please inform Staff",
                );
                return;
            }
        };

        match storage.get_channel(msg.author.id.as_u64()) {
            // modmail channel exists
            Some(c) => {
                let channel_id = ChannelId(*c);

                let _ = channel_id.send_message(&ctx, |m| {
                    m.embed(|e| {
                        let mut embed = e
                            .author(|a| {
                                a.name(msg.author.name.clone())
                                    .icon_url(msg.author.static_avatar_url().unwrap_or_default())
                            })
                            .color((200, 100, 100))
                            .description(msg.content_safe(&ctx));

                        if let Some(image) = msg
                            .attachments
                            .iter()
                            .cloned()
                            .filter(|a| a.width.is_some())
                            .collect::<Vec<Attachment>>()
                            .first()
                        {
                            embed = embed.image(&image.url);
                        }

                        embed
                    })
                });
            }
            // modmail channel does not exist
            None => {
                let modmail_channel = MODMAIL_SERVER
                    .create_channel(&ctx, |c| {
                        c.name(format!("{}-{}", msg.author.name, msg.author.discriminator))
                            .kind(ChannelType::Text)
                            .category(&*MODMAIL_CATEGORY)
                    })
                    .expect("Could not create modmail channel");

                storage.insert_user_channel(*msg.author.id.as_u64(), *modmail_channel.id.as_u64());

                let _ = modmail_channel.send_message(&ctx, |m| {
                    m.content(
                        MessageBuilder::new()
                            .mention(&*MOD_ROLE)
                            .push(" ")
                            .mention(&msg.author)
                            .push(" has started a modmail session!")
                            .build(),
                    )
                });

                let _ = modmail_channel.send_message(&ctx, |m| {
                    m.embed(|e| {
                        let mut embed = e
                            .author(|a| {
                                a.name(msg.author.name.clone())
                                    .icon_url(msg.author.static_avatar_url().unwrap_or_default())
                            })
                            .color((200, 100, 100))
                            .description(msg.content_safe(&ctx));

                        if let Some(image) = msg
                            .attachments
                            .iter()
                            .cloned()
                            .filter(|a| a.width.is_some())
                            .collect::<Vec<Attachment>>()
                            .first()
                        {
                            embed = embed.image(&image.url);
                        }

                        embed
                    })
                });
            }
        }
    }
}
