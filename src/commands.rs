use crate::ShardManagerContainer;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
#[description = "Creates a modmail channel with the mentioned user"]
pub fn start_modmail(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    Ok(())
}

#[command]
#[description = "Anser in a modmail channel to user"]
pub fn reply(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    Ok(())
}

#[command]
#[description = ""]
pub fn archive(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    Ok(())
}

#[command]
#[description = ""]
pub fn close(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    Ok(())
}
