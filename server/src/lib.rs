mod validation;

use crate::validation::{validate_message, validate_name};
use spacetimedb::{reducer, table, Identity, ReducerContext, Table, Timestamp};
use std::ops::Add;

/// Defines a user
#[table(name = user, public)]
pub struct User {
    #[primary_key]
    id: Identity,
    #[unique]
    name: String,
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
    #[unique]
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
    sender: String,
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
    if !validate_name(&name) {
        return Err("Name isn't valid".into());
    }

    if ctx.db.user().name().find(&name).is_some() {
        return Err("Name not available".into());
    }

    if let Some(user) = ctx.db.user().id().find(ctx.sender) {
        ctx.db.user().id().update(User { name, ..user });
    } else {
        ctx.db.user().insert(User {
            id: ctx.sender,
            name,
            online: true,
        });
    }

    Ok(())
}

#[reducer]
pub fn send_message(ctx: &ReducerContext, text: String, channel: String) -> Result<(), String> {
    let user = ctx
        .db
        .user()
        .id()
        .find(ctx.sender)
        .ok_or("User is not registered")?;

    let channel = ctx
        .db
        .channel()
        .name()
        .find(channel)
        .ok_or("No channel found")?;

    if !validate_message(&text) {
        return Err("Message content isn't valid".into());
    }

    ctx.db.message().insert(Message {
        id: 0,
        sender: user.name,
        channel_id: channel.id,
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

    if !validate_name(&channel_name) {
        return Err("Channel name isn't valid".into());
    }

    if ctx.db.channel().name().find(&channel_name).is_some() {
        return Err("A channel already exists with this name".into());
    }

    let channel = ctx.db.channel().insert(Channel {
        id: 0,
        name: channel_name,
    });

    ctx.db.member().insert(Member {
        user_id: user.id,
        channel_id: channel.id,
    });

    Ok(())
}

#[reducer]
pub fn add_user(ctx: &ReducerContext, channel: String, user_name: String) -> Result<(), String> {
    let chan: Channel;
    if let Some(c) = ctx.db.channel().name().find(&channel) {
        chan = c;
    } else {
        log::info!("channel: {channel}");
        log::info!(
            "channels: {:?}",
            ctx.db
                .channel()
                .iter()
                .fold(String::new(), |str, channel| str.add(&channel.name))
        );
        return Err("No channel found".into());
    }

    let user = ctx
        .db
        .user()
        .name()
        .find(user_name)
        .ok_or("No user found")?;

    ctx.db.member().insert(Member {
        user_id: user.id,
        channel_id: chan.id,
    });

    Ok(())
}

#[reducer]
pub fn add_friend(ctx: &ReducerContext, user_name: String) -> Result<(), String> {
    let user_a = ctx.db.user().id().find(ctx.sender).ok_or("No user found")?;
    let user_b = ctx
        .db
        .user()
        .name()
        .find(user_name)
        .ok_or("No user found")?;

    // This sucks, it'd be better to compute an id starting from the two user ids
    for friend in ctx.db.friend().iter() {
        if (friend.user_a == user_a.id && friend.user_b == user_b.id)
            || (friend.user_b == user_a.id && friend.user_a == user_b.id)
        {
            return Err("Already a friend".into());
        }
    }

    ctx.db.friend().insert(Friend {
        user_a: user_a.id,
        user_b: user_b.id,
    });

    Ok(())
}
