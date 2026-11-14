//! Blocking HTTP worker.

use super::*;

mod account;
mod bots;
mod guilds;
mod media;
mod messages;
mod moderation;
mod roles;
mod social;

pub(crate) fn http_worker(shared: Arc<Shared>, rx: mpsc::Receiver<Command>) {
    while let Ok(cmd) = rx.recv() {
        match cmd {
            Command::Ping => {}
            Command::Login { username, password } => account::login(&shared, username, password),
            Command::Register {
                username,
                display_name,
                password,
            } => account::register(&shared, username, display_name, password),
            Command::Logout => account::logout(&shared),
            Command::Bootstrap => account::bootstrap(&shared),
            Command::LoadGuildDetail { guild } => guilds::load_guild_detail(&shared, guild),
            Command::LoadMembers {
                guild,
                channel,
                offset,
                query,
                replace,
            } => guilds::load_members(&shared, guild, channel, offset, query, replace),
            Command::LoadUsers { users } => guilds::load_users(&shared, users),
            Command::LookupUser { query } => guilds::lookup_user(&shared, query),
            Command::LoadMessages { channel, before } => {
                messages::load_messages(&shared, channel, before)
            }
            Command::Send {
                channel,
                content,
                reply_to,
                attachments,
                nonce: _,
            } => messages::send(&shared, channel, content, reply_to, attachments),
            Command::Edit { message, content } => messages::edit(&shared, message, content),
            Command::Delete { message } => messages::delete(&shared, message),
            Command::React { message, emoji } => messages::react(&shared, message, emoji),
            Command::Pin { message } => messages::pin(&shared, message),
            Command::Typing { channel } => messages::typing(&shared, channel),
            Command::MarkRead { channel, message } => {
                messages::mark_read(&shared, channel, message)
            }
            Command::UpdateProfileExtras {
                activity,
                connections,
            } => account::update_profile_extras(&shared, activity, connections),
            Command::UploadActivityArtwork { path } => {
                account::upload_activity_artwork(&shared, path)
            }
            Command::RemoveActivityArtwork => account::remove_activity_artwork(&shared),
            Command::LoadGuildEvents { guild } => guilds::load_guild_events(&shared, guild),
            Command::CreateGuildEvent {
                guild,
                name,
                description,
                start_at,
                channel,
            } => guilds::create_guild_event(&shared, guild, name, description, start_at, channel),
            Command::DeleteGuildEvent { guild, event } => {
                guilds::delete_guild_event(&shared, guild, event)
            }
            Command::CheckServer { url } => account::check_server(&shared, url),
            Command::UpdateProfileStyle {
                effect,
                nameplate,
                widget,
            } => account::update_profile_style(&shared, effect, nameplate, widget),
            Command::LoadInbox => guilds::load_inbox(&shared),
            Command::Search { q } => messages::search(&shared, q),
            Command::CreateGuild { name, color } => guilds::create_guild(&shared, name, color),
            Command::LeaveGuild { guild } => guilds::leave_guild(&shared, guild),
            Command::SetChannelNotify {
                channel,
                level,
                suppress,
                clear,
            } => moderation::set_channel_notify(&shared, channel, level, suppress, clear),
            Command::SetGuildNotify { guild, settings } => {
                moderation::set_guild_notify(&shared, guild, settings)
            }
            Command::LoadPublicGuilds => guilds::load_public_guilds(&shared),
            Command::JoinPublicGuild { guild } => guilds::join_public_guild(&shared, guild),
            Command::LoadPins { channel } => messages::load_pins(&shared, channel),
            Command::SetPrivacy { dm, friend } => social::set_privacy(&shared, dm, friend),
            Command::SetGuildDiscovery {
                guild,
                discoverable,
            } => guilds::set_guild_discovery(&shared, guild, discoverable),
            Command::Video { channel, kind } => media::video(&shared, channel, kind),
            Command::CreateChannel {
                guild,
                name,
                kind,
                parent,
                topic,
            } => guilds::create_channel(&shared, guild, name, kind, parent, topic),
            Command::UpdateChannel {
                channel,
                name,
                topic,
                status,
                slowmode,
                nsfw,
            } => guilds::update_channel(&shared, channel, name, topic, status, slowmode, nsfw),
            Command::DeleteChannel { channel } => guilds::delete_channel(&shared, channel),
            Command::KickMember { guild, user } => moderation::kick_member(&shared, guild, user),
            Command::BanMember {
                guild,
                user,
                reason,
            } => moderation::ban_member(&shared, guild, user, reason),
            Command::TimeoutMember {
                guild,
                user,
                seconds,
            } => moderation::timeout_member(&shared, guild, user, seconds),
            Command::LoadBans { guild } => moderation::load_bans(&shared, guild),
            Command::UnbanMember { guild, user } => moderation::unban_member(&shared, guild, user),
            Command::UpdateGuildSettings {
                guild,
                name,
                description,
                color,
                tag,
                discoverable,
            } => guilds::update_guild_settings(
                &shared,
                guild,
                name,
                description,
                color,
                tag,
                discoverable,
            ),
            Command::PrunePreview { guild, days, roles } => {
                moderation::prune_preview(&shared, guild, days, roles)
            }
            Command::UploadGuildImage {
                guild,
                banner,
                path,
            } => guilds::upload_guild_image(&shared, guild, banner, path),
            Command::RemoveGuildImage { guild, banner } => {
                guilds::remove_guild_image(&shared, guild, banner)
            }
            Command::UploadUserImage { banner, path } => {
                media::upload_user_image(&shared, banner, path)
            }
            Command::RemoveUserImage { banner } => media::remove_user_image(&shared, banner),
            Command::RenameDm { channel, name } => social::rename_dm(&shared, channel, name),
            Command::UploadDmIcon { channel, path } => {
                social::upload_dm_icon(&shared, channel, path)
            }
            Command::RemoveDmIcon { channel } => social::remove_dm_icon(&shared, channel),
            Command::AddDmMember { channel, user } => social::add_dm_member(&shared, channel, user),
            Command::RemoveDmMember { channel, user } => {
                social::remove_dm_member(&shared, channel, user)
            }
            Command::LeaveDm { channel } => social::leave_dm(&shared, channel),
            Command::CreateGroupDm { users, name } => social::create_group_dm(&shared, users, name),
            Command::LoadMutuals { user } => social::load_mutuals(&shared, user),
            Command::BoostGuild { guild, on } => guilds::boost_guild(&shared, guild, on),
            Command::LoadAppeals { guild } => moderation::load_appeals(&shared, guild),
            Command::ResolveAppeal {
                guild,
                appeal,
                accept,
            } => moderation::resolve_appeal(&shared, guild, appeal, accept),
            Command::SubmitAppeal { code, message } => {
                moderation::submit_appeal(&shared, code, message)
            }
            Command::PruneMembers { guild, days, roles } => {
                moderation::prune_members(&shared, guild, days, roles)
            }
            Command::CreateInvite { guild } => guilds::create_invite(&shared, guild),
            Command::JoinInvite { code } => guilds::join_invite(&shared, code),
            Command::OpenDm { user } => social::open_dm(&shared, user),
            Command::FriendRequest { user } => social::friend_request(&shared, user),
            Command::FriendAccept { user } => social::friend_accept(&shared, user),
            Command::FriendRemove { user } => social::friend_remove(&shared, user),
            Command::FriendBlock { user } => social::friend_block(&shared, user),
            Command::CreateThread {
                channel,
                message,
                name,
            } => messages::create_thread(&shared, channel, message, name),
            Command::ArchiveThread { channel, archived } => {
                messages::archive_thread(&shared, channel, archived)
            }
            Command::LoadBots => bots::load_bots(&shared),
            Command::CreateBot { name, guild } => bots::create_bot(&shared, name, guild),
            Command::DeleteBot { id } => bots::delete_bot(&shared, id),
            Command::LoadApplications => bots::load_applications(&shared),
            Command::CreateApplication { name, description } => {
                bots::create_application(&shared, name, description)
            }
            Command::DeleteApplication { id } => bots::delete_application(&shared, id),
            Command::ApplicationToken { id, name } => bots::application_token(&shared, id, name),
            Command::ApplicationInstallLink { id } => bots::application_install_link(&shared, id),
            Command::InstallApplication { code, guild } => {
                bots::install_application(&shared, code, guild)
            }
            Command::UpdateProfile {
                display_name,
                pronouns,
                badges,
                avatar_decoration,
                custom_status,
                bio,
                color,
                avatar_border,
                banner_color,
            } => account::update_profile(
                &shared,
                display_name,
                pronouns,
                badges,
                avatar_decoration,
                custom_status,
                bio,
                color,
                avatar_border,
                banner_color,
            ),
            Command::ChangePassword { current, new } => {
                account::change_password(&shared, current, new)
            }
            Command::FetchImage { url } => media::fetch_image(&shared, url),
            Command::UpdateRole {
                guild,
                role,
                name,
                color,
                permissions,
                hoist,
                mentionable,
                position,
            } => roles::update_role(
                &shared,
                guild,
                role,
                name,
                color,
                permissions,
                hoist,
                mentionable,
                position,
            ),
            Command::CreateRole {
                guild,
                name,
                color,
                permissions,
            } => roles::create_role(&shared, guild, name, color, permissions),
            Command::DeleteRole { guild, role } => roles::delete_role(&shared, guild, role),
            Command::UploadRoleIcon { guild, role, path } => {
                roles::upload_role_icon(&shared, guild, role, path)
            }
            Command::RemoveRoleIcon { guild, role } => {
                roles::remove_role_icon(&shared, guild, role)
            }
            Command::LoadRoleLinks { guild, role } => roles::load_role_links(&shared, guild, role),
            Command::AddRoleLink {
                guild,
                role,
                label,
                url,
            } => roles::add_role_link(&shared, guild, role, label, url),
            Command::DeleteRoleLink { guild, role, link } => {
                roles::delete_role_link(&shared, guild, role, link)
            }
            Command::UpdateMemberRoles { guild, user, roles } => {
                roles::update_member_roles(&shared, guild, user, roles)
            }
            Command::LoadEmojis { guild } => roles::load_emojis(&shared, guild),
            Command::LoadGuildCommands { guild } => roles::load_guild_commands(&shared, guild),
            Command::InvokeCommand {
                channel,
                command,
                options,
                response,
            } => roles::invoke_command(&shared, channel, command, options, response),
            Command::LoadAudit { guild, before } => guilds::load_audit(&shared, guild, before),
            Command::CreateEmoji { guild, name, path } => {
                roles::create_emoji(&shared, guild, name, path)
            }
            Command::DeleteEmoji { guild, id } => roles::delete_emoji(&shared, guild, id),
            Command::SetOverwrite {
                channel,
                target,
                kind,
                allow,
                deny,
            } => guilds::set_overwrite(&shared, channel, target, kind, allow, deny),
            Command::ClearOverwrite {
                channel,
                target,
                kind,
            } => guilds::clear_overwrite(&shared, channel, target, kind),
            Command::Voice {
                guild,
                channel,
                mute,
                deaf,
            } => guilds::voice(&shared, guild, channel, mute, deaf),
            Command::Upload { channel, path } => media::upload(&shared, channel, path),
        }
    }
}

pub(crate) fn fetch_bootstrap(shared: &Arc<Shared>) {
    if let Ok(v) = shared.get("/api/bootstrap")
        && let Ok(b) = serde_json::from_value::<Bootstrap>(v)
    {
        shared.bootstrap_seen.store(true, Ordering::SeqCst);
        shared.emit(NetEvent::Bootstrap(Box::new(b)));
    }
}

pub(crate) fn complete_auth(shared: &Arc<Shared>, auth: AuthResponse) {
    if let Ok(mut token) = shared.token.lock() {
        *token = Some(auth.token.clone());
    }
    write_token(&shared.profile, Some(&auth.token));
    shared.emit(NetEvent::Session(Box::new(auth)));
    fetch_bootstrap(shared);
}
