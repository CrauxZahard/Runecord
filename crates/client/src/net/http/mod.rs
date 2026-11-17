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
        dispatch(&shared, cmd);
    }
}

/// Runs one command. The desktop worker and the browser loop share this, so a
/// handler only has to be written once.
pub(crate) fn dispatch(shared: &Arc<Shared>, cmd: Command) {
    match cmd {
        Command::Ping => {}
        Command::Login { username, password } => account::login(shared, username, password),
        Command::Register {
            username,
            display_name,
            password,
            signup_code,
        } => account::register(shared, username, display_name, password, signup_code),
        Command::Logout => account::logout(shared),
        Command::Bootstrap => account::bootstrap(shared),
        Command::LoadGuildDetail { guild } => guilds::load_guild_detail(shared, guild),
        Command::LoadMembers {
            guild,
            channel,
            offset,
            query,
            replace,
        } => guilds::load_members(shared, guild, channel, offset, query, replace),
        Command::LoadUsers { users } => guilds::load_users(shared, users),
        Command::LookupUser { query } => guilds::lookup_user(shared, query),
        Command::LoadMessages { channel, before } => {
            messages::load_messages(shared, channel, before)
        }
        Command::Send {
            channel,
            content,
            reply_to,
            attachments,
            nonce: _,
        } => messages::send(shared, channel, content, reply_to, attachments),
        Command::Edit { message, content } => messages::edit(shared, message, content),
        Command::ForwardMessage { message, channel } => messages::forward(shared, message, channel),
        Command::CreatePoll {
            channel,
            question,
            options,
            multiple,
            anonymous,
            duration_hours,
        } => messages::create_poll(
            shared,
            channel,
            question,
            options,
            multiple,
            anonymous,
            duration_hours,
        ),
        Command::VotePoll { message, option } => messages::vote_poll(shared, message, option),
        Command::ClickComponent { message, custom_id } => {
            messages::click_component(shared, message, custom_id)
        }
        Command::ScheduleMessage {
            channel,
            content,
            send_at,
        } => messages::schedule_message(shared, channel, content, send_at),
        Command::LoadScheduled { channel } => messages::load_scheduled(shared, channel),
        Command::DeleteScheduled { id } => messages::delete_scheduled(shared, id),
        Command::Delete { message } => messages::delete(shared, message),
        Command::React { message, emoji } => messages::react(shared, message, emoji),
        Command::Pin { message } => messages::pin(shared, message),
        Command::Typing { channel } => messages::typing(shared, channel),
        Command::MarkRead { channel, message } => messages::mark_read(shared, channel, message),
        Command::UpdateProfileExtras {
            activity,
            connections,
        } => account::update_profile_extras(shared, activity, connections),
        Command::ConnectionChallenge => account::connection_challenge(shared),
        Command::VerifyConnection { kind, handle, url } => {
            account::verify_connection(shared, kind, handle, url)
        }
        Command::UploadActivityArtwork { source } => {
            account::upload_activity_artwork(shared, source)
        }
        Command::RemoveActivityArtwork => account::remove_activity_artwork(shared),
        Command::LoadGuildEvents { guild } => guilds::load_guild_events(shared, guild),
        Command::CreateGuildEvent {
            guild,
            name,
            description,
            start_at,
            channel,
            recurrence,
            duration_minutes,
            reminder_minutes,
        } => guilds::create_guild_event(
            shared,
            guild,
            CreateGuildEvent {
                name,
                description,
                start_at,
                channel_id: channel,
                recurrence,
                duration_minutes,
                reminder_minutes,
            },
        ),
        Command::RsvpEvent {
            guild,
            event,
            status,
        } => guilds::rsvp_event(shared, guild, event, status),
        Command::DeleteGuildEvent { guild, event } => {
            guilds::delete_guild_event(shared, guild, event)
        }
        Command::CheckServer { url } => account::check_server(shared, url),
        Command::UpdateProfileStyle {
            effect,
            nameplate,
            widget,
        } => account::update_profile_style(shared, effect, nameplate, widget),
        Command::LoadInbox => guilds::load_inbox(shared),
        Command::Search { q } => messages::search(shared, q),
        Command::CreateGuild { name, color } => guilds::create_guild(shared, name, color),
        Command::LeaveGuild { guild } => guilds::leave_guild(shared, guild),
        Command::SetChannelNotify {
            channel,
            level,
            suppress,
            clear,
        } => moderation::set_channel_notify(shared, channel, level, suppress, clear),
        Command::SetGuildNotify { guild, settings } => {
            moderation::set_guild_notify(shared, guild, settings)
        }
        Command::LoadPublicGuilds => guilds::load_public_guilds(shared),
        Command::JoinPublicGuild { guild } => guilds::join_public_guild(shared, guild),
        Command::LoadPins { channel } => messages::load_pins(shared, channel),
        Command::SetPrivacy { dm, friend } => social::set_privacy(shared, dm, friend),
        Command::SetGuildDiscovery {
            guild,
            discoverable,
        } => guilds::set_guild_discovery(shared, guild, discoverable),
        Command::Video { channel, kind } => media::video(shared, channel, kind),
        Command::CreateChannel {
            guild,
            name,
            kind,
            parent,
            topic,
        } => guilds::create_channel(shared, guild, name, kind, parent, topic),
        Command::UpdateChannel {
            channel,
            name,
            topic,
            status,
            slowmode,
            nsfw,
        } => guilds::update_channel(shared, channel, name, topic, status, slowmode, nsfw),
        Command::DeleteChannel { channel } => guilds::delete_channel(shared, channel),
        Command::KickMember { guild, user } => moderation::kick_member(shared, guild, user),
        Command::BanMember {
            guild,
            user,
            reason,
        } => moderation::ban_member(shared, guild, user, reason),
        Command::TimeoutMember {
            guild,
            user,
            seconds,
        } => moderation::timeout_member(shared, guild, user, seconds),
        Command::LoadBans { guild } => moderation::load_bans(shared, guild),
        Command::UnbanMember { guild, user } => moderation::unban_member(shared, guild, user),
        Command::LoadWebhooks { guild } => guilds::load_webhooks(shared, guild),
        Command::CreateWebhook { channel, name } => guilds::create_webhook(shared, channel, name),
        Command::DeleteWebhook { guild, id } => guilds::delete_webhook(shared, guild, id),
        Command::LoadTemplates { guild } => guilds::load_templates(shared, guild),
        Command::CreateTemplate {
            guild,
            name,
            description,
        } => guilds::create_template(shared, guild, name, description),
        Command::DeleteTemplate { guild, template } => {
            guilds::delete_template(shared, guild, template)
        }
        Command::ApplyTemplate { code, name } => guilds::apply_template(shared, code, name),
        Command::LoadAutoMod { guild } => guilds::load_automod(shared, guild),
        Command::SaveAutoMod { guild, rules } => guilds::save_automod(shared, guild, rules),
        Command::UpdateOnboarding { guild, update } => {
            guilds::update_onboarding(shared, guild, update)
        }
        Command::AcceptOnboarding { guild } => guilds::accept_onboarding(shared, guild),
        Command::UpdateGuildSettings {
            guild,
            name,
            description,
            color,
            tag,
            discoverable,
        } => guilds::update_guild_settings(
            shared,
            guild,
            name,
            description,
            color,
            tag,
            discoverable,
        ),
        Command::PrunePreview { guild, days, roles } => {
            moderation::prune_preview(shared, guild, days, roles)
        }
        Command::UploadGuildImage {
            guild,
            banner,
            source,
        } => guilds::upload_guild_image(shared, guild, banner, source),
        Command::RemoveGuildImage { guild, banner } => {
            guilds::remove_guild_image(shared, guild, banner)
        }
        Command::UploadUserImage { banner, source } => {
            media::upload_user_image(shared, banner, source)
        }
        Command::RemoveUserImage { banner } => media::remove_user_image(shared, banner),
        Command::RenameDm { channel, name } => social::rename_dm(shared, channel, name),
        Command::UploadDmIcon { channel, source } => {
            social::upload_dm_icon(shared, channel, source)
        }
        Command::RemoveDmIcon { channel } => social::remove_dm_icon(shared, channel),
        Command::AddDmMember { channel, user } => social::add_dm_member(shared, channel, user),
        Command::RemoveDmMember { channel, user } => {
            social::remove_dm_member(shared, channel, user)
        }
        Command::LeaveDm { channel } => social::leave_dm(shared, channel),
        Command::CreateGroupDm { users, name } => social::create_group_dm(shared, users, name),
        Command::LoadMutuals { user } => social::load_mutuals(shared, user),
        Command::BoostGuild { guild, on } => guilds::boost_guild(shared, guild, on),
        Command::LoadAppeals { guild } => moderation::load_appeals(shared, guild),
        Command::ResolveAppeal {
            guild,
            appeal,
            accept,
        } => moderation::resolve_appeal(shared, guild, appeal, accept),
        Command::SubmitAppeal { code, message } => moderation::submit_appeal(shared, code, message),
        Command::PruneMembers { guild, days, roles } => {
            moderation::prune_members(shared, guild, days, roles)
        }
        Command::CreateInvite { guild } => guilds::create_invite(shared, guild),
        Command::JoinInvite { code } => guilds::join_invite(shared, code),
        Command::OpenDm { user } => social::open_dm(shared, user),
        Command::FriendRequest { user } => social::friend_request(shared, user),
        Command::FriendAccept { user } => social::friend_accept(shared, user),
        Command::FriendRemove { user } => social::friend_remove(shared, user),
        Command::FriendBlock { user } => social::friend_block(shared, user),
        Command::SetForumTags { channel, tags } => messages::set_forum_tags(shared, channel, tags),
        Command::SetThreadTags { thread, tags } => messages::set_thread_tags(shared, thread, tags),
        Command::CreateThread {
            channel,
            message,
            name,
            tag_ids,
        } => messages::create_thread(shared, channel, message, name, tag_ids),
        Command::ArchiveThread { channel, archived } => {
            messages::archive_thread(shared, channel, archived)
        }
        Command::LoadBots => bots::load_bots(shared),
        Command::CreateBot { name, guild } => bots::create_bot(shared, name, guild),
        Command::DeleteBot { id } => bots::delete_bot(shared, id),
        Command::LoadApplications => bots::load_applications(shared),
        Command::CreateApplication { name, description } => {
            bots::create_application(shared, name, description)
        }
        Command::DeleteApplication { id } => bots::delete_application(shared, id),
        Command::ApplicationToken { id, name } => bots::application_token(shared, id, name),
        Command::ApplicationInstallLink { id } => bots::application_install_link(shared, id),
        Command::InstallApplication { code, guild } => {
            bots::install_application(shared, code, guild)
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
            shared,
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
        Command::ChangePassword { current, new } => account::change_password(shared, current, new),
        Command::FetchImage { url } => media::fetch_image(shared, url),
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
            shared,
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
        } => roles::create_role(shared, guild, name, color, permissions),
        Command::DeleteRole { guild, role } => roles::delete_role(shared, guild, role),
        Command::UploadRoleIcon {
            guild,
            role,
            source,
        } => roles::upload_role_icon(shared, guild, role, source),
        Command::RemoveRoleIcon { guild, role } => roles::remove_role_icon(shared, guild, role),
        Command::LoadRoleLinks { guild, role } => roles::load_role_links(shared, guild, role),
        Command::AddRoleLink {
            guild,
            role,
            label,
            url,
        } => roles::add_role_link(shared, guild, role, label, url),
        Command::DeleteRoleLink { guild, role, link } => {
            roles::delete_role_link(shared, guild, role, link)
        }
        Command::UpdateMemberRoles { guild, user, roles } => {
            roles::update_member_roles(shared, guild, user, roles)
        }
        Command::LoadEmojis { guild } => roles::load_emojis(shared, guild),
        Command::LoadGuildCommands { guild } => roles::load_guild_commands(shared, guild),
        Command::InvokeCommand {
            channel,
            command,
            options,
            response,
        } => roles::invoke_command(shared, channel, command, options, response),
        Command::LoadAudit {
            guild,
            before,
            actor,
            action,
        } => guilds::load_audit(shared, guild, before, actor, action),
        Command::CreateEmoji {
            guild,
            name,
            sticker,
            source,
        } => roles::create_emoji(shared, guild, name, sticker, source),
        Command::DeleteEmoji { guild, id } => roles::delete_emoji(shared, guild, id),
        Command::SetOverwrite {
            channel,
            target,
            kind,
            allow,
            deny,
        } => guilds::set_overwrite(shared, channel, target, kind, allow, deny),
        Command::ClearOverwrite {
            channel,
            target,
            kind,
        } => guilds::clear_overwrite(shared, channel, target, kind),
        Command::Voice {
            guild,
            channel,
            mute,
            deaf,
        } => guilds::voice(shared, guild, channel, mute, deaf),
        Command::LoadGifs { query } => guilds::load_gifs(shared, query),
        Command::SendGif { channel, slug } => guilds::send_gif(shared, channel, slug),
        Command::LoadActivities { guild } => guilds::load_activities(shared, guild),
        Command::StartActivity { channel, activity } => {
            guilds::start_activity(shared, channel, activity)
        }
        Command::StopActivity { channel } => guilds::stop_activity(shared, channel),
        Command::StageAction {
            channel,
            action,
            user,
        } => guilds::stage_action(shared, channel, action, user),
        Command::LoadStage { channel } => guilds::load_stage(shared, channel),
        Command::Upload { channel, source } => media::upload(shared, channel, source),
    }
}

pub(crate) fn fetch_bootstrap(shared: &Arc<Shared>) {
    let done = shared.clone();
    get_then(
        shared,
        "/api/bootstrap".to_string(),
        move |result: Result<Bootstrap, String>| match result {
            Ok(bootstrap) => {
                done.bootstrap_seen.store(true, Ordering::SeqCst);
                done.emit(NetEvent::Bootstrap(Box::new(bootstrap)));
            }
            Err(error) => {
                // A server that is still starting is retried by the gateway and
                // the next bootstrap; only an answered-but-rejected token should
                // sign out.
                if error.contains("invalid or missing session") {
                    done.emit(NetEvent::Error(error));
                }
            }
        },
    );
}

pub(crate) fn complete_auth(shared: &Arc<Shared>, auth: AuthResponse) {
    if let Ok(mut token) = shared.token.lock() {
        *token = Some(auth.token.clone());
    }
    write_token(&shared.profile, Some(&auth.token));
    shared.emit(NetEvent::Session(Box::new(auth)));
    fetch_bootstrap(shared);
}
