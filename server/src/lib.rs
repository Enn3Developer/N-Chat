//! Entry point of the module
//!
//! WARNING: When changing anything related to the public api, remember to regenerate the spacetime bindings for the client:
//! `spacetime generate --lang csharp --out-dir Client/ModuleBindings --project-path server`

mod types;
mod validation;

use crate::types::Permission;
use crate::validation::{validate_message, validate_name};
use spacetimedb::{reducer, table, Identity, RangedIndex, ReducerContext, Table, Timestamp};
use std::hash::{DefaultHasher, Hash, Hasher};

pub type ReducerResult = Result<(), String>;

/// Defines a user
#[table(name = user, public)]
pub struct User {
    #[primary_key]
    id: Identity,
    #[unique]
    name: String,
    display_name: String,
    online: bool,
    created_at: Timestamp,
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

/// Defines a channel that has a name and the owner whom is a member of the channel
#[table(name = channel, public)]
pub struct Channel {
    #[primary_key]
    #[auto_inc]
    id: i128,
    #[unique]
    name: String,
    created_at: Timestamp,
    owner: Identity,
}

/// Defines a member, aka a user and which channel he's in
#[table(name = member, public)]
pub struct Member {
    #[unique]
    hash: u64,
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

/// Defines a guild that may contain channels and has a permission system and an owner whom is a member
#[table(name = guild, public)]
pub struct Guild {
    #[primary_key]
    #[auto_inc]
    id: i128,
    name: String,
    created_at: Timestamp,
    owner: Identity,
}

/// Defines a guild channel
#[table(name = guild_channel, public)]
pub struct GuildChannel {
    #[primary_key]
    #[auto_inc]
    id: i128,
    guild_id: i128,
    name: String,
    created_at: Timestamp,
}

/// Defines a guild member
#[table(name = guild_member, public, index(name = user_and_guild, btree(columns = [user_id, guild_id])))]
pub struct GuildMember {
    user_id: Identity,
    guild_id: i128,
}

/// Defines a role in the guild that has a name and a color
#[table(name = guild_role, public)]
pub struct GuildRole {
    #[primary_key]
    #[auto_inc]
    id: i128,
    guild_id: i128,
    name: String,
    // we use 32 bits per color to enable clients to use 10-bit depth colors
    // transparency is *not* supported
    color: u32,
}

/// Defines a permission for a role
#[table(name = guild_permission, public, index(name = role, btree(columns = [role_id])))]
pub struct GuildPermission {
    #[primary_key]
    #[auto_inc]
    id: i128,
    role_id: i128,
    permission: Permission,
}

/// Assigns a role to a member
#[table(name = guild_member_role, public, index(name = user_and_role, btree(columns = [user_id, role_id])), index(name = role_and_user, btree(columns = [role_id, user_id])))]
pub struct GuildMemberRole {
    user_id: Identity,
    role_id: i128,
}

#[table(name = guild_message, public)]
pub struct GuildMessage {
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
pub fn set_name(ctx: &ReducerContext, name: String) -> ReducerResult {
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
            name: name.clone(),
            // we set the display name as the id name because the user is newly created
            display_name: name,
            // we set it to true because we assume the user is connected before it can use reducers
            online: true,
            // we set `created_at` at the time of setting name because we don't have user data before this moment
            created_at: ctx.timestamp,
        });
    }

    Ok(())
}

#[reducer]
pub fn set_display_name(ctx: &ReducerContext, name: String) -> ReducerResult {
    // validate the name
    if !validate_name(&name) {
        return Err("Name isn't valid".into());
    }

    // get the user
    let user = ctx.db.user().id().find(ctx.sender).ok_or("No user found")?;

    // update its display name
    ctx.db.user().insert(User {
        display_name: name,
        ..user
    });

    Ok(())
}

#[reducer]
pub fn send_message(ctx: &ReducerContext, text: String, channel: String) -> ReducerResult {
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
pub fn create_channel(ctx: &ReducerContext, channel_name: String) -> ReducerResult {
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
        created_at: ctx.timestamp,
        owner: ctx.sender,
    });

    // compute the hash
    let mut hasher = DefaultHasher::new();
    user.id.hash(&mut hasher);
    channel.id.hash(&mut hasher);
    let hash = hasher.finish();

    // add the user as a member of the channel
    ctx.db.member().insert(Member {
        hash,
        user_id: user.id,
        channel_id: channel.id,
    });

    Ok(())
}

#[reducer]
pub fn add_user(ctx: &ReducerContext, channel: String, user_name: String) -> ReducerResult {
    // get the channel
    let channel = ctx
        .db
        .channel()
        .name()
        .find(&channel)
        .ok_or("No channel found")?;

    // check if the requesting user is a member of the channel
    if ctx.db.user().id().find(ctx.sender).is_none() {
        return Err("Not a member of the channel".into());
    }

    // get the user
    let user = ctx
        .db
        .user()
        .name()
        .find(user_name)
        .ok_or("No user found")?;

    // compute the hash
    let mut hasher = DefaultHasher::new();
    user.id.hash(&mut hasher);
    channel.id.hash(&mut hasher);
    let hash = hasher.finish();

    // check if the user is already a member of the channel
    if ctx.db.member().hash().find(hash).is_some() {
        return Err("Already a member of the channel".into());
    }

    // add the user as a member of the channel
    ctx.db.member().insert(Member {
        hash,
        user_id: user.id,
        channel_id: channel.id,
    });

    Ok(())
}

#[reducer]
pub fn remove_user(ctx: &ReducerContext, channel: String, user_name: String) -> ReducerResult {
    // get the channel
    let channel = ctx
        .db
        .channel()
        .name()
        .find(&channel)
        .ok_or("No channel found")?;

    // check if the requesting user is the owner of the channel
    // if it is, it is assumed it is also a member of the channel
    if channel.owner != ctx.sender {
        return Err("Only the owner can remove a user".into());
    }

    // get the user
    let user = ctx
        .db
        .user()
        .name()
        .find(user_name)
        .ok_or("No user found")?;

    // compute the hash
    let mut hasher = DefaultHasher::new();
    user.id.hash(&mut hasher);
    channel.id.hash(&mut hasher);
    let hash = hasher.finish();

    // check if the user is a member of the channel
    if ctx.db.member().hash().find(hash).is_none() {
        return Err("The user is not a member of the channel".into());
    }

    // check if the requesting user is trying to remove itself
    if user.id == ctx.sender {
        let members = ctx
            .db
            .member()
            .iter()
            .filter(|member| member.channel_id == channel.id)
            .count();

        // check if there are more than one member in the channel
        if members > 1 {
            return Err("The owner need to transfer the ownership first before removing itself from the channel".into());
        }

        // remove channel because there are no more members in it
        ctx.db.channel().id().delete(channel.id);
    }

    // remove the user
    ctx.db.member().hash().delete(hash);

    Ok(())
}

#[reducer]
pub fn add_friend(ctx: &ReducerContext, user_name: String) -> ReducerResult {
    // get users
    let user_a = ctx.db.user().id().find(ctx.sender).ok_or("No user found")?;
    let user_b = ctx
        .db
        .user()
        .name()
        .find(user_name)
        .ok_or("No user found")?;

    // default ordering of the ids
    let id_a = ctx.sender.min(user_b.id);
    let id_b = ctx.sender.max(user_b.id);

    // we compute the hash of the user ids and use that has the unique id
    // this way we avoid to iterate through each row and the database can optimize its access
    let mut hasher = DefaultHasher::new();
    id_a.hash(&mut hasher);
    id_b.hash(&mut hasher);
    let hash = hasher.finish();

    // check if the user are already friends with each other
    if ctx.db.friend().hash().find(hash).is_some() {
        return Err("Already a friend".into());
    }

    // check if the friendship was already requested
    if ctx.db.friend_request().hash().find(hash).is_some() {
        return Err("Already requested friendship".into());
    }

    // add the request to the database
    ctx.db.friend_request().insert(FriendRequest {
        hash,
        user_a: id_a,
        user_b: id_b,
    });

    Ok(())
}

#[reducer]
pub fn accept_friend(ctx: &ReducerContext, user_name: String) -> ReducerResult {
    // get users
    let user_a = ctx.db.user().id().find(ctx.sender).ok_or("No user found")?;
    let user_b = ctx
        .db
        .user()
        .name()
        .find(user_name)
        .ok_or("No user found")?;

    // default ordering of the ids
    let id_a = ctx.sender.min(user_b.id);
    let id_b = ctx.sender.max(user_b.id);

    // compute hashes
    let mut hasher = DefaultHasher::new();
    id_a.hash(&mut hasher);
    id_b.hash(&mut hasher);
    let hash = hasher.finish();

    // check if the user are already friends with each other
    if ctx.db.friend().hash().find(hash).is_some() {
        return Err("Already a friend".into());
    }

    // check if the friendship request exists
    if ctx.db.friend_request().hash().find(hash).is_none() {
        return Err("Friendship wasn't requested".into());
    }

    // remove the request
    ctx.db.friend_request().hash().delete(hash);
    // and add as friend
    ctx.db.friend().insert(Friend {
        hash,
        user_a: id_a,
        user_b: id_b,
    });

    Ok(())
}

#[reducer]
pub fn create_guild(ctx: &ReducerContext, name: String) -> ReducerResult {
    // get the user
    let user = ctx.db.user().id().find(ctx.sender).ok_or("No user found")?;

    // create the guild
    let guild = ctx.db.guild().insert(Guild {
        // the id is auto_inc so when committed it will change to the correct id and the returning
        // value has the correct id
        id: 0,
        name,
        owner: ctx.sender,
        created_at: ctx.timestamp,
    });

    // add the user as a member of the guild
    ctx.db.guild_member().insert(GuildMember {
        user_id: ctx.sender,
        guild_id: guild.id,
    });

    Ok(())
}

#[reducer]
pub fn join_guild(ctx: &ReducerContext, guild_id: i128) -> ReducerResult {
    // get the user
    let user = ctx.db.user().id().find(ctx.sender).ok_or("No user found")?;

    // get the guild
    let guild = ctx.db.guild().id().find(guild_id).ok_or("No guild found")?;

    // check if user is already a member of the guild
    if ctx
        .db
        .guild_member()
        .user_and_guild()
        .filter((ctx.sender, guild_id))
        .count()
        == 1
    {
        return Err("Already a member of the guild".into());
    }

    // add the user as a member
    ctx.db.guild_member().insert(GuildMember {
        user_id: ctx.sender,
        guild_id,
    });

    Ok(())
}

#[reducer]
pub fn create_role(
    ctx: &ReducerContext,
    guild_id: i128,
    name: String,
    color: u32,
) -> ReducerResult {
    // get the guild
    let guild = ctx.db.guild().id().find(guild_id).ok_or("No guild found")?;

    // check if the user is the owner of the guild
    if guild.owner != ctx.sender {
        return Err("You must be the owner".into());
    }

    // add the role
    ctx.db.guild_role().insert(GuildRole {
        id: 0,
        guild_id,
        name,
        color,
    });

    Ok(())
}

#[reducer]
pub fn set_role_name(ctx: &ReducerContext, role_id: i128, name: String) -> ReducerResult {
    // get the role
    let role = ctx
        .db
        .guild_role()
        .id()
        .find(role_id)
        .ok_or("No role found")?;

    // get the guild where the role is defined
    let guild = ctx
        .db
        .guild()
        .id()
        .find(role.guild_id)
        .ok_or("No guild found")?;

    // check if the user is the owner of the guild
    if guild.owner != ctx.sender {
        return Err("Only owner can change the role name".into());
    }

    // update the role
    ctx.db.guild_role().insert(GuildRole { name, ..role });

    Ok(())
}

#[reducer]
pub fn set_role_color(ctx: &ReducerContext, role_id: i128, color: u32) -> ReducerResult {
    // get the role
    let role = ctx
        .db
        .guild_role()
        .id()
        .find(role_id)
        .ok_or("No role found")?;

    // get the guild where the role is defined
    let guild = ctx
        .db
        .guild()
        .id()
        .find(role.guild_id)
        .ok_or("No guild found")?;

    // check if the user is the owner of the guild
    if guild.owner != ctx.sender {
        return Err("Only owner can change the role color".into());
    }

    // update the role
    ctx.db.guild_role().insert(GuildRole { color, ..role });

    Ok(())
}

#[reducer]
pub fn remove_role(ctx: &ReducerContext, role_id: i128) -> ReducerResult {
    // get the role
    let role = ctx
        .db
        .guild_role()
        .id()
        .find(role_id)
        .ok_or("No role found")?;

    // get the guild where the role is defined
    let guild = ctx
        .db
        .guild()
        .id()
        .find(role.guild_id)
        .ok_or("No guild found")?;

    // check if the user is the owner of the guild
    if guild.owner != ctx.sender {
        return Err("Only owner can remove a role".into());
    }

    // remove the role
    ctx.db.guild_role().id().delete(role_id);

    // remove all member roles linked to the role
    ctx.db.guild_member_role().role_and_user().delete(role_id);

    Ok(())
}

#[reducer]
pub fn add_permission(
    ctx: &ReducerContext,
    role_id: i128,
    permission: Permission,
) -> ReducerResult {
    // get the role
    let role = ctx
        .db
        .guild_role()
        .id()
        .find(role_id)
        .ok_or("No role found")?;

    // get the guild where the role is defined
    let guild = ctx
        .db
        .guild()
        .id()
        .find(role.guild_id)
        .ok_or("No guild found")?;

    // check if the user is the owner of the guild
    if guild.owner != ctx.sender {
        return Err("Only owner can add permissions to a role".into());
    }

    // check if there's already the same permission as the one currently adding
    if ctx
        .db
        .guild_permission()
        .role()
        .filter(role_id)
        .find(|guild_permission| guild_permission.permission == permission)
        .is_some()
    {
        return Err("Permission already added".into());
    }

    // add the permission
    ctx.db.guild_permission().insert(GuildPermission {
        id: 0,
        role_id,
        permission,
    });

    Ok(())
}

#[reducer]
pub fn remove_permission(ctx: &ReducerContext, permission_id: i128) -> ReducerResult {
    // get the permission
    let permission = ctx
        .db
        .guild_permission()
        .id()
        .find(permission_id)
        .ok_or("No permission found")?;

    // get the role
    let role = ctx
        .db
        .guild_role()
        .id()
        .find(permission.role_id)
        .ok_or("No role found")?;

    // get the guild where the role is defined
    let guild = ctx
        .db
        .guild()
        .id()
        .find(role.guild_id)
        .ok_or("No guild found")?;

    // check if the user is the owner of the guild
    if guild.owner != ctx.sender {
        return Err("Only owner can add permissions to a role".into());
    }

    ctx.db.guild_permission().id().delete(permission_id);

    Ok(())
}

#[reducer]
pub fn add_role_user(ctx: &ReducerContext, role_id: i128, user_name: String) -> ReducerResult {
    // get the user
    let user = ctx
        .db
        .user()
        .name()
        .find(user_name)
        .ok_or("No user found")?;

    // get the role
    let role = ctx
        .db
        .guild_role()
        .id()
        .find(role_id)
        .ok_or("No role found")?;

    // get the guild where the role is defined
    let guild = ctx
        .db
        .guild()
        .id()
        .find(role.guild_id)
        .ok_or("No guild found")?;

    // check if the user is the owner of the guild
    if guild.owner != ctx.sender {
        return Err("Only owner can add role to a user".into());
    }

    // check if user has the role
    if ctx
        .db
        .guild_member_role()
        .user_and_role()
        .filter((user.id, role_id))
        .count()
        > 1
    {
        return Err("User has already the role".into());
    }

    ctx.db.guild_member_role().insert(GuildMemberRole {
        user_id: user.id,
        role_id,
    });

    Ok(())
}

#[reducer]
pub fn remove_role_user(ctx: &ReducerContext, role_id: i128, user_name: String) -> ReducerResult {
    // get the user
    let user = ctx
        .db
        .user()
        .name()
        .find(user_name)
        .ok_or("No user found")?;

    // get the role
    let role = ctx
        .db
        .guild_role()
        .id()
        .find(role_id)
        .ok_or("No role found")?;

    // get the guild where the role is defined
    let guild = ctx
        .db
        .guild()
        .id()
        .find(role.guild_id)
        .ok_or("No guild found")?;

    // check if the user is the owner of the guild
    if guild.owner != ctx.sender {
        return Err("Only owner can remove role to a user".into());
    }

    // save the indexer
    let index = ctx.db.guild_member_role().user_and_role();

    // check if user has the role
    if index.filter((user.id, role_id)).count() == 0 {
        return Err("User doesn't have the role".into());
    }

    // remove the role from the user
    index.delete((user.id, role_id));

    Ok(())
}

#[reducer]
pub fn create_guild_channel(ctx: &ReducerContext, guild_id: i128, name: String) -> ReducerResult {
    // get the guild
    let guild = ctx.db.guild().id().find(guild_id).ok_or("No guild found")?;

    // check if the user is the owner of the guild
    if guild.owner != ctx.sender {
        return Err("Only owner can create a channel in the guild".into());
    }

    // create the channel
    ctx.db.guild_channel().insert(GuildChannel {
        id: 0,
        guild_id,
        name,
        created_at: ctx.timestamp,
    });

    Ok(())
}

#[reducer]
pub fn delete_guild_channel(ctx: &ReducerContext, channel_id: i128) -> ReducerResult {
    // get the channel
    let channel = ctx
        .db
        .guild_channel()
        .id()
        .find(channel_id)
        .ok_or("No channel found")?;

    // get the guild
    let guild = ctx
        .db
        .guild()
        .id()
        .find(channel.guild_id)
        .ok_or("No guild found")?;

    // check if the user is the owner of the guild
    if guild.owner != ctx.sender {
        return Err("Only owner can delete a channel in the guild".into());
    }

    // delete the channel
    ctx.db.guild_channel().id().delete(channel_id);

    Ok(())
}

pub fn send_guild_message(ctx: &ReducerContext, channel_id: i128, text: String) -> ReducerResult {
    // get the user
    let user = ctx.db.user().id().find(ctx.sender).ok_or("No user found")?;

    // get the channel
    let channel = ctx
        .db
        .guild_channel()
        .id()
        .find(channel_id)
        .ok_or("No channel found")?;

    // get the guild
    let guild = ctx
        .db
        .guild()
        .id()
        .find(channel.guild_id)
        .ok_or("No guild found")?;

    // check if the user is the owner of the guild
    if guild.owner != ctx.sender {
        // if he isn't
        // get the role
        let roles = ctx
            .db
            .guild_member_role()
            .user_and_role()
            .filter(ctx.sender);

        // initialize the variable that holds the bool of whether we found the correct permission
        let mut permission_found = false;

        // for every role the user has
        for role in roles {
            // get the permissions of the role
            let permissions = ctx.db.guild_permission().role().filter(role.role_id);

            // check for every permission if it can write to the channel
            if permissions
                .filter(|permission| {
                    if let Permission::Write(id) = permission.permission {
                        id == channel_id
                    } else {
                        false
                    }
                })
                .count()
                > 0
            {
                // found at least one permission
                permission_found = true;

                // exit the loop of the roles
                break;
            }
        }

        // check if we found the permission
        if !permission_found {
            return Err("You don't have enough permission".into());
        }
    }

    // validate the message
    if !validate_message(&text) {
        return Err("Message content isn't valid".into());
    }

    // add the message
    ctx.db.guild_message().insert(GuildMessage {
        // id is auto inc
        id: 0,
        sender: user.name,
        channel_id,
        sent: ctx.timestamp,
        text,
    });

    Ok(())
}
