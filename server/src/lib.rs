mod validation;

use crate::validation::{validate_message, validate_name};
use spacetimedb::{reducer, table, Identity, ReducerContext, Table, Timestamp};

/// Defines a user
#[table(name = user, public)]
pub struct User {
    #[primary_key]
    id: Identity,
    name: Option<String>,
    online: bool,
}

/// Defines a friendship relation between two users
#[table(name = friend, public)]
pub struct Friend {
    user_a: Identity,
    user_b: Identity,
}

/// Defines a channel that has a name
#[table(name = channel, public)]
pub struct Channel {
    #[primary_key]
    #[auto_inc]
    id: i128,
    name: String,
}

/// Defines a member, aka a user and which channel he's in
#[table(name = member, public)]
pub struct Member {
    user_id: Identity,
    channel_id: i128,
}

/// Defines a message sent in a specific channel
#[table(name = message, public)]
pub struct Message {
    #[primary_key]
    #[auto_inc]
    id: i128,
    sender: Identity,
    channel_id: i128,
    sent: Timestamp,
    text: String,
}

#[reducer(client_connected)]
pub fn client_connected(ctx: &ReducerContext) {
    if let Some(user) = ctx.db.user().id().find(ctx.sender) {
        ctx.db.user().id().update(User {
            online: true,
            ..user
        });
    } else {
        log::info!("New user connected");
        ctx.db.user().insert(User {
            id: ctx.sender,
            online: true,
            name: None,
        });
    }
}

#[reducer(client_disconnected)]
pub fn client_disconnected(ctx: &ReducerContext) {
    if let Some(user) = ctx.db.user().id().find(ctx.sender) {
        ctx.db.user().id().update(User {
            online: false,
            ..user
        });
    } else {
        log::info!("Unregistered user disconnected");
    }
}

#[reducer]
pub fn set_name(ctx: &ReducerContext, name: String) -> Result<(), String> {
    let user = ctx
        .db
        .user()
        .id()
        .find(ctx.sender)
        .ok_or("User is not registered")?;

    if !validate_name(&name) {
        return Err("Name isn't valid".into());
    }

    ctx.db.user().id().update(User {
        name: Some(name),
        ..user
    });

    Ok(())
}

#[reducer]
pub fn send_message(ctx: &ReducerContext, text: String, channel: i128) -> Result<(), String> {
    let user = ctx
        .db
        .user()
        .id()
        .find(ctx.sender)
        .ok_or("User is not registered")?;

    if user.name.is_none() {
        return Err("User doesn't have a name".into());
    }

    if ctx.db.channel().id().find(channel).is_none() {
        return Err("Channel doesn't exist".into());
    }

    if !validate_message(&text) {
        return Err("Message content isn't valid".into());
    }

    ctx.db.message().insert(Message {
        id: 0,
        sender: ctx.sender,
        channel_id: channel,
        sent: ctx.timestamp,
        text,
    });

    Ok(())
}

#[reducer]
pub fn create_channel(ctx: &ReducerContext, channel_name: String) -> Result<(), String> {
    let user = ctx
        .db
        .user()
        .id()
        .find(ctx.sender)
        .ok_or("User is not registered")?;

    if user.name.is_none() {
        return Err("User doesn't have a name".into());
    }

    if !validate_name(&channel_name) {
        return Err("Channel name isn't valid".into());
    }

    let channel = ctx.db.channel().insert(Channel {
        id: 0,
        name: channel_name,
    });

    ctx.db.member().insert(Member {
        user_id: ctx.sender,
        channel_id: channel.id,
    });

    Ok(())
}

#[reducer]
pub fn add_user(ctx: &ReducerContext, channel_id: i128, user_id: Identity) -> Result<(), String> {
    if ctx.db.channel().id().find(channel_id).is_none() {
        return Err("No channel found".into());
    }

    if ctx.db.user().id().find(user_id).is_none() {
        return Err("No user found".into());
    }

    ctx.db.member().insert(Member {
        user_id,
        channel_id,
    });

    Ok(())
}

#[reducer]
pub fn add_friend(ctx: &ReducerContext, user_id: Identity) -> Result<(), String> {
    if ctx.db.user().id().find(ctx.sender).is_none() || ctx.db.user().id().find(user_id).is_none() {
        return Err("No user found".into());
    }

    // This sucks, it'd be better to compute an id starting from the two user ids
    for friend in ctx.db.friend().iter() {
        if (friend.user_a == ctx.sender && friend.user_b == user_id)
            || (friend.user_b == ctx.sender && friend.user_a == user_id)
        {
            return Err("Already a friend".into());
        }
    }

    ctx.db.friend().insert(Friend {
        user_a: ctx.sender,
        user_b: user_id,
    });

    Ok(())
}
