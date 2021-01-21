use super::{MODMAIL_CATEGORY, MODMAIL_SERVER, MOD_ROLE, SELF_ID};
use crate::storage::Storage;
use serenity::{
    async_trait,
    model::{
        channel::{Attachment, ChannelType, Message, ReactionType},
        event::ResumedEvent,
        gateway::Ready,
        id::ChannelId,
    },
    prelude::*,
    utils::MessageBuilder,
};
use tracing::*;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if !msg.is_private() || msg.author.id == *SELF_ID {
            return;
        }
        let data = ctx.data.read().await;
        let mut storage = match data.get::<Storage>() {
            Some(v) => v.lock().await,
            None => {
                let _ = msg
                    .reply(
                        &ctx,
                        "Could not retrieve the needed Storage, please inform Staff",
                    )
                    .await;
                return;
            }
        };

        match storage.get_channel(msg.author.id.as_u64()) {
            // modmail channel exists
            Some(c) => {
                let channel_id = ChannelId(*c);

                let image = msg
                    .attachments
                    .iter()
                    .cloned()
                    .filter(|a| a.width.is_some())
                    .collect::<Vec<Attachment>>()
                    .first()
                    .map(|a| a.url.to_string());

                let result = channel_id
                    .send_message(&ctx, |m| {
                        m.embed(|e| {
                            build_embed(
                                e,
                                &msg.author.name,
                                &msg.author.avatar_url().unwrap_or_default(),
                                &msg.content,
                                image,
                            )
                        })
                    })
                    .await;

                match result {
                    Ok(_) => {
                        debug!("sent message successfully to modmail channel");
                        let _ = msg
                            .react(&ctx, ReactionType::Unicode("✅".to_string()))
                            .await;
                    }
                    Err(e) => {
                        debug!(?e, "failed to send message to modmail channel");
                        let _ = msg
                            .react(&ctx, ReactionType::Unicode("❌".to_string()))
                            .await;
                    }
                }
            }
            // modmail channel does not exist
            None => {
                let modmail_channel = MODMAIL_SERVER
                    .create_channel(&ctx, |c| {
                        c.name(format!("{}-{}", msg.author.name, msg.author.discriminator))
                            .kind(ChannelType::Text)
                            .category(&*MODMAIL_CATEGORY)
                    })
                    .await
                    .expect("Could not create modmail channel");

                storage.insert_user_channel(*msg.author.id.as_u64(), *modmail_channel.id.as_u64());

                let r = modmail_channel
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
                tracing::debug!(?r, "opened modmail session");

                let image = msg
                    .attachments
                    .iter()
                    .cloned()
                    .filter(|a| a.width.is_some())
                    .collect::<Vec<Attachment>>()
                    .first()
                    .map(|a| a.url.to_string());

                let result = modmail_channel
                    .send_message(&ctx, |m| {
                        m.embed(|e| {
                            build_embed(
                                e,
                                &msg.author.name,
                                &msg.author.avatar_url().unwrap_or_default(),
                                &msg.content,
                                image,
                            )
                        })
                    })
                    .await;

                match result {
                    Ok(_) => {
                        debug!("sent message successfully to modmail channel");
                        let _ = msg
                            .react(&ctx, ReactionType::Unicode("✅".to_string()))
                            .await;
                    }
                    Err(e) => {
                        debug!(?e, "failed to send message to modmail channel");
                        let _ = msg
                            .react(&ctx, ReactionType::Unicode("❌".to_string()))
                            .await;
                    }
                }
            }
        }
    }
}

fn build_embed<'a>(
    e: &'a mut serenity::builder::CreateEmbed,
    author: &str,
    avatar_url: &str,
    content: &str,
    image: Option<String>,
) -> &'a mut serenity::builder::CreateEmbed {
    let mut embed = e
        .author(|a| a.name(author).icon_url(avatar_url))
        .color((200, 100, 100))
        .description(content);

    if let Some(image) = image {
        embed = embed.image(&image);
    }

    embed
}
