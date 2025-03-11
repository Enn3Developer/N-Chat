//! Entry point of the module
//!
//! WARNING: When changing anything related to the public api, remember to regenerate the spacetime bindings for the client:
//! `spacetime generate --lang csharp --out-dir Client/ModuleBindings --project-path server`

mod validation;

use crate::validation::{validate_message, validate_name};
use spacetimedb::{reducer, table, Identity, ReducerContext, Table, Timestamp};
use std::hash::{DefaultHasher, Hash, Hasher};

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
    #[unique]
    hash: u64,
    user_a: Identity,
    user_b: Identity,
}

#[table(name = friend_request, public)]
pub struct FriendRequest {
    #[unique]
    hash: u64,
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
    // update the user if it exists
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
    // update the user if it exists
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
    // validate the name
    if !validate_name(&name) {
        return Err("Name isn't valid".into());
    }

    // check if the name is unique
    if ctx.db.user().name().find(&name).is_some() {
        return Err("Name not available".into());
    }

    // update or add the user
    if let Some(user) = ctx.db.user().id().find(ctx.sender) {
        ctx.db.user().id().update(User { name, ..user });
    } else {
        ctx.db.user().insert(User {
            id: ctx.sender,
            name,
            // we set it to true because we assume the user is connected before it can use reducers
            online: true,
        });
    }

    Ok(())
}

#[reducer]
pub fn send_message(ctx: &ReducerContext, text: String, channel: String) -> Result<(), String> {
    // get the user
    let user = ctx
        .db
        .user()
        .id()
        .find(ctx.sender)
        .ok_or("User is not registered")?;

    // get the channel
    let channel = ctx
        .db
        .channel()
        .name()
        .find(channel)
        .ok_or("No channel found")?;

    // validate the message
    if !validate_message(&text) {
        return Err("Message content isn't valid".into());
    }

    // add the message
    ctx.db.message().insert(Message {
        // it is 0 because `id` is `auto_inc` so it is changed before committing the change
        // to the database with the new id value
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
    // get the user
    let user = ctx
        .db
        .user()
        .id()
        .find(ctx.sender)
        .ok_or("User is not registered")?;

    // check the channel name
    if !validate_name(&channel_name) {
        return Err("Channel name isn't valid".into());
    }

    // check if the channel name is unique
    if ctx.db.channel().name().find(&channel_name).is_some() {
        return Err("A channel already exists with this name".into());
    }

    // create the channel
    let channel = ctx.db.channel().insert(Channel {
        id: 0,
        name: channel_name,
    });

    // add the user as a member of the channel
    ctx.db.member().insert(Member {
        user_id: user.id,
        channel_id: channel.id,
    });

    Ok(())
}

#[reducer]
pub fn add_user(ctx: &ReducerContext, channel: String, user_name: String) -> Result<(), String> {
    // get the channel
    let channel = ctx
        .db
        .channel()
        .name()
        .find(&channel)
        .ok_or("No channel found")?;

    // get the user
    let user = ctx
        .db
        .user()
        .name()
        .find(user_name)
        .ok_or("No user found")?;

    // add the user as a member of the channel
    ctx.db.member().insert(Member {
        user_id: user.id,
        channel_id: channel.id,
    });

    Ok(())
}

#[reducer]
pub fn add_friend(ctx: &ReducerContext, user_name: String) -> Result<(), String> {
    // get users
    let user_a = ctx.db.user().id().find(ctx.sender).ok_or("No user found")?;
    let user_b = ctx
        .db
        .user()
        .name()
        .find(user_name)
        .ok_or("No user found")?;

    // we compute the hashes of the user ids and use that has the unique id
    // this way we avoid to iterate through each row and the database can optimize its access
    let mut hasher = DefaultHasher::new();
    ctx.sender.hash(&mut hasher);
    user_b.id.hash(&mut hasher);
    let hash_a = hasher.finish();

    // we compute two hashes because the ordering changes the hash
    let mut hasher = DefaultHasher::new();
    user_b.id.hash(&mut hasher);
    ctx.sender.hash(&mut hasher);
    let hash_b = hasher.finish();

    // check if the user are already friends with each other
    if ctx.db.friend().hash().find(hash_a).is_some()
        || ctx.db.friend().hash().find(hash_b).is_some()
    {
        return Err("Already a friend".into());
    }

    // check if the friendship was already requested
    if ctx.db.friend_request().hash().find(hash_a).is_some()
        || ctx.db.friend_request().hash().find(hash_b).is_some()
    {
        return Err("Already requested friendship".into());
    }

    // add the request to the database
    ctx.db.friend_request().insert(FriendRequest {
        hash: hash_a,
        user_a: user_a.id,
        user_b: user_b.id,
    });

    Ok(())
}

#[reducer]
pub fn accept_friend(ctx: &ReducerContext, user_name: String) -> Result<(), String> {
    // get users
    let user_a = ctx.db.user().id().find(ctx.sender).ok_or("No user found")?;
    let user_b = ctx
        .db
        .user()
        .name()
        .find(user_name)
        .ok_or("No user found")?;

    // compute hashes
    let mut hasher = DefaultHasher::new();
    ctx.sender.hash(&mut hasher);
    user_b.id.hash(&mut hasher);
    let hash_a = hasher.finish();

    // we compute two hashes because the ordering changes the hash
    let mut hasher = DefaultHasher::new();
    user_b.id.hash(&mut hasher);
    ctx.sender.hash(&mut hasher);
    let hash_b = hasher.finish();

    // check if the user are already friends with each other
    if ctx.db.friend().hash().find(hash_a).is_some()
        || ctx.db.friend().hash().find(hash_b).is_some()
    {
        return Err("Already a friend".into());
    }

    // compute whether to use hash_a or hash_b
    let is_not_a = ctx.db.friend_request().hash().find(hash_a).is_none();
    let is_not_b = ctx.db.friend_request().hash().find(hash_b).is_none();

    // if we can't use either hash_a or hash_b, then there's an error because the friendship wasn't requested
    if is_not_a || is_not_b {
        return Err("Friendship wasn't requested".into());
    }

    // use the correct hash
    let hash = if is_not_a { hash_b } else { hash_a };

    // remove the request
    ctx.db.friend_request().hash().delete(hash);
    // and add as friend
    ctx.db.friend().insert(Friend {
        hash,
        user_a: user_a.id,
        user_b: user_b.id,
    });

    Ok(())
}
